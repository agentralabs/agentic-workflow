//! Phase 1b: Variable type matching, approval/audit variants, serde roundtrips, strategy tests.

use std::collections::HashMap;
use agentic_workflow::types::*;
use agentic_workflow::types::workflow::*;
use agentic_workflow::types::execution::*;
use agentic_workflow::types::schedule::*;
use agentic_workflow::types::trigger::*;
use agentic_workflow::types::retry::*;
use agentic_workflow::types::variable::*;
use agentic_workflow::types::approval::*;
use agentic_workflow::types::audit::*;
use agentic_workflow::types::rollback::*;
use agentic_workflow::types::idempotency::*;
use agentic_workflow::types::template::*;

// ── VariableType::matches() ──

#[test]
fn test_variable_type_matches_string() {
    assert!(VariableType::String.matches(&serde_json::json!("hello")));
    assert!(!VariableType::String.matches(&serde_json::json!(42)));
}

#[test]
fn test_variable_type_matches_integer() {
    assert!(VariableType::Integer.matches(&serde_json::json!(42)));
    assert!(!VariableType::Integer.matches(&serde_json::json!("42")));
}

#[test]
fn test_variable_type_matches_float() {
    assert!(VariableType::Float.matches(&serde_json::json!(3.14)));
}

#[test]
fn test_variable_type_matches_boolean() {
    assert!(VariableType::Boolean.matches(&serde_json::json!(true)));
    assert!(!VariableType::Boolean.matches(&serde_json::json!(1)));
}

#[test]
fn test_variable_type_matches_array() {
    assert!(VariableType::Array.matches(&serde_json::json!([1, 2, 3])));
    assert!(!VariableType::Array.matches(&serde_json::json!({})));
}

#[test]
fn test_variable_type_matches_object() {
    assert!(VariableType::Object.matches(&serde_json::json!({"k": "v"})));
    assert!(!VariableType::Object.matches(&serde_json::json!([1])));
}

#[test]
fn test_variable_type_matches_null() {
    assert!(VariableType::Null.matches(&serde_json::Value::Null));
    assert!(!VariableType::Null.matches(&serde_json::json!(0)));
}

#[test]
fn test_variable_type_matches_any() {
    assert!(VariableType::Any.matches(&serde_json::json!("anything")));
    assert!(VariableType::Any.matches(&serde_json::json!(42)));
    assert!(VariableType::Any.matches(&serde_json::Value::Null));
}

// ── ApprovalDecision variants ──

#[test]
fn test_approval_decision_variants() {
    let _a = ApprovalDecision::Approved;
    let _d = ApprovalDecision::Denied;
    let _e = ApprovalDecision::Escalated;
    let _t = ApprovalDecision::TimedOut;
    let _del = ApprovalDecision::Delegated { to: "backup-admin".into() };
}

// ── AuditEventType / AuditOutcome variants ──

#[test]
fn test_audit_event_type_variants() {
    let _wc = AuditEventType::WorkflowCreated;
    let _ws = AuditEventType::WorkflowStarted;
    let _se = AuditEventType::StepExecuted;
    let _tf = AuditEventType::TriggerFired;
    let _custom = AuditEventType::Custom("deploy".into());
}

#[test]
fn test_audit_outcome_variants() {
    let _s = AuditOutcome::Success;
    let _f = AuditOutcome::Failure { reason: "timeout".into() };
    let _sk = AuditOutcome::Skipped { reason: "disabled".into() };
    let _p = AuditOutcome::Pending;
}

// ── Serialization roundtrip ──

#[test]
fn test_serde_roundtrip_workflow() {
    let mut wf = Workflow::new("serde-test", "Roundtrip");
    wf.add_step(StepNode::new("A", StepType::Noop));
    let json = serde_json::to_string(&wf).expect("serialize");
    let back: Workflow = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.name, "serde-test");
    assert_eq!(back.steps.len(), 1);
}

#[test]
fn test_serde_roundtrip_execution_context() {
    let ctx = ExecutionContext {
        execution_id: "e1".into(), workflow_id: "w1".into(),
        status: ExecutionStatus::Running, step_states: HashMap::new(),
        variables: HashMap::new(), started_at: chrono::Utc::now(),
        completed_at: None, trigger_info: None, metadata: HashMap::new(),
    };
    let json = serde_json::to_string(&ctx).expect("serialize");
    let back: ExecutionContext = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.execution_id, "e1");
}

