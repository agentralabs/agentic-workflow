//! Phase 1: Type system tests — workflow structures, enums, serde, strategies.
//! ~40 tests covering every type variant and serialization roundtrip.

use agentic_workflow::types::*;
use agentic_workflow::types::workflow::*;
use agentic_workflow::types::execution::*;
use agentic_workflow::types::schedule::*;
use agentic_workflow::types::trigger::*;
use agentic_workflow::types::retry::*;
use agentic_workflow::types::batch::*;
use agentic_workflow::types::stream::*;
use agentic_workflow::types::fanout::*;
use agentic_workflow::types::fsm::*;
use agentic_workflow::types::variable::*;
use agentic_workflow::types::approval::*;
use agentic_workflow::types::audit::*;
use agentic_workflow::types::circuit::*;
use agentic_workflow::types::rollback::*;
use agentic_workflow::types::idempotency::*;
use agentic_workflow::types::template::*;
use std::collections::HashMap;

// ── Workflow creation and modification ──

#[test]
fn test_workflow_creation() {
    let wf = Workflow::new("deploy-pipeline", "Deploys to production");
    assert_eq!(wf.name, "deploy-pipeline");
    assert_eq!(wf.description, "Deploys to production");
    assert_eq!(wf.version, 1);
    assert!(wf.steps.is_empty());
    assert!(wf.edges.is_empty());
    assert!(!wf.id.is_empty());
}

#[test]
fn test_workflow_add_step() {
    let mut wf = Workflow::new("wf", "test");
    let step = StepNode::new("Build", StepType::Noop);
    let sid = step.id.clone();
    wf.add_step(step);
    assert_eq!(wf.steps.len(), 1);
    assert!(wf.step_by_id(&sid).is_some());
}

#[test]
fn test_workflow_add_edge() {
    let mut wf = Workflow::new("wf", "test");
    let s1 = StepNode::new("A", StepType::Noop);
    let s2 = StepNode::new("B", StepType::Noop);
    let edge = Edge {
        from: s1.id.clone(),
        to: s2.id.clone(),
        edge_type: EdgeType::Sequence,
    };
    wf.add_step(s1);
    wf.add_step(s2);
    wf.add_edge(edge);
    assert_eq!(wf.edges.len(), 1);
}

#[test]
fn test_workflow_remove_step_cascades_edges() {
    let mut wf = Workflow::new("wf", "test");
    let s1 = StepNode::new("A", StepType::Noop);
    let s2 = StepNode::new("B", StepType::Noop);
    let s1_id = s1.id.clone();
    let s2_id = s2.id.clone();
    wf.add_step(s1);
    wf.add_step(s2);
    wf.add_edge(Edge { from: s1_id.clone(), to: s2_id, edge_type: EdgeType::Sequence });
    wf.remove_step(&s1_id);
    assert_eq!(wf.steps.len(), 1);
    assert!(wf.edges.is_empty(), "Edges referencing removed step must be removed");
}

#[test]
fn test_workflow_step_ids() {
    let mut wf = Workflow::new("wf", "test");
    let s1 = StepNode::new("A", StepType::Noop);
    let s2 = StepNode::new("B", StepType::Noop);
    let id1 = s1.id.clone();
    let id2 = s2.id.clone();
    wf.add_step(s1);
    wf.add_step(s2);
    let ids = wf.step_ids();
    assert!(ids.contains(&id1.as_str()));
    assert!(ids.contains(&id2.as_str()));
}

// ── StepType variants ──

#[test]
fn test_step_type_command() {
    let step = StepNode::new("Run", StepType::Command {
        command: "cargo".into(), args: vec!["build".into()],
    });
    matches!(step.step_type, StepType::Command { .. });
}

#[test]
fn test_step_type_mcp_tool() {
    let step = StepNode::new("MCP", StepType::McpTool {
        sister: "codebase".into(), tool: "search".into(),
        params: serde_json::json!({"query": "main"}),
    });
    matches!(step.step_type, StepType::McpTool { .. });
}

#[test]
fn test_step_type_http_request() {
    let step = StepNode::new("HTTP", StepType::HttpRequest {
        method: "POST".into(), url: "https://api.example.com".into(),
        headers: HashMap::new(), body: Some("{}".into()),
    });
    matches!(step.step_type, StepType::HttpRequest { .. });
}

