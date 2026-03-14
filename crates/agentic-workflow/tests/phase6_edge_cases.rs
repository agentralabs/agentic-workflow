//! Phase 6: Edge cases — boundary values, error paths, format corruption.

use std::io::Cursor;
use agentic_workflow::types::*;
use agentic_workflow::engine::*;
use agentic_workflow::resilience::*;
use agentic_workflow::governance::*;
use agentic_workflow::format::{AwfWriter, AwfReader};

// === DAG edge cases ===

#[test]
fn test_edge_empty_workflow_validates() {
    let engine = DagEngine::new();
    assert!(engine.validate_dag(&Workflow::new("empty", "")).is_ok());
}

#[test]
fn test_edge_single_step_no_edges() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("single", "");
    wf.add_step(StepNode::new("Only", StepType::Noop));
    assert!(engine.validate_dag(&wf).is_ok());
}

#[test]
fn test_edge_to_nonexistent_step() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("bad", "");
    wf.add_step(StepNode::new("A", StepType::Noop));
    wf.add_edge(Edge { from: "nonexistent".into(), to: "also-missing".into(), edge_type: EdgeType::Sequence });
    assert!(engine.validate_dag(&wf).is_err());
}

#[test]
fn test_edge_self_loop() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("loop", "");
    let s = StepNode::new("A", StepType::Noop);
    let sid = s.id.clone();
    wf.add_step(s);
    wf.add_edge(Edge { from: sid.clone(), to: sid, edge_type: EdgeType::Sequence });
    assert!(engine.validate_dag(&wf).is_err());
}

#[test]
fn test_edge_workflow_100_steps() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("large", "");
    let mut prev_id = String::new();
    for i in 0..100 {
        let s = StepNode::new(format!("Step-{}", i), StepType::Noop);
        let sid = s.id.clone();
        wf.add_step(s);
        if i > 0 {
            wf.add_edge(Edge { from: prev_id.clone(), to: sid.clone(), edge_type: EdgeType::Sequence });
        }
        prev_id = sid;
    }
    assert!(engine.validate_dag(&wf).is_ok());
}

// === Execution edge cases ===

#[test]
fn test_edge_execution_double_pause() {
    let mut engine = DagEngine::new();
    let wf = Workflow::new("dp", "");
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let eid = engine.start_execution(&wfid).unwrap();
    engine.pause_execution(&eid).unwrap();
    assert!(engine.pause_execution(&eid).is_err());
}

#[test]
fn test_edge_execution_resume_not_paused() {
    let mut engine = DagEngine::new();
    let wf = Workflow::new("rnp", "");
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let eid = engine.start_execution(&wfid).unwrap();
    assert!(engine.resume_execution(&eid).is_err());
}

#[test]
fn test_edge_execution_nonexistent() {
    let engine = DagEngine::new();
    assert!(engine.get_execution("ghost").is_err());
    assert!(engine.get_progress("ghost").is_err());
}

#[test]
fn test_edge_start_nonexistent_workflow() {
    let mut engine = DagEngine::new();
    assert!(engine.start_execution("no-such-wf").is_err());
}

// === Batch edge cases ===

#[test]
fn test_edge_batch_zero_items() {
    let mut engine = BatchEngine::new();
    let bid = engine.create_batch("wf", vec![], 1, 1).unwrap();
    let p = engine.get_progress(&bid).unwrap();
    assert_eq!(p.total_items, 0);
}

#[test]
fn test_edge_batch_concurrency_clamped() {
    let mut engine = BatchEngine::new();
    let bid = engine.create_batch("wf", vec![serde_json::json!(1)], 0, 0).unwrap();
    let job = engine.get_job(&bid).unwrap();
    assert_eq!(job.concurrency, 1); // clamped to 1
    assert_eq!(job.checkpoint_every, 1);
}

// === FanOut edge cases ===

#[test]
fn test_edge_fanout_all_branches_fail() {
    let mut engine = FanOutEngine::new();
    let dests = vec![
        FanOutDestination { id: "d1".into(), name: "A".into(), step_config: serde_json::json!({}) },
        FanOutDestination { id: "d2".into(), name: "B".into(), step_config: serde_json::json!({}) },
    ];
    let fid = engine.create_fanout(dests, CompletionPolicy::WaitAll, ResultAggregator::Merge, None).unwrap();
    engine.start_execution(&fid, "exec-fail").unwrap();
    engine.update_branch("exec-fail", "d1", FanOutBranchStatus::Failed, None, Some("err".into()), None).unwrap();
    engine.update_branch("exec-fail", "d2", FanOutBranchStatus::TimedOut, None, None, None).unwrap();
    assert!(engine.get_status("exec-fail").unwrap().completed);
}

// === FSM edge cases ===

#[test]
fn test_edge_fsm_invalid_initial_state() {
    let mut engine = FsmEngine::new();
    let states = vec![State { name: "A".into(), description: None, entry_action: None, exit_action: None, is_terminal: false }];
    assert!(engine.create_fsm("bad", states, vec![], "Nonexistent").is_err());
}

