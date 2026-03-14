//! Phase 7: Stress tests — scale, performance, and durability.

use std::collections::HashMap;
use std::time::Instant;
use agentic_workflow::types::*;
use agentic_workflow::engine::*;
use agentic_workflow::resilience::*;
use agentic_workflow::governance::*;
use agentic_workflow::intelligence::*;
use agentic_workflow::format::{AwfWriter, AwfReader};
use std::io::Cursor;

#[test]
fn stress_dag_1000_steps_linear_chain() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("1k-linear", "");
    let mut prev_id = String::new();
    for i in 0..1000 {
        let s = StepNode::new(format!("S{}", i), StepType::Noop);
        let sid = s.id.clone();
        wf.add_step(s);
        if i > 0 {
            wf.add_edge(Edge { from: prev_id.clone(), to: sid.clone(), edge_type: EdgeType::Sequence });
        }
        prev_id = sid;
    }
    let start = Instant::now();
    assert!(engine.validate_dag(&wf).is_ok());
    let order = engine.topological_sort(&wf).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(order.len(), 1000);
    assert!(elapsed.as_millis() < 1000, "1K DAG validation took {}ms", elapsed.as_millis());
}

#[test]
fn stress_dag_1000_parallel_steps() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("1k-parallel", "");
    for i in 0..1000 {
        wf.add_step(StepNode::new(format!("P{}", i), StepType::Noop));
    }
    let start = Instant::now();
    assert!(engine.validate_dag(&wf).is_ok());
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "1K parallel validation took {}ms", elapsed.as_millis());
}

#[test]
fn stress_execution_start_100() {
    let mut engine = DagEngine::new();
    let wf = Workflow::new("multi-exec", "");
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let start = Instant::now();
    for _ in 0..100 {
        engine.start_execution(&wfid).unwrap();
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 500, "100 executions took {}ms", elapsed.as_millis());
}

#[test]
fn stress_batch_10k_items() {
    let mut engine = BatchEngine::new();
    let items: Vec<serde_json::Value> = (0..10_000).map(|i| serde_json::json!({"id": i})).collect();
    let start = Instant::now();
    let bid = engine.create_batch("wf", items, 10, 100).unwrap();
    let p = engine.get_progress(&bid).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(p.total_items, 10_000);
    assert!(elapsed.as_millis() < 2000, "10K batch creation took {}ms", elapsed.as_millis());
}

#[test]
fn stress_dead_letter_10k() {
    let mut engine = DeadLetterEngine::new();
    let start = Instant::now();
    for i in 0..10_000 {
        let class = if i % 3 == 0 { "rate_limit" } else if i % 3 == 1 { "network" } else { "permanent" };
        engine.add_item(&format!("e{}", i), "wf", "s1", class, "error", serde_json::json!({}), 1).unwrap();
    }
    let summary = engine.summary();
    let elapsed = start.elapsed();
    assert_eq!(summary.total_items, 10_000);
    assert!(summary.by_failure_class.len() >= 2);
    assert!(elapsed.as_millis() < 2000, "10K dead letters took {}ms", elapsed.as_millis());
}

#[test]
fn stress_idempotency_10k_keys() {
    let mut engine = IdempotencyEngine::new();
    let start = Instant::now();
    for i in 0..10_000 {
        let key = format!("key-{}", i);
        engine.store(key, "step-1", &format!("exec-{}", i), "hash", serde_json::json!(i)).unwrap();
    }
    let elapsed_write = start.elapsed();

    let start = Instant::now();
    for i in 0..10_000 {
        let key = format!("key-{}", i);
        assert!(engine.check(&key).is_some());
    }
    let elapsed_read = start.elapsed();
    assert!(elapsed_write.as_millis() < 2000, "10K idempotency writes took {}ms", elapsed_write.as_millis());
    assert!(elapsed_read.as_millis() < 1000, "10K idempotency reads took {}ms", elapsed_read.as_millis());
}

#[test]
fn stress_audit_10k_events() {
    let mut engine = AuditEngine::new();
    let start = Instant::now();
    for i in 0..10_000 {
        engine.record(
            &format!("exec-{}", i), "wf-1", Some("step-1"),
            AuditEventType::StepExecuted, "system", Some("db"),
            None, None, AuditOutcome::Success,
        );
    }
    let elapsed_write = start.elapsed();

    let start = Instant::now();
    let q = AuditQuery {
        workflow_id: Some("wf-1".into()), execution_id: None,
        event_types: None, actor: None, resource: None,
        from: None, to: None, limit: Some(100),
    };
    let results = engine.query(&q);
    let elapsed_query = start.elapsed();
    assert_eq!(results.len(), 100);
    assert!(elapsed_write.as_millis() < 2000, "10K audit writes took {}ms", elapsed_write.as_millis());
    assert!(elapsed_query.as_millis() < 500, "10K audit query took {}ms", elapsed_query.as_millis());
}

