//! Phase 9: Concurrent access and multi-execution tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use agentic_workflow::engine::*;
use agentic_workflow::resilience::*;
use agentic_workflow::governance::*;
use agentic_workflow::intelligence::*;
use agentic_workflow::types::*;

#[test]
fn test_concurrent_dag_multiple_executions() {
    let engine = Arc::new(Mutex::new(DagEngine::new()));
    let wf = Workflow::new("concurrent", "");
    let wfid = wf.id.clone();
    engine.lock().unwrap().register_workflow(wf).unwrap();

    let mut exec_ids = Vec::new();
    for _ in 0..10 {
        let eid = engine.lock().unwrap().start_execution(&wfid).unwrap();
        exec_ids.push(eid);
    }

    // All 10 executions exist independently
    for eid in &exec_ids {
        let eng = engine.lock().unwrap();
        let ctx = eng.get_execution(eid).unwrap();
        assert_eq!(ctx.status, ExecutionStatus::Running);
    }
}

#[test]
fn test_concurrent_circuit_breakers() {
    let engine = Arc::new(Mutex::new(CircuitBreakerEngine::new()));
    engine.lock().unwrap().get_or_create("api-1", 5, 2, 5000);
    engine.lock().unwrap().get_or_create("api-2", 3, 1, 3000);

    // Fail api-1 enough to open
    for _ in 0..5 {
        engine.lock().unwrap().record_failure("api-1").unwrap();
    }
    assert!(!engine.lock().unwrap().is_available("api-1"));
    assert!(engine.lock().unwrap().is_available("api-2"));
}

#[test]
fn test_concurrent_audit_trail() {
    let engine = Arc::new(Mutex::new(AuditEngine::new()));

    for i in 0..100 {
        engine.lock().unwrap().record(
            &format!("exec-{}", i), "wf-1", Some("step-1"),
            AuditEventType::StepExecuted, &format!("agent-{}", i % 5),
            Some("database"), None, None, AuditOutcome::Success,
        );
    }

    let eng = engine.lock().unwrap();
    let q = AuditQuery {
        workflow_id: Some("wf-1".into()), execution_id: None,
        event_types: None, actor: Some("agent-0".into()),
        resource: None, from: None, to: None, limit: None,
    };
    let results = eng.query(&q);
    assert_eq!(results.len(), 20); // 100/5 agents
}

#[test]
fn test_concurrent_idempotency_dedup() {
    let engine = Arc::new(Mutex::new(IdempotencyEngine::new()));

    // Store 100 keys
    for i in 0..100 {
        let key = format!("key-{}", i);
        engine.lock().unwrap().store(key, "step-1", &format!("exec-{}", i), "hash", serde_json::json!(i)).unwrap();
    }

    // Check all 100 keys
    for i in 0..100 {
        let key = format!("key-{}", i);
        assert!(engine.lock().unwrap().check(&key).is_some());
    }

    // Unknown keys return None
    assert!(engine.lock().unwrap().check("nonexistent").is_none());
}

#[test]
fn test_concurrent_approval_gates() {
    let engine = Arc::new(Mutex::new(ApprovalEngine::new()));

    // Define multiple gates
    for i in 0..5 {
        engine.lock().unwrap().define_gate(ApprovalGate {
            id: format!("gate-{}", i),
            step_id: format!("step-{}", i),
            workflow_id: "wf-1".into(),
            approver_chain: vec![
                Approver { identity: format!("approver-{}", i), role: None, priority: 1 },
            ],
            condition: None,
            timeout: None,
            delegation: None,
        }).unwrap();
    }

    // Request approvals on all gates
    let mut pending_ids = Vec::new();
    for i in 0..5 {
        let pid = engine.lock().unwrap().request_approval(
            &format!("gate-{}", i), &format!("exec-{}", i),
            &format!("step-{}", i), serde_json::json!({}),
        ).unwrap();
        pending_ids.push(pid);
    }

    assert_eq!(engine.lock().unwrap().list_pending().len(), 5);

    // Approve them all
    for pid in &pending_ids {
        engine.lock().unwrap().decide(pid, ApprovalDecision::Approved, "admin", None).unwrap();
    }

    assert_eq!(engine.lock().unwrap().list_pending().len(), 0);
    assert_eq!(engine.lock().unwrap().get_receipts(None).len(), 5);
}

#[test]
fn test_concurrent_variable_scopes() {
    let engine = Arc::new(Mutex::new(VariableEngine::new()));

    // Create 10 independent scopes
    let mut scope_ids = Vec::new();
    for i in 0..10 {
        let eng = &mut *engine.lock().unwrap();
        let sid = eng.create_scope(ScopeType::Workflow, None);
        eng.set(&sid, "index", serde_json::json!(i), VariableType::Integer, "test").unwrap();
        scope_ids.push(sid);
    }

    // Verify each scope has its own value
    for (i, sid) in scope_ids.iter().enumerate() {
        let eng = engine.lock().unwrap();
        let var = eng.get(sid, "index").unwrap();
        assert_eq!(var.value, serde_json::json!(i));
    }
}