#[test]
fn test_edge_fsm_no_valid_transitions() {
    let mut engine = FsmEngine::new();
    let states = vec![
        State { name: "Terminal".into(), description: None, entry_action: None, exit_action: None, is_terminal: true },
    ];
    let fid = engine.create_fsm("terminal", states, vec![], "Terminal").unwrap();
    let valid = engine.valid_next(&fid).unwrap();
    assert!(valid.is_empty());
}

// === Variable edge cases ===

#[test]
fn test_edge_variable_wrong_type() {
    let mut engine = VariableEngine::new();
    let sid = engine.create_scope(ScopeType::Workflow, None);
    let result = engine.set(&sid, "x", serde_json::json!(42), VariableType::String, "test");
    assert!(result.is_err());
}

#[test]
fn test_edge_variable_not_found() {
    let engine = VariableEngine::new();
    assert!(engine.get("no-scope", "no-var").is_err());
}

#[test]
fn test_edge_variable_immutable_reject() {
    let mut engine = VariableEngine::new();
    let sid = engine.create_scope(ScopeType::Workflow, None);
    engine.set(&sid, "frozen", serde_json::json!(1), VariableType::Integer, "test").unwrap();
    engine.make_immutable(&sid, "frozen").unwrap();
    assert!(engine.set(&sid, "frozen", serde_json::json!(2), VariableType::Integer, "test").is_err());
}

// === Approval edge cases ===

#[test]
fn test_edge_approve_nonexistent_pending() {
    let mut engine = ApprovalEngine::new();
    assert!(engine.decide("ghost", ApprovalDecision::Approved, "alice", None).is_err());
}

// === Audit edge cases ===

#[test]
fn test_edge_audit_query_no_results() {
    let engine = AuditEngine::new();
    let q = AuditQuery {
        workflow_id: Some("nonexistent".into()), execution_id: None,
        event_types: None, actor: None, resource: None, from: None, to: None, limit: None,
    };
    assert!(engine.query(&q).is_empty());
}

// === Circuit breaker edge cases ===

#[test]
fn test_edge_circuit_unknown_service() {
    let engine = CircuitBreakerEngine::new();
    assert!(engine.is_available("never-registered"));
}

// === Dead letter edge cases ===

#[test]
fn test_edge_dead_letter_remove_nonexistent() {
    let mut engine = DeadLetterEngine::new();
    assert!(engine.remove_item("ghost").is_err());
}

// === .awf format edge cases ===

#[test]
fn test_edge_awf_invalid_magic() {
    let data = b"BADMxxxxxxxxxxxxxx";
    let cursor = Cursor::new(data.to_vec());
    let mut reader = AwfReader::new(cursor);
    assert!(reader.read_header().is_err());
}

#[test]
fn test_edge_awf_future_version() {
    let mut data = Vec::new();
    data.extend_from_slice(b"AWFL");
    data.extend_from_slice(&999u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());
    let cursor = Cursor::new(data);
    let mut reader = AwfReader::new(cursor);
    assert!(reader.read_header().is_err());
}

#[test]
fn test_edge_awf_corrupt_checksum() {
    let mut buf = Vec::new();
    {
        let mut writer = AwfWriter::new(&mut buf);
        writer.write_header().unwrap();
        let wf = Workflow::new("test", "");
        writer.write_workflow(&wf).unwrap();
        writer.finish().unwrap();
    }
    // Corrupt the last byte of the checksum
    if let Some(last) = buf.last_mut() { *last ^= 0xFF; }
    let cursor = Cursor::new(buf);
    let mut reader = AwfReader::new(cursor);
    reader.read_header().unwrap();
    assert!(reader.read_workflow().is_err());
}

#[test]
fn test_edge_execution_serialization_roundtrip() {
    let ctx = ExecutionContext {
        execution_id: "e1".into(), workflow_id: "wf1".into(),
        status: ExecutionStatus::Running,
        step_states: std::collections::HashMap::new(),
        variables: std::collections::HashMap::new(),
        started_at: chrono::Utc::now(), completed_at: None,
        trigger_info: None, metadata: std::collections::HashMap::new(),
    };
    let json = serde_json::to_string(&ctx).unwrap();
    let restored: ExecutionContext = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.execution_id, "e1");
}

// === Rollback edge cases ===

#[test]
fn test_edge_rollback_empty_steps() {
    let mut engine = RollbackEngine::new();
    let receipt = engine.execute_rollback("exec-1", RollbackScope::Full, &[]).unwrap();
    assert!(receipt.overall_success);
    assert!(receipt.rolled_back_steps.is_empty());
}

// === Template edge cases ===

#[test]
fn test_edge_template_unknown_id() {
    let mut engine = agentic_workflow::template::TemplateEngine::new();
    assert!(engine.instantiate("nonexistent", &std::collections::HashMap::new()).is_err());
}

// === Prediction edge cases ===

#[test]
fn test_edge_prediction_no_history() {
    let engine = agentic_workflow::intelligence::PredictionEngine::new();
    let pred = engine.predict_duration("unknown").unwrap();
    assert_eq!(pred.based_on_executions, 0);
    assert_eq!(pred.confidence, 0.0);
}
