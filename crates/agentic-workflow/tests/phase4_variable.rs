//! Phase 4b: Variable scoping tests — hierarchy, cascade, promotion, type checking, immutability.

use agentic_workflow::governance::VariableEngine;
use agentic_workflow::types::*;

#[test]
fn variable_create_scope_hierarchy() {
    let mut engine = VariableEngine::new();
    let wf = engine.create_scope(ScopeType::Workflow, None);
    let step = engine.create_scope(ScopeType::Step, Some(&wf));
    assert!(!wf.is_empty());
    assert!(!step.is_empty());
    assert_ne!(wf, step);
}

#[test]
fn variable_set_and_get() {
    let mut engine = VariableEngine::new();
    let sid = engine.create_scope(ScopeType::Workflow, None);
    engine.set(&sid, "env", serde_json::json!("staging"), VariableType::String, "deploy").unwrap();
    let var = engine.get(&sid, "env").unwrap();
    assert_eq!(var.value, serde_json::json!("staging"));
    assert_eq!(var.set_by, "deploy");
}

#[test]
fn variable_scope_cascade_child_reads_parent() {
    let mut engine = VariableEngine::new();
    let parent = engine.create_scope(ScopeType::Workflow, None);
    let child = engine.create_scope(ScopeType::Step, Some(&parent));
    engine.set(&parent, "region", serde_json::json!("us-east"), VariableType::String, "sys").unwrap();
    let var = engine.get(&child, "region").unwrap();
    assert_eq!(var.value, serde_json::json!("us-east"));
}

#[test]
fn variable_child_overrides_parent() {
    let mut engine = VariableEngine::new();
    let parent = engine.create_scope(ScopeType::Workflow, None);
    let child = engine.create_scope(ScopeType::Step, Some(&parent));
    engine.set(&parent, "env", serde_json::json!("prod"), VariableType::String, "sys").unwrap();
    engine.set(&child, "env", serde_json::json!("staging"), VariableType::String, "sys").unwrap();
    assert_eq!(engine.get(&child, "env").unwrap().value, serde_json::json!("staging"));
    assert_eq!(engine.get(&parent, "env").unwrap().value, serde_json::json!("prod"));
}

#[test]
fn variable_promote_from_child_to_parent() {
    let mut engine = VariableEngine::new();
    let parent = engine.create_scope(ScopeType::Workflow, None);
    let child = engine.create_scope(ScopeType::Step, Some(&parent));
    engine.set(&child, "result", serde_json::json!(42), VariableType::Integer, "step").unwrap();
    assert!(engine.get(&parent, "result").is_err());
    engine.promote(&child, "result").unwrap();
    assert_eq!(engine.get(&parent, "result").unwrap().value, serde_json::json!(42));
}

#[test]
fn variable_type_checking_match() {
    let mut engine = VariableEngine::new();
    let sid = engine.create_scope(ScopeType::Workflow, None);
    engine.set(&sid, "count", serde_json::json!(10), VariableType::Integer, "sys").unwrap();
    assert_eq!(engine.get(&sid, "count").unwrap().value, serde_json::json!(10));
}

#[test]
fn variable_type_checking_mismatch() {
    let mut engine = VariableEngine::new();
    let sid = engine.create_scope(ScopeType::Workflow, None);
    assert!(engine.set(&sid, "count", serde_json::json!("not_a_number"), VariableType::Integer, "sys").is_err());
}

#[test]
fn variable_immutability_enforcement() {
    let mut engine = VariableEngine::new();
    let sid = engine.create_scope(ScopeType::Workflow, None);
    engine.set(&sid, "locked", serde_json::json!("initial"), VariableType::String, "sys").unwrap();
    engine.make_immutable(&sid, "locked").unwrap();
    assert!(engine.set(&sid, "locked", serde_json::json!("changed"), VariableType::String, "sys").is_err());
    assert_eq!(engine.get(&sid, "locked").unwrap().value, serde_json::json!("initial"));
}

#[test]
fn variable_type_check_across_scopes() {
    let mut engine = VariableEngine::new();
    let s1 = engine.create_scope(ScopeType::Workflow, None);
    let s2 = engine.create_scope(ScopeType::Step, Some(&s1));
    engine.set(&s1, "a", serde_json::json!("hello"), VariableType::String, "sys").unwrap();
    engine.set(&s2, "b", serde_json::json!(true), VariableType::Boolean, "sys").unwrap();
    let result = engine.type_check();
    assert!(result.valid);
}

#[test]
fn variable_type_matches_all_types() {
    assert!(VariableType::String.matches(&serde_json::json!("hello")));
    assert!(VariableType::Integer.matches(&serde_json::json!(42)));
    assert!(VariableType::Float.matches(&serde_json::json!(3.14)));
    assert!(VariableType::Boolean.matches(&serde_json::json!(true)));
    assert!(VariableType::Array.matches(&serde_json::json!([1, 2])));
    assert!(VariableType::Object.matches(&serde_json::json!({"k": "v"})));
    assert!(VariableType::Null.matches(&serde_json::Value::Null));
    assert!(VariableType::Any.matches(&serde_json::json!("anything")));
    // Mismatches
    assert!(!VariableType::String.matches(&serde_json::json!(42)));
    assert!(!VariableType::Integer.matches(&serde_json::json!("42")));
    assert!(!VariableType::Boolean.matches(&serde_json::json!(1)));
    assert!(!VariableType::Array.matches(&serde_json::json!({"a": 1})));
    assert!(!VariableType::Object.matches(&serde_json::json!([1])));
    assert!(!VariableType::Null.matches(&serde_json::json!(0)));
}