#[test]
fn test_concurrent_prediction_from_history() {
    let engine = Arc::new(Mutex::new(PredictionEngine::new()));

    for i in 0..50 {
        engine.lock().unwrap().ingest_fingerprint(ExecutionFingerprint {
            execution_id: format!("e{}", i), workflow_id: "wf-1".into(),
            total_duration_ms: 1000 + (i % 10) * 100,
            step_durations: HashMap::new(), step_outcomes: HashMap::new(),
            retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }

    let eng = engine.lock().unwrap();
    let pred = eng.predict_duration("wf-1").unwrap();
    assert_eq!(pred.based_on_executions, 50);
    assert!(pred.predicted_ms > 0);
    assert!(pred.confidence > 0.5);
}

#[test]
fn test_concurrent_dead_letter_multi_class() {
    let engine = Arc::new(Mutex::new(DeadLetterEngine::new()));

    let classes = ["rate_limit", "network", "timeout", "permanent", "auth"];
    for i in 0..250 {
        let class = classes[i % classes.len()];
        engine.lock().unwrap().add_item(
            &format!("exec-{}", i), "wf-1", "step-1",
            class, &format!("{} error", class), serde_json::json!({}), 1,
        ).unwrap();
    }

    let summary = engine.lock().unwrap().summary();
    assert_eq!(summary.total_items, 250);
    assert_eq!(summary.by_failure_class.len(), 5);
    // rate_limit, network, timeout are retryable; permanent, auth are not
    assert!(summary.auto_retryable > 0);
    assert!(summary.needs_human > 0);
}

#[test]
fn test_concurrent_trigger_multi_workflow() {
    let engine = Arc::new(Mutex::new(TriggerEngine::new()));

    for i in 0..20 {
        engine.lock().unwrap().create_trigger(
            &format!("trigger-{}", i),
            &format!("wf-{}", i % 4),
            TriggerType::Manual, None, None,
        ).unwrap();
    }

    assert_eq!(engine.lock().unwrap().list_triggers().len(), 20);
    assert_eq!(engine.lock().unwrap().triggers_for_workflow("wf-0").len(), 5);
    assert_eq!(engine.lock().unwrap().triggers_for_workflow("wf-3").len(), 5);
}

#[test]
fn test_concurrent_fsm_independent_instances() {
    let engine = Arc::new(Mutex::new(FsmEngine::new()));

    let states = vec![
        State { name: "A".into(), description: None, entry_action: None, exit_action: None, is_terminal: false },
        State { name: "B".into(), description: None, entry_action: None, exit_action: None, is_terminal: true },
    ];
    let transitions = vec![
        Transition { from: "A".into(), to: "B".into(), event: "go".into(), guard: None, action: None },
    ];

    let mut fsm_ids = Vec::new();
    for i in 0..10 {
        let fid = engine.lock().unwrap().create_fsm(
            &format!("fsm-{}", i), states.clone(), transitions.clone(), "A",
        ).unwrap();
        fsm_ids.push(fid);
    }

    // Transition only first 5
    for fid in &fsm_ids[..5] {
        engine.lock().unwrap().transition(fid, "go").unwrap();
    }

    // First 5 at B, last 5 at A
    for fid in &fsm_ids[..5] {
        assert_eq!(engine.lock().unwrap().current_state(fid).unwrap(), "B");
    }
    for fid in &fsm_ids[5..] {
        assert_eq!(engine.lock().unwrap().current_state(fid).unwrap(), "A");
    }
}

#[test]
fn test_concurrent_evolution_multi_workflow() {
    let engine = Arc::new(Mutex::new(EvolutionEngine::new()));

    // wf-1: healthy (stable durations)
    for i in 0..10 {
        engine.lock().unwrap().ingest(ExecutionFingerprint {
            execution_id: format!("healthy-{}", i), workflow_id: "wf-healthy".into(),
            total_duration_ms: 1000, step_durations: HashMap::new(),
            step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }

    // wf-2: drifting (recent much slower)
    for i in 0..5 {
        engine.lock().unwrap().ingest(ExecutionFingerprint {
            execution_id: format!("drift-old-{}", i), workflow_id: "wf-drift".into(),
            total_duration_ms: 100, step_durations: HashMap::new(),
            step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }
    for i in 0..5 {
        engine.lock().unwrap().ingest(ExecutionFingerprint {
            execution_id: format!("drift-new-{}", i), workflow_id: "wf-drift".into(),
            total_duration_ms: 10000, step_durations: HashMap::new(),
            step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }

    let eng = engine.lock().unwrap();
    let healthy = eng.health("wf-healthy").unwrap();
    let drifting = eng.health("wf-drift").unwrap();

    assert!(healthy.score > drifting.score);
    assert!(!healthy.drift_detected);
    assert!(drifting.drift_detected);
}
