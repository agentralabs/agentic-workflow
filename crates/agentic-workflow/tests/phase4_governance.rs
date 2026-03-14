//! Phase 4: Governance tests — approval gates, audit trails, and scoped variables.

use std::collections::HashMap;

use agentic_workflow::governance::{ApprovalEngine, AuditEngine, VariableEngine};
use agentic_workflow::{
    ApprovalDecision, ApprovalGate, Approver, AuditEventType, AuditOutcome,
    AuditQuery, AuditRetention, ScopeType, VariableType,
};

// ─── Approval Engine ─────────────────────────────────────────────────

fn make_gate(id: &str, approvers: Vec<(&str, &str, u32)>) -> ApprovalGate {
    ApprovalGate {
        id: id.into(),
        step_id: format!("step-{}", id),
        workflow_id: "wf-1".into(),
        approver_chain: approvers
            .into_iter()
            .map(|(identity, role, priority)| Approver {
                identity: identity.into(),
                role: Some(role.into()),
                priority,
            })
            .collect(),
        condition: None,
        timeout: None,
        delegation: None,
    }
}

#[test]
fn approval_define_gate_with_approver_chain() {
    let mut engine = ApprovalEngine::new();
    let gate = make_gate("g1", vec![("alice", "lead", 1), ("bob", "mgr", 2)]);
    engine.define_gate(gate).unwrap();

    let stored = engine.get_gate("g1").unwrap();
    assert_eq!(stored.approver_chain.len(), 2);
    assert_eq!(stored.approver_chain[0].identity, "alice");
    assert_eq!(stored.approver_chain[1].identity, "bob");
}

#[test]
fn approval_request_creates_pending() {
    let mut engine = ApprovalEngine::new();
    engine
        .define_gate(make_gate("g1", vec![("alice", "lead", 1)]))
        .unwrap();

    let pid = engine
        .request_approval("g1", "exec-1", "step-g1", serde_json::json!({"env": "prod"}))
        .unwrap();

    assert!(!pid.is_empty());
    let pending = engine.list_pending();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].1.current_approver, "alice");
    assert_eq!(pending[0].1.gate_id, "g1");
}

#[test]
fn approval_approve_produces_receipt_with_checksum() {
    let mut engine = ApprovalEngine::new();
    engine
        .define_gate(make_gate("g1", vec![("alice", "lead", 1)]))
        .unwrap();

    let pid = engine
        .request_approval("g1", "exec-1", "step-g1", serde_json::json!({}))
        .unwrap();

    let receipt = engine
        .decide(&pid, ApprovalDecision::Approved, "alice", Some("LGTM".into()))
        .unwrap();

    assert!(matches!(receipt.decision, ApprovalDecision::Approved));
    assert_eq!(receipt.decided_by, "alice");
    assert_eq!(receipt.reason.as_deref(), Some("LGTM"));
    assert!(!receipt.checksum.is_empty());
    assert_eq!(engine.list_pending().len(), 0);
}

#[test]
fn approval_deny_decision() {
    let mut engine = ApprovalEngine::new();
    engine
        .define_gate(make_gate("g1", vec![("alice", "lead", 1)]))
        .unwrap();

    let pid = engine
        .request_approval("g1", "exec-1", "step-g1", serde_json::json!({}))
        .unwrap();

    let receipt = engine
        .decide(&pid, ApprovalDecision::Denied, "alice", Some("Not ready".into()))
        .unwrap();

    assert!(matches!(receipt.decision, ApprovalDecision::Denied));
    assert_eq!(receipt.reason.as_deref(), Some("Not ready"));
}

#[test]
fn approval_escalate_to_next_in_chain() {
    let mut engine = ApprovalEngine::new();
    engine
        .define_gate(make_gate(
            "g1",
            vec![("alice", "lead", 1), ("bob", "mgr", 2), ("carol", "vp", 3)],
        ))
        .unwrap();

    let pid = engine
        .request_approval("g1", "exec-1", "step-g1", serde_json::json!({}))
        .unwrap();

    // Starts with alice
    assert_eq!(engine.list_pending()[0].1.current_approver, "alice");

    engine.escalate(&pid).unwrap();
    assert_eq!(engine.list_pending()[0].1.current_approver, "bob");

    engine.escalate(&pid).unwrap();
    assert_eq!(engine.list_pending()[0].1.current_approver, "carol");
}

#[test]
fn approval_escalation_fails_at_end_of_chain() {
    let mut engine = ApprovalEngine::new();
    engine
        .define_gate(make_gate("g1", vec![("alice", "lead", 1)]))
        .unwrap();

    let pid = engine
        .request_approval("g1", "exec-1", "step-g1", serde_json::json!({}))
        .unwrap();

    let result = engine.escalate(&pid);
    assert!(result.is_err());
}

#[test]
fn approval_list_pending_multiple() {
    let mut engine = ApprovalEngine::new();
    engine
        .define_gate(make_gate("g1", vec![("alice", "lead", 1)]))
        .unwrap();
    engine
        .define_gate(make_gate("g2", vec![("bob", "mgr", 1)]))
        .unwrap();

    engine
        .request_approval("g1", "exec-1", "step-g1", serde_json::json!({}))
        .unwrap();
    engine
        .request_approval("g2", "exec-2", "step-g2", serde_json::json!({}))
        .unwrap();

    assert_eq!(engine.list_pending().len(), 2);
}

