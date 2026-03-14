//! Phase 3: Resilience tests — retry, rollback, circuit breaker, dead letter, idempotency.

use agentic_workflow::resilience::*;
use agentic_workflow::types::*;

// === Retry Engine ===

#[test]
fn test_retry_configure_policy() {
    let mut engine = RetryEngine::new();
    let profiles = vec![
        RetryProfile { failure_class: FailureClass::Transient, max_attempts: 5,
            strategy: RetryStrategy::ExponentialBackoff { initial_ms: 100, max_ms: 10000, multiplier: 2.0 }, jitter: true },
        RetryProfile { failure_class: FailureClass::RateLimit, max_attempts: 3,
            strategy: RetryStrategy::FixedDelay { delay_ms: 1000 }, jitter: false },
    ];
    let pid = engine.configure_policy("default", profiles, None).unwrap();
    let policy = engine.get_policy(&pid).unwrap();
    assert_eq!(policy.profiles.len(), 2);
}

#[test]
fn test_retry_profile_for_failure_class() {
    let mut engine = RetryEngine::new();
    let profiles = vec![
        RetryProfile { failure_class: FailureClass::Network, max_attempts: 3,
            strategy: RetryStrategy::Immediate, jitter: false },
    ];
    let pid = engine.configure_policy("net", profiles, None).unwrap();
    let profile = engine.get_profile_for_failure(&pid, &FailureClass::Network).unwrap();
    assert!(profile.is_some());
    let profile = engine.get_profile_for_failure(&pid, &FailureClass::Permanent).unwrap();
    assert!(profile.is_none());
}

#[test]
fn test_retry_budget_within() {
    let mut engine = RetryEngine::new();
    let budget = Some(RetryBudget { max_total_attempts: Some(3), max_total_time_ms: None, max_cost_units: None });
    let pid = engine.configure_policy("budgeted", vec![], budget).unwrap();
    assert!(engine.within_budget(&pid, "step-1").unwrap());
}

#[test]
fn test_retry_budget_exhausted() {
    let mut engine = RetryEngine::new();
    let budget = Some(RetryBudget { max_total_attempts: Some(2), max_total_time_ms: None, max_cost_units: None });
    let pid = engine.configure_policy("tight", vec![], budget).unwrap();
    engine.record_attempt("step-1", FailureClass::Transient);
    engine.record_attempt("step-1", FailureClass::Transient);
    assert!(!engine.within_budget(&pid, "step-1").unwrap());
}

#[test]
fn test_retry_record_and_stats() {
    let mut engine = RetryEngine::new();
    engine.record_attempt("step-a", FailureClass::Network);
    engine.record_attempt("step-a", FailureClass::Timeout);
    let stats = engine.get_stats("step-a").unwrap();
    assert_eq!(stats.total_attempts, 2);
    assert_eq!(stats.last_failure_class, Some(FailureClass::Timeout));
}

#[test]
fn test_retry_learned_patterns() {
    let mut engine = RetryEngine::new();
    for _ in 0..15 { engine.record_attempt("step-x", FailureClass::RateLimit); }
    let patterns = engine.get_patterns();
    assert_eq!(patterns.len(), 1);
    assert!(patterns[0].recommendation.contains("optimizing"));
}

// === Rollback Engine ===

#[test]
fn test_rollback_define_and_get() {
    let mut engine = RollbackEngine::new();
    engine.define_action(RollbackAction {
        id: "r1".into(), step_id: "step-1".into(),
        action_type: RollbackType::Command { command: "undo.sh".into(), args: vec![] },
        description: "Undo step 1".into(), verification: None,
    }).unwrap();
    assert!(engine.get_action("step-1").is_some());
    assert!(engine.get_action("step-2").is_none());
}

#[test]
fn test_rollback_preview_full_reverse_order() {
    let mut engine = RollbackEngine::new();
    for i in 1..=3 {
        engine.define_action(RollbackAction {
            id: format!("r{}", i), step_id: format!("step-{}", i),
            action_type: RollbackType::Command { command: "undo".into(), args: vec![] },
            description: "".into(), verification: None,
        }).unwrap();
    }
    let completed = vec!["step-1".into(), "step-2".into(), "step-3".into()];
    let preview = engine.preview(&RollbackScope::Full, &completed);
    assert_eq!(preview, vec!["step-3", "step-2", "step-1"]);
}