#[test]
fn stress_variable_1000_scopes() {
    let mut engine = VariableEngine::new();
    let mut parent = engine.create_scope(ScopeType::Workflow, None);
    engine.set(&parent, "root", serde_json::json!("root-value"), VariableType::String, "test").unwrap();
    for i in 0..1000 {
        let child = engine.create_scope(ScopeType::Step, Some(&parent));
        engine.set(&child, &format!("local-{}", i), serde_json::json!(i), VariableType::Integer, "test").unwrap();
        parent = child;
    }
    // Deepest scope can still read root variable via cascade
    let val = engine.get(&parent, "root").unwrap();
    assert_eq!(val.value, serde_json::json!("root-value"));
}

#[test]
fn stress_fsm_1000_transitions() {
    let mut engine = FsmEngine::new();
    let mut states = Vec::new();
    let mut transitions = Vec::new();
    for i in 0..1001 {
        states.push(State {
            name: format!("S{}", i), description: None,
            entry_action: None, exit_action: None, is_terminal: i == 1000,
        });
        if i > 0 {
            transitions.push(Transition {
                from: format!("S{}", i - 1), to: format!("S{}", i),
                event: format!("go{}", i), guard: None, action: None,
            });
        }
    }
    let fid = engine.create_fsm("long", states, transitions, "S0").unwrap();
    let start = Instant::now();
    for i in 1..=1000 {
        engine.transition(&fid, &format!("go{}", i)).unwrap();
    }
    let elapsed = start.elapsed();
    assert_eq!(engine.current_state(&fid).unwrap(), "S1000");
    assert_eq!(engine.get_history(&fid).unwrap().len(), 1000);
    assert!(elapsed.as_millis() < 2000, "1K FSM transitions took {}ms", elapsed.as_millis());
}

#[test]
fn stress_archaeology_1000_fingerprints() {
    let mut engine = ArchaeologyEngine::new();
    for i in 0..1000 {
        engine.record_fingerprint(ExecutionFingerprint {
            execution_id: format!("e{}", i), workflow_id: "wf".into(),
            total_duration_ms: 1000 + (i % 50) * 10,
            step_durations: HashMap::new(), step_outcomes: HashMap::new(),
            retry_count: 0, completed_at: chrono::Utc::now(),
        });
    }
    // Add outlier
    engine.record_fingerprint(ExecutionFingerprint {
        execution_id: "outlier".into(), workflow_id: "wf".into(),
        total_duration_ms: 1_000_000, step_durations: HashMap::new(),
        step_outcomes: HashMap::new(), retry_count: 0, completed_at: chrono::Utc::now(),
    });
    let start = Instant::now();
    let anomalies = engine.detect_anomalies("wf");
    let elapsed = start.elapsed();
    assert!(!anomalies.is_empty());
    assert!(elapsed.as_millis() < 500, "Anomaly detection took {}ms", elapsed.as_millis());
}

#[test]
fn stress_circuit_breaker_rapid_failures() {
    let mut engine = CircuitBreakerEngine::new();
    engine.get_or_create("api", 100, 10, 5000);
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = engine.record_failure("api");
    }
    let elapsed = start.elapsed();
    assert!(!engine.is_available("api"));
    engine.reset("api").unwrap();
    assert!(engine.is_available("api"));
    assert!(elapsed.as_millis() < 500, "1K circuit breaker failures took {}ms", elapsed.as_millis());
}

#[test]
fn stress_trigger_10k_activations() {
    let mut engine = TriggerEngine::new();
    let tid = engine.create_trigger("t1", "wf", TriggerType::Manual, None, None).unwrap();
    let start = Instant::now();
    for i in 0..10_000 {
        engine.record_activation(&tid, &format!("exec-{}", i), serde_json::json!({}), true).unwrap();
    }
    let elapsed = start.elapsed();
    assert_eq!(engine.activation_history(&tid).len(), 10_000);
    assert!(elapsed.as_millis() < 3000, "10K trigger activations took {}ms", elapsed.as_millis());
}

#[test]
fn stress_collective_1000_workflows() {
    let mut engine = CollectiveEngine::new();
    for i in 0..1000 {
        engine.share(&format!("Workflow-{}", i), &format!("Desc {}", i),
            serde_json::json!({"step": i}), "team", vec![format!("tag-{}", i % 10)]);
    }
    let start = Instant::now();
    let results = engine.search("Workflow-50");
    let elapsed = start.elapsed();
    assert!(!results.is_empty());
    assert!(elapsed.as_millis() < 500, "Search 1K collective took {}ms", elapsed.as_millis());
}

#[test]
fn stress_awf_format_100_workflows() {
    let mut buf = Vec::new();
    {
        let mut writer = AwfWriter::new(&mut buf);
        writer.write_header().unwrap();
        for i in 0..100 {
            let mut wf = Workflow::new(format!("wf-{}", i), format!("Workflow {}", i));
            wf.add_step(StepNode::new("S1", StepType::Noop));
            writer.write_workflow(&wf).unwrap();
        }
        writer.finish().unwrap();
    }
    // Read back
    let cursor = Cursor::new(buf.clone());
    let mut reader = AwfReader::new(cursor);
    reader.read_header().unwrap();
    let start = Instant::now();
    for _ in 0..100 {
        let wf = reader.read_workflow().unwrap();
        assert!(!wf.name.is_empty());
    }
    let elapsed = start.elapsed();
    assert!(elapsed.as_millis() < 1000, "Reading 100 .awf workflows took {}ms", elapsed.as_millis());
}