#[test]
fn approval_audit_trail_receipts_by_gate() {
    let mut engine = ApprovalEngine::new();
    engine
        .define_gate(make_gate("g1", vec![("alice", "lead", 1)]))
        .unwrap();
    engine
        .define_gate(make_gate("g2", vec![("bob", "mgr", 1)]))
        .unwrap();

    let p1 = engine
        .request_approval("g1", "exec-1", "step-g1", serde_json::json!({}))
        .unwrap();
    let p2 = engine
        .request_approval("g2", "exec-2", "step-g2", serde_json::json!({}))
        .unwrap();

    engine
        .decide(&p1, ApprovalDecision::Approved, "alice", None)
        .unwrap();
    engine
        .decide(&p2, ApprovalDecision::Denied, "bob", None)
        .unwrap();

    let g1_receipts = engine.get_receipts(Some("g1"));
    assert_eq!(g1_receipts.len(), 1);
    assert!(matches!(g1_receipts[0].decision, ApprovalDecision::Approved));

    let all_receipts = engine.get_receipts(None);
    assert_eq!(all_receipts.len(), 2);
}

// ─── Audit Engine ────────────────────────────────────────────────────

fn record_sample_events(engine: &mut AuditEngine) {
    engine.record(
        "exec-1", "wf-1", Some("step-a"), AuditEventType::StepExecuted,
        "alice", Some("db-main"), None, None, AuditOutcome::Success,
    );
    engine.record(
        "exec-1", "wf-1", Some("step-b"), AuditEventType::StepExecuted,
        "bob", Some("cache"), None, None, AuditOutcome::Success,
    );
    engine.record(
        "exec-2", "wf-2", None, AuditEventType::WorkflowStarted,
        "alice", None, None, None, AuditOutcome::Success,
    );
    engine.record(
        "exec-3", "wf-1", Some("step-a"), AuditEventType::StepExecuted,
        "carol", Some("db-main"),
        Some(serde_json::json!({"query": "SELECT 1"})),
        Some(serde_json::json!({"rows": 1})),
        AuditOutcome::Success,
    );
}

#[test]
fn audit_record_events_all_fields() {
    let mut engine = AuditEngine::new();
    let eid = engine.record(
        "exec-1", "wf-1", Some("step-a"), AuditEventType::StepExecuted,
        "alice", Some("db-main"),
        Some(serde_json::json!({"q": 1})),
        Some(serde_json::json!({"r": 2})),
        AuditOutcome::Success,
    );
    assert!(!eid.is_empty());
    assert_eq!(engine.event_count(), 1);
}

#[test]
fn audit_query_by_workflow_id() {
    let mut engine = AuditEngine::new();
    record_sample_events(&mut engine);

    let q = AuditQuery {
        workflow_id: Some("wf-1".into()),
        execution_id: None, event_types: None, actor: None,
        resource: None, from: None, to: None, limit: None,
    };
    let results = engine.query(&q);
    assert_eq!(results.len(), 3);
}

#[test]
fn audit_query_by_actor() {
    let mut engine = AuditEngine::new();
    record_sample_events(&mut engine);

    let q = AuditQuery {
        workflow_id: None, execution_id: None, event_types: None,
        actor: Some("alice".into()),
        resource: None, from: None, to: None, limit: None,
    };
    let results = engine.query(&q);
    assert_eq!(results.len(), 2); // exec-1/step-a + exec-2
}

#[test]
fn audit_query_by_resource() {
    let mut engine = AuditEngine::new();
    record_sample_events(&mut engine);

    let q = AuditQuery {
        workflow_id: None, execution_id: None, event_types: None,
        actor: None, resource: Some("db-main".into()),
        from: None, to: None, limit: None,
    };
    let results = engine.query(&q);
    assert_eq!(results.len(), 2);
}

#[test]
fn audit_query_by_time_range() {
    let mut engine = AuditEngine::new();
    let before = chrono::Utc::now();
    record_sample_events(&mut engine);
    let after = chrono::Utc::now();

    let q = AuditQuery {
        workflow_id: None, execution_id: None, event_types: None,
        actor: None, resource: None,
        from: Some(before), to: Some(after), limit: None,
    };
    let results = engine.query(&q);
    assert_eq!(results.len(), 4);
}

#[test]
fn audit_timeline_chronological() {
    let mut engine = AuditEngine::new();
    record_sample_events(&mut engine);

    let timeline = engine.timeline(None, 100);
    assert_eq!(timeline.len(), 4);
    for window in timeline.windows(2) {
        assert!(window[0].timestamp <= window[1].timestamp);
    }
}

#[test]
fn audit_impact_analysis_for_resource() {
    let mut engine = AuditEngine::new();
    record_sample_events(&mut engine);

    let impact = engine.impact_analysis("db-main");
    assert_eq!(impact.resource, "db-main");
    assert_eq!(impact.event_count, 2);
    assert!(impact.workflow_ids.contains(&"wf-1".to_string()));
    assert!(impact.first_touch <= impact.last_touch);
}

#[test]
fn audit_export_as_json() {
    let mut engine = AuditEngine::new();
    record_sample_events(&mut engine);

    let q = AuditQuery {
        workflow_id: Some("wf-1".into()),
        execution_id: None, event_types: None, actor: None,
        resource: None, from: None, to: None, limit: None,
    };
    let json_str = engine.export(&q).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 3);
}

#[test]
fn audit_retention_policy() {
    let mut engine = AuditEngine::new();
    assert_eq!(engine.get_retention().retain_days, 90);

    engine.set_retention(AuditRetention {
        retain_days: 365,
        compliance_preset: None,
        archive_after_days: Some(730),
    });
    assert_eq!(engine.get_retention().retain_days, 365);
    assert_eq!(engine.get_retention().archive_after_days, Some(730));
}

// Variable engine tests are in phase4_variable.rs

// (end of file — variable tests split to phase4_variable.rs to stay under 400 lines)
