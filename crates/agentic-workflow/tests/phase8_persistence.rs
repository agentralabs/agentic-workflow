//! Phase 8: Persistence tests — WorkflowStore, file roundtrip, auto-save.

use agentic_workflow::engine::store::WorkflowStore;
use agentic_workflow::engine::dag_exec::*;
use agentic_workflow::types::*;
use std::collections::HashMap;

// === WorkflowStore tests ===

#[test]
fn test_store_memory_insert_get() {
    let mut store = WorkflowStore::open_memory();
    let wf = Workflow::new("test", "A test");
    let id = wf.id.clone();
    store.insert(wf).unwrap();
    assert_eq!(store.get(&id).unwrap().name, "test");
    assert_eq!(store.count(), 1);
}

#[test]
fn test_store_memory_remove() {
    let mut store = WorkflowStore::open_memory();
    let wf = Workflow::new("rm", "");
    let id = wf.id.clone();
    store.insert(wf).unwrap();
    store.remove(&id).unwrap();
    assert_eq!(store.count(), 0);
    assert!(store.get(&id).is_err());
}

#[test]
fn test_store_memory_list() {
    let mut store = WorkflowStore::open_memory();
    store.insert(Workflow::new("a", "")).unwrap();
    store.insert(Workflow::new("b", "")).unwrap();
    store.insert(Workflow::new("c", "")).unwrap();
    assert_eq!(store.list().len(), 3);
}

#[test]
fn test_store_not_found() {
    let store = WorkflowStore::open_memory();
    assert!(store.get("ghost").is_err());
}

#[test]
fn test_store_dirty_tracking() {
    let mut store = WorkflowStore::open_memory();
    assert!(!store.is_dirty());
    store.insert(Workflow::new("x", "")).unwrap();
    // Memory store auto-save is disabled, so dirty stays true
    // (auto_save is false for memory stores)
}

#[test]
fn test_store_file_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.awf");

    let wf_id;
    {
        let mut store = WorkflowStore::open(&path).unwrap();
        store.set_auto_save(false);
        let mut wf = Workflow::new("persist", "Persisted");
        wf.add_step(StepNode::new("Step1", StepType::Noop));
        wf.add_step(StepNode::new("Step2", StepType::Noop));
        wf_id = wf.id.clone();
        store.insert(wf).unwrap();
        store.save().unwrap();
    }

    {
        let store = WorkflowStore::open(&path).unwrap();
        assert_eq!(store.count(), 1);
        let wf = store.get(&wf_id).unwrap();
        assert_eq!(wf.name, "persist");
        assert_eq!(wf.steps.len(), 2);
    }
}

#[test]
fn test_store_file_multiple_workflows() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("multi.awf");

    {
        let mut store = WorkflowStore::open(&path).unwrap();
        store.set_auto_save(false);
        for i in 0..10 {
            let wf = Workflow::new(format!("wf-{}", i), "");
            store.insert(wf).unwrap();
        }
        store.save().unwrap();
    }

    {
        let store = WorkflowStore::open(&path).unwrap();
        assert_eq!(store.count(), 10);
    }
}

#[test]
fn test_store_auto_save_on_drop() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("autosave.awf");

    {
        let mut store = WorkflowStore::open(&path).unwrap();
        store.insert(Workflow::new("dropped", "Auto-saved on drop")).unwrap();
    }

    let store = WorkflowStore::open(&path).unwrap();
    assert_eq!(store.count(), 1);
}

#[test]
fn test_store_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("overwrite.awf");

    {
        let mut store = WorkflowStore::open(&path).unwrap();
        store.set_auto_save(false);
        store.insert(Workflow::new("first", "")).unwrap();
        store.save().unwrap();
    }

    {
        let mut store = WorkflowStore::open(&path).unwrap();
        store.set_auto_save(false);
        store.insert(Workflow::new("second", "")).unwrap();
        store.save().unwrap();
    }

    let store = WorkflowStore::open(&path).unwrap();
    assert_eq!(store.count(), 2);
}