#[test]
fn test_rollback_preview_from_step() {
    let mut engine = RollbackEngine::new();
    for i in 1..=3 {
        engine.define_action(RollbackAction {
            id: format!("r{}", i), step_id: format!("step-{}", i),
            action_type: RollbackType::Command { command: "undo".into(), args: vec![] },
            description: "".into(), verification: None,
        }).unwrap();
    }
    let completed = vec!["step-1".into(), "step-2".into(), "step-3".into()];
    let preview = engine.preview(&RollbackScope::FromStep { step_id: "step-2".into() }, &completed);
    assert_eq!(preview, vec!["step-3", "step-2"]);
}

#[test]
fn test_rollback_preview_selective() {
    let mut engine = RollbackEngine::new();
    for i in 1..=3 {
        engine.define_action(RollbackAction {
            id: format!("r{}", i), step_id: format!("step-{}", i),
            action_type: RollbackType::Command { command: "undo".into(), args: vec![] },
            description: "".into(), verification: None,
        }).unwrap();
    }
    let preview = engine.preview(
        &RollbackScope::Selective { step_ids: vec!["step-1".into(), "step-3".into()] },
        &[],
    );
    assert_eq!(preview.len(), 2);
}

#[test]
fn test_rollback_execute_receipt() {
    let mut engine = RollbackEngine::new();
    engine.define_action(RollbackAction {
        id: "r1".into(), step_id: "step-1".into(),
        action_type: RollbackType::Command { command: "undo".into(), args: vec![] },
        description: "".into(), verification: None,
    }).unwrap();
    let receipt = engine.execute_rollback("exec-1", RollbackScope::Full, &["step-1".into()]).unwrap();
    assert!(receipt.overall_success);
    assert_eq!(receipt.rolled_back_steps.len(), 1);
}

#[test]
fn test_rollback_missing_action_partial_failure() {
    let mut engine = RollbackEngine::new();
    let receipt = engine.execute_rollback("exec-1", RollbackScope::Full, &["no-action".into()]).unwrap();
    assert!(!receipt.overall_success);
    assert!(receipt.rolled_back_steps[0].error.is_some());
}

// === Circuit Breaker Engine ===

#[test]
fn test_circuit_breaker_opens_after_threshold() {
    let mut engine = CircuitBreakerEngine::new();
    engine.get_or_create("api", 3, 2, 5000);
    assert!(engine.is_available("api"));
    engine.record_failure("api").unwrap();
    engine.record_failure("api").unwrap();
    assert!(engine.is_available("api"));
    engine.record_failure("api").unwrap();
    assert!(!engine.is_available("api"));
}

#[test]
fn test_circuit_breaker_reset() {
    let mut engine = CircuitBreakerEngine::new();
    engine.get_or_create("api", 1, 1, 1000);
    engine.record_failure("api").unwrap();
    assert!(!engine.is_available("api"));
    engine.reset("api").unwrap();
    assert!(engine.is_available("api"));
}

#[test]
fn test_circuit_breaker_unknown_service_available() {
    let engine = CircuitBreakerEngine::new();
    assert!(engine.is_available("never-registered"));
}

#[test]
fn test_circuit_breaker_preflight_all_healthy() {
    let mut engine = CircuitBreakerEngine::new();
    engine.get_or_create("svc-a", 5, 2, 5000);
    engine.get_or_create("svc-b", 5, 2, 5000);
    let result = engine.preflight_check("wf-1", &["svc-a".into(), "svc-b".into()]);
    assert!(result.all_services_healthy);
}

#[test]
fn test_circuit_breaker_preflight_some_down() {
    let mut engine = CircuitBreakerEngine::new();
    engine.get_or_create("healthy", 5, 2, 5000);
    engine.get_or_create("broken", 1, 2, 5000);
    engine.record_failure("broken").unwrap();
    let result = engine.preflight_check("wf-1", &["healthy".into(), "broken".into()]);
    assert!(!result.all_services_healthy);
}