#[test]
fn test_step_type_sub_workflow() {
    let step = StepNode::new("Sub", StepType::SubWorkflow { workflow_id: "wf-2".into() });
    matches!(step.step_type, StepType::SubWorkflow { .. });
}

#[test]
fn test_step_type_fan_out() {
    let step = StepNode::new("Fan", StepType::FanOut {
        destinations: vec!["d1".into()], completion_policy: CompletionPolicy::WaitAll,
    });
    matches!(step.step_type, StepType::FanOut { .. });
}

#[test]
fn test_step_type_approval_gate() {
    let step = StepNode::new("Approve", StepType::ApprovalGate {
        approvers: vec!["admin@co.com".into()], timeout_ms: Some(3600_000),
    });
    matches!(step.step_type, StepType::ApprovalGate { .. });
}

#[test]
fn test_step_type_expression() {
    let step = StepNode::new("Expr", StepType::Expression {
        expression: "result = input.x + 1".into(),
    });
    matches!(step.step_type, StepType::Expression { .. });
}

#[test]
fn test_step_type_noop() {
    let step = StepNode::new("Noop", StepType::Noop);
    matches!(step.step_type, StepType::Noop);
}

// ── EdgeType variants ──

#[test]
fn test_edge_type_sequence() {
    let e = EdgeType::Sequence;
    assert!(matches!(e, EdgeType::Sequence));
}

#[test]
fn test_edge_type_parallel() {
    let e = EdgeType::Parallel;
    assert!(matches!(e, EdgeType::Parallel));
}

#[test]
fn test_edge_type_conditional() {
    let e = EdgeType::Conditional { expression: "status == 200".into() };
    assert!(matches!(e, EdgeType::Conditional { .. }));
}

#[test]
fn test_edge_type_loop() {
    let e = EdgeType::Loop { max_iterations: Some(10), condition: Some("retry".into()) };
    assert!(matches!(e, EdgeType::Loop { .. }));
}

// ── ExecutionStatus transitions ──

#[test]
fn test_execution_status_eq() {
    assert_eq!(ExecutionStatus::Pending, ExecutionStatus::Pending);
    assert_eq!(ExecutionStatus::Running, ExecutionStatus::Running);
    assert_ne!(ExecutionStatus::Running, ExecutionStatus::Paused);
    let f = ExecutionStatus::Failed { error: "boom".into() };
    assert!(matches!(f, ExecutionStatus::Failed { .. }));
}

// ── StepLifecycle states ──

#[test]
fn test_step_lifecycle_all_variants() {
    let variants = vec![
        StepLifecycle::Pending, StepLifecycle::Queued, StepLifecycle::Running,
        StepLifecycle::Success, StepLifecycle::Failed, StepLifecycle::Skipped,
        StepLifecycle::Cancelled, StepLifecycle::WaitingApproval, StepLifecycle::RolledBack,
    ];
    assert_eq!(variants.len(), 9);
    assert_ne!(StepLifecycle::Pending, StepLifecycle::Running);
}

// ── TriggerType variants ──

#[test]
fn test_trigger_type_filesystem() {
    let t = TriggerType::FileSystem {
        path: "/data".into(), events: vec![FileEvent::Created, FileEvent::Modified],
    };
    assert!(matches!(t, TriggerType::FileSystem { .. }));
}

#[test]
fn test_trigger_type_webhook() {
    let t = TriggerType::Webhook { endpoint: "/hook".into(), method: "POST".into() };
    assert!(matches!(t, TriggerType::Webhook { .. }));
}

#[test]
fn test_trigger_type_schedule() {
    let t = TriggerType::Schedule { schedule_id: "s1".into() };
    assert!(matches!(t, TriggerType::Schedule { .. }));
}

#[test]
fn test_trigger_type_manual() {
    assert!(matches!(TriggerType::Manual, TriggerType::Manual));
}

#[test]
fn test_trigger_type_workflow_output() {
    let t = TriggerType::WorkflowOutput { source_workflow_id: "wf-1".into() };
    assert!(matches!(t, TriggerType::WorkflowOutput { .. }));
}