#[test]
fn test_serde_roundtrip_schedule() {
    let s = Schedule {
        id: "s1".into(), workflow_id: "w1".into(),
        expression: ScheduleExpression::Cron("0 * * * *".into()),
        conflict_policy: ConflictPolicy::Skip, enabled: true,
        next_fire_at: None, last_fired_at: None,
        timezone: "UTC".into(), created_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: Schedule = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "s1");
}

#[test]
fn test_serde_roundtrip_trigger() {
    let t = Trigger {
        id: "t1".into(), name: "on-push".into(), workflow_id: "w1".into(),
        trigger_type: TriggerType::Manual, condition: None,
        debounce_ms: None, enabled: true, created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    };
    let json = serde_json::to_string(&t).unwrap();
    let back: Trigger = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "on-push");
}

// ── CompletionPolicy variants ──

#[test]
fn test_completion_policy_variants() {
    let _all = CompletionPolicy::WaitAll;
    let _any = CompletionPolicy::WaitAny;
    let _n = CompletionPolicy::WaitN(3);
    let _t = CompletionPolicy::WaitTimeout(5000);
}

// ── RetryStrategy calculation ──

#[test]
fn test_retry_strategy_exponential_backoff() {
    let engine = agentic_workflow::resilience::RetryEngine::new();
    let s = RetryStrategy::ExponentialBackoff {
        initial_ms: 100, max_ms: 10_000, multiplier: 2.0,
    };
    assert_eq!(engine.calculate_delay(&s, 0), 100);
    assert_eq!(engine.calculate_delay(&s, 1), 200);
    assert_eq!(engine.calculate_delay(&s, 2), 400);
    assert_eq!(engine.calculate_delay(&s, 3), 800);
    assert_eq!(engine.calculate_delay(&s, 10), 10_000);
}

#[test]
fn test_retry_strategy_linear() {
    let engine = agentic_workflow::resilience::RetryEngine::new();
    let s = RetryStrategy::Linear { delay_ms: 100, increment_ms: 50 };
    assert_eq!(engine.calculate_delay(&s, 0), 100);
    assert_eq!(engine.calculate_delay(&s, 1), 150);
    assert_eq!(engine.calculate_delay(&s, 4), 300);
}

#[test]
fn test_retry_strategy_fixed() {
    let engine = agentic_workflow::resilience::RetryEngine::new();
    let s = RetryStrategy::FixedDelay { delay_ms: 500 };
    assert_eq!(engine.calculate_delay(&s, 0), 500);
    assert_eq!(engine.calculate_delay(&s, 5), 500);
}

#[test]
fn test_retry_strategy_immediate() {
    let engine = agentic_workflow::resilience::RetryEngine::new();
    assert_eq!(engine.calculate_delay(&RetryStrategy::Immediate, 0), 0);
    assert_eq!(engine.calculate_delay(&RetryStrategy::Immediate, 99), 0);
}

// ── RollbackScope variants ──

#[test]
fn test_rollback_scope_variants() {
    let _full = RollbackScope::Full;
    let _from = RollbackScope::FromStep { step_id: "s3".into() };
    let _sel = RollbackScope::Selective { step_ids: vec!["s1".into(), "s3".into()] };
}

// ── IdempotencyWindow variants ──

#[test]
fn test_idempotency_window_variants() {
    let _dur = IdempotencyWindow::Duration { ms: 60_000 };
    let _forever = IdempotencyWindow::Forever;
    let _next = IdempotencyWindow::UntilNextExecution;
}

// ── TemplateParameter types ──

#[test]
fn test_template_parameter_types() {
    let p = TemplateParameter {
        name: "env".into(), description: "Target environment".into(),
        param_type: ParameterType::Enum { values: vec!["dev".into(), "prod".into()] },
        required: true, default: Some(serde_json::json!("dev")),
        validation: None,
    };
    assert!(p.required);
    assert!(matches!(p.param_type, ParameterType::Enum { .. }));
}