#[test]
fn test_circuit_breaker_queue_workflow() {
    let mut engine = CircuitBreakerEngine::new();
    engine.queue_workflow("wf-1", "exec-1", "broken-svc", 1);
    assert_eq!(engine.queued_workflows().len(), 1);
}

// === Dead Letter Engine ===

#[test]
fn test_dead_letter_add_and_list() {
    let mut engine = DeadLetterEngine::new();
    engine.add_item("e1", "w1", "s1", "rate_limit", "429", serde_json::json!({}), 3).unwrap();
    assert_eq!(engine.list_items().len(), 1);
}

#[test]
fn test_dead_letter_summary_grouping() {
    let mut engine = DeadLetterEngine::new();
    engine.add_item("e1", "w1", "s1", "rate_limit", "429", serde_json::json!({}), 3).unwrap();
    engine.add_item("e2", "w1", "s2", "rate_limit", "429", serde_json::json!({}), 2).unwrap();
    engine.add_item("e3", "w1", "s3", "permanent", "bad data", serde_json::json!({}), 1).unwrap();
    let summary = engine.summary();
    assert_eq!(summary.total_items, 3);
    assert_eq!(summary.auto_retryable, 2);
    assert_eq!(summary.needs_human, 1);
}

#[test]
fn test_dead_letter_empty_summary() {
    let engine = DeadLetterEngine::new();
    let summary = engine.summary();
    assert_eq!(summary.total_items, 0);
}

#[test]
fn test_dead_letter_retryable_filter() {
    let mut engine = DeadLetterEngine::new();
    engine.add_item("e1", "w1", "s1", "network", "timeout", serde_json::json!({}), 1).unwrap();
    engine.add_item("e2", "w1", "s2", "permanent", "invalid", serde_json::json!({}), 1).unwrap();
    assert_eq!(engine.retryable_items("network").len(), 1);
    assert_eq!(engine.retryable_items("permanent").len(), 0);
}

// === Idempotency Engine ===

#[test]
fn test_idempotency_store_and_check() {
    let mut engine = IdempotencyEngine::new();
    let key = engine.compute_key("step-1", "wf-1", &serde_json::json!({"x": 1})).unwrap();
    assert!(engine.check(&key).is_none());
    engine.store(key.clone(), "step-1", "exec-1", "abc", serde_json::json!(42)).unwrap();
    assert!(engine.check(&key).is_some());
    assert_eq!(engine.check(&key).unwrap().output, serde_json::json!(42));
}

#[test]
fn test_idempotency_different_inputs_different_keys() {
    let engine = IdempotencyEngine::new();
    let k1 = engine.compute_key("step-1", "wf-1", &serde_json::json!({"x": 1})).unwrap();
    let k2 = engine.compute_key("step-1", "wf-1", &serde_json::json!({"x": 2})).unwrap();
    assert_ne!(k1, k2);
}

#[test]
fn test_idempotency_report() {
    let mut engine = IdempotencyEngine::new();
    engine.store("k1".into(), "step-1", "exec-1", "h1", serde_json::json!(1)).unwrap();
    engine.store("k2".into(), "step-1", "exec-2", "h2", serde_json::json!(2)).unwrap();
    engine.record_hit("step-1");
    let report = engine.report();
    assert_eq!(report.total_entries, 2);
    assert_eq!(report.deduplicated_count, 1);
}

#[test]
fn test_idempotency_empty_input() {
    let engine = IdempotencyEngine::new();
    let key = engine.compute_key("step-1", "wf-1", &serde_json::json!(null)).unwrap();
    assert!(!key.is_empty());
}

#[test]
fn test_idempotency_clear() {
    let mut engine = IdempotencyEngine::new();
    engine.store("k1".into(), "s1", "e1", "h1", serde_json::json!(1)).unwrap();
    assert_eq!(engine.report().total_entries, 1);
    engine.clear();
    assert_eq!(engine.report().total_entries, 0);
}