#[test]
fn test_trigger_type_api_callback() {
    let t = TriggerType::ApiCallback { callback_id: "cb-1".into() };
    assert!(matches!(t, TriggerType::ApiCallback { .. }));
}

#[test]
fn test_trigger_type_custom() {
    let t = TriggerType::Custom {
        event_type: "git-push".into(), config: serde_json::json!({}),
    };
    assert!(matches!(t, TriggerType::Custom { .. }));
}

// ── ScheduleExpression variants ──

#[test]
fn test_schedule_expression_all_variants() {
    let _cron = ScheduleExpression::Cron("0 8 * * 1-5".into());
    let _nat = ScheduleExpression::Natural("every weekday at 8am".into());
    let _int = ScheduleExpression::Interval { every_ms: 60_000 };
    let _once = ScheduleExpression::Once { at: chrono::Utc::now() };
}

// ── ConflictPolicy variants ──

#[test]
fn test_conflict_policy_all_variants() {
    let _skip = ConflictPolicy::Skip;
    let _queue = ConflictPolicy::Queue;
    let _cancel = ConflictPolicy::CancelPrevious;
    let _wait = ConflictPolicy::Wait;
}

// ── FailureClass variants ──

#[test]
fn test_failure_class_all_variants() {
    let classes = vec![
        FailureClass::Transient, FailureClass::Permanent, FailureClass::RateLimit,
        FailureClass::ResourceContention, FailureClass::Authentication,
        FailureClass::Network, FailureClass::Timeout, FailureClass::Unknown,
    ];
    assert_eq!(classes.len(), 8);
    assert_eq!(FailureClass::Transient, FailureClass::Transient);
    assert_ne!(FailureClass::Transient, FailureClass::Permanent);
}

// ── CircuitState transitions ──

#[test]
fn test_circuit_state_transitions() {
    assert_eq!(CircuitState::Closed, CircuitState::Closed);
    assert_ne!(CircuitState::Closed, CircuitState::Open);
    assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
}

// ── BatchStatus / BatchItemStatus ──

#[test]
fn test_batch_status_variants() {
    let statuses = vec![
        BatchStatus::Pending, BatchStatus::Running, BatchStatus::Paused,
        BatchStatus::Completed, BatchStatus::PartiallyCompleted,
        BatchStatus::Failed, BatchStatus::Cancelled,
    ];
    assert_eq!(statuses.len(), 7);
}

#[test]
fn test_batch_item_status_variants() {
    let statuses = vec![
        BatchItemStatus::Pending, BatchItemStatus::Running,
        BatchItemStatus::Success, BatchItemStatus::Failed, BatchItemStatus::Skipped,
    ];
    assert_eq!(statuses.len(), 5);
}

// ── StreamStatus transitions ──

#[test]
fn test_stream_status_transitions() {
    let chain = vec![
        StreamStatus::Created, StreamStatus::Running,
        StreamStatus::Paused, StreamStatus::Stopped, StreamStatus::Error,
    ];
    assert_eq!(chain.len(), 5);
    assert_ne!(StreamStatus::Created, StreamStatus::Running);
}

// ── FanOutBranchStatus ──

#[test]
fn test_fanout_branch_status_variants() {
    let statuses = vec![
        FanOutBranchStatus::Pending, FanOutBranchStatus::Running,
        FanOutBranchStatus::Success, FanOutBranchStatus::Failed,
        FanOutBranchStatus::TimedOut, FanOutBranchStatus::Cancelled,
    ];
    assert_eq!(statuses.len(), 6);
}

// ── FSM State + Transition creation ──

#[test]
fn test_fsm_state_creation() {
    let state = State {
        name: "Idle".into(), description: Some("Waiting".into()),
        entry_action: None, exit_action: None, is_terminal: false,
    };
    assert_eq!(state.name, "Idle");
    assert!(!state.is_terminal);
}

#[test]
fn test_fsm_transition_creation() {
    let t = Transition {
        from: "A".into(), to: "B".into(), event: "go".into(),
        guard: None, action: None,
    };
    assert_eq!(t.from, "A");
    assert_eq!(t.event, "go");
}

// Remaining type tests are in phase1_types_serde.rs