// === DAG Execution tests ===

#[test]
fn test_exec_noop_step() {
    let result = execute_step("s1", &StepType::Noop, &HashMap::new(), None);
    assert!(result.success);
    assert!(result.output.is_some());
    assert_eq!(result.output.unwrap()["status"], "noop");
}

#[test]
fn test_exec_expression_step() {
    let result = execute_step("s1", &StepType::Expression { expression: "x+1".into() }, &HashMap::new(), None);
    assert!(result.success);
    assert_eq!(result.output.unwrap()["evaluated"], true);
}

#[test]
fn test_exec_command_step_prepared() {
    let result = execute_step("s1", &StepType::Command { command: "ls".into(), args: vec!["-la".into()] }, &HashMap::new(), None);
    assert!(result.success);
    assert_eq!(result.output.unwrap()["status"], "prepared");
}

#[test]
fn test_exec_mcp_tool_step() {
    let result = execute_step("s1", &StepType::McpTool {
        sister: "memory".into(), tool: "memory_add".into(), params: serde_json::json!({}),
    }, &HashMap::new(), None);
    assert!(result.success);
    assert_eq!(result.output.unwrap()["sister"], "memory");
}

#[test]
fn test_exec_http_step() {
    let result = execute_step("s1", &StepType::HttpRequest {
        method: "GET".into(), url: "https://api.example.com".into(),
        headers: HashMap::new(), body: None,
    }, &HashMap::new(), None);
    assert!(result.success);
    assert_eq!(result.output.unwrap()["method"], "GET");
}

#[test]
fn test_exec_approval_gate_step() {
    let result = execute_step("s1", &StepType::ApprovalGate {
        approvers: vec!["alice".into()], timeout_ms: Some(60000),
    }, &HashMap::new(), None);
    assert!(result.success);
    assert_eq!(result.output.unwrap()["status"], "waiting_approval");
}

#[test]
fn test_exec_apply_result_success() {
    let mut ctx = make_test_ctx(vec!["s1"]);
    let result = StepExecutionResult {
        step_id: "s1".into(), success: true,
        output: Some(serde_json::json!(42)), error: None, duration_ms: 100,
    };
    apply_step_result(&mut ctx, &result);
    assert_eq!(ctx.step_states["s1"].lifecycle, StepLifecycle::Success);
    assert_eq!(ctx.step_states["s1"].duration_ms, Some(100));
}

#[test]
fn test_exec_apply_result_failure() {
    let mut ctx = make_test_ctx(vec!["s1"]);
    let result = StepExecutionResult {
        step_id: "s1".into(), success: false,
        output: None, error: Some("boom".into()), duration_ms: 50,
    };
    apply_step_result(&mut ctx, &result);
    assert_eq!(ctx.step_states["s1"].lifecycle, StepLifecycle::Failed);
    assert_eq!(ctx.step_states["s1"].error.as_deref(), Some("boom"));
}

#[test]
fn test_exec_next_ready_no_deps() {
    let mut wf = Workflow::new("nodeps", "");
    let s1 = StepNode::new("A", StepType::Noop);
    let s2 = StepNode::new("B", StepType::Noop);
    let id1 = s1.id.clone();
    let id2 = s2.id.clone();
    wf.add_step(s1); wf.add_step(s2);
    // No edges — both ready immediately
    let ctx = make_test_ctx_for(&wf);
    let ready = next_ready_steps(&wf, &ctx);
    assert_eq!(ready.len(), 2);
}

#[test]
fn test_exec_is_complete_all_success() {
    let mut ctx = make_test_ctx(vec!["s1", "s2"]);
    ctx.step_states.get_mut("s1").unwrap().lifecycle = StepLifecycle::Success;
    ctx.step_states.get_mut("s2").unwrap().lifecycle = StepLifecycle::Success;
    assert!(is_execution_complete(&ctx));
}

#[test]
fn test_exec_is_complete_partial() {
    let mut ctx = make_test_ctx(vec!["s1", "s2"]);
    ctx.step_states.get_mut("s1").unwrap().lifecycle = StepLifecycle::Success;
    assert!(!is_execution_complete(&ctx));
}

#[test]
fn test_exec_compute_status_succeeded() {
    let mut ctx = make_test_ctx(vec!["s1"]);
    ctx.step_states.get_mut("s1").unwrap().lifecycle = StepLifecycle::Success;
    assert!(matches!(compute_execution_status(&ctx), ExecutionStatus::Succeeded));
}

#[test]
fn test_exec_compute_status_failed() {
    let mut ctx = make_test_ctx(vec!["s1"]);
    ctx.step_states.get_mut("s1").unwrap().lifecycle = StepLifecycle::Failed;
    assert!(matches!(compute_execution_status(&ctx), ExecutionStatus::Failed { .. }));
}

#[test]
fn test_exec_output_propagation() {
    let mut wf = Workflow::new("prop", "");
    let s1 = StepNode::new("A", StepType::Noop);
    let s2 = StepNode::new("B", StepType::Noop);
    let (id1, id2) = (s1.id.clone(), s2.id.clone());
    wf.add_step(s1); wf.add_step(s2);
    wf.add_edge(Edge { from: id1.clone(), to: id2.clone(), edge_type: EdgeType::Sequence });

    let mut ctx = make_test_ctx_for(&wf);
    ctx.step_states.get_mut(&id1).unwrap().lifecycle = StepLifecycle::Success;
    ctx.step_states.get_mut(&id1).unwrap().output = Some(serde_json::json!({"data": "hello"}));

    let inputs = propagate_outputs(&wf, &ctx, &id2);
    assert_eq!(inputs[&id1], serde_json::json!({"data": "hello"}));
}

#[test]
fn test_exec_build_fingerprint() {
    let mut ctx = make_test_ctx(vec!["s1", "s2"]);
    ctx.step_states.get_mut("s1").unwrap().lifecycle = StepLifecycle::Success;
    ctx.step_states.get_mut("s1").unwrap().duration_ms = Some(100);
    ctx.step_states.get_mut("s1").unwrap().attempt = 2;
    ctx.step_states.get_mut("s2").unwrap().lifecycle = StepLifecycle::Success;
    ctx.step_states.get_mut("s2").unwrap().duration_ms = Some(200);
    ctx.completed_at = Some(chrono::Utc::now());

    let fp = build_fingerprint(&ctx);
    assert_eq!(fp.total_duration_ms, 300);
    assert_eq!(fp.retry_count, 1);
    assert_eq!(fp.step_durations.len(), 2);
}

// === Helpers ===

fn make_test_ctx(step_ids: Vec<&str>) -> ExecutionContext {
    let mut step_states = HashMap::new();
    for id in step_ids {
        step_states.insert(id.to_string(), StepState {
            step_id: id.to_string(), lifecycle: StepLifecycle::Pending, attempt: 0,
            started_at: None, completed_at: None, duration_ms: None, output: None, error: None,
        });
    }
    ExecutionContext {
        execution_id: "test-exec".into(), workflow_id: "test-wf".into(),
        status: ExecutionStatus::Running, step_states,
        variables: HashMap::new(), started_at: chrono::Utc::now(),
        completed_at: None, trigger_info: None, metadata: HashMap::new(),
    }
}

fn make_test_ctx_for(wf: &Workflow) -> ExecutionContext {
    let mut step_states = HashMap::new();
    for step in &wf.steps {
        step_states.insert(step.id.clone(), StepState {
            step_id: step.id.clone(), lifecycle: StepLifecycle::Pending, attempt: 0,
            started_at: None, completed_at: None, duration_ms: None, output: None, error: None,
        });
    }
    ExecutionContext {
        execution_id: "test-exec".into(), workflow_id: wf.id.clone(),
        status: ExecutionStatus::Running, step_states,
        variables: HashMap::new(), started_at: chrono::Utc::now(),
        completed_at: None, trigger_info: None, metadata: HashMap::new(),
    }
}
