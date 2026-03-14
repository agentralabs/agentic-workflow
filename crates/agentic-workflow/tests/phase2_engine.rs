use agentic_workflow::types::*;
use agentic_workflow::engine::*;

// === DAG Engine Tests ===

#[test]
fn test_dag_create_and_validate() {
    let mut engine = DagEngine::new();
    let mut wf = Workflow::new("pipeline", "CI/CD");
    let s1 = StepNode::new("Build", StepType::Noop);
    let s2 = StepNode::new("Test", StepType::Noop);
    let (id1, id2) = (s1.id.clone(), s2.id.clone());
    wf.add_step(s1);
    wf.add_step(s2);
    wf.add_edge(Edge { from: id1, to: id2, edge_type: EdgeType::Sequence });
    assert!(engine.register_workflow(wf).is_ok());
}

#[test]
fn test_dag_cycle_detection() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("cyclic", "");
    let (s1, s2) = (StepNode::new("A", StepType::Noop), StepNode::new("B", StepType::Noop));
    let (id1, id2) = (s1.id.clone(), s2.id.clone());
    wf.add_step(s1); wf.add_step(s2);
    wf.add_edge(Edge { from: id1.clone(), to: id2.clone(), edge_type: EdgeType::Sequence });
    wf.add_edge(Edge { from: id2, to: id1, edge_type: EdgeType::Sequence });
    assert!(engine.validate_dag(&wf).is_err());
}

#[test]
fn test_dag_self_loop_detection() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("self-loop", "");
    let s = StepNode::new("A", StepType::Noop);
    let sid = s.id.clone();
    wf.add_step(s);
    wf.add_edge(Edge { from: sid.clone(), to: sid, edge_type: EdgeType::Sequence });
    assert!(engine.validate_dag(&wf).is_err());
}

#[test]
fn test_dag_missing_step_in_edge() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("bad-edge", "");
    wf.add_step(StepNode::new("A", StepType::Noop));
    wf.add_edge(Edge { from: "nonexistent".into(), to: "also-missing".into(), edge_type: EdgeType::Sequence });
    assert!(engine.validate_dag(&wf).is_err());
}

#[test]
fn test_dag_diamond_topology() {
    let mut engine = DagEngine::new();
    let mut wf = Workflow::new("diamond", "");
    let (a, b, c, d) = (StepNode::new("A", StepType::Noop), StepNode::new("B", StepType::Noop),
                         StepNode::new("C", StepType::Noop), StepNode::new("D", StepType::Noop));
    let (aid, bid, cid, did) = (a.id.clone(), b.id.clone(), c.id.clone(), d.id.clone());
    wf.add_step(a); wf.add_step(b); wf.add_step(c); wf.add_step(d);
    wf.add_edge(Edge { from: aid.clone(), to: bid.clone(), edge_type: EdgeType::Sequence });
    wf.add_edge(Edge { from: aid, to: cid.clone(), edge_type: EdgeType::Parallel });
    wf.add_edge(Edge { from: bid, to: did.clone(), edge_type: EdgeType::Sequence });
    wf.add_edge(Edge { from: cid, to: did, edge_type: EdgeType::Sequence });
    assert!(engine.register_workflow(wf).is_ok());
}

#[test]
fn test_dag_topological_sort_order() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("linear", "");
    let (a, b, c) = (StepNode::new("A", StepType::Noop), StepNode::new("B", StepType::Noop), StepNode::new("C", StepType::Noop));
    let (aid, bid, cid) = (a.id.clone(), b.id.clone(), c.id.clone());
    wf.add_step(a); wf.add_step(b); wf.add_step(c);
    wf.add_edge(Edge { from: aid.clone(), to: bid.clone(), edge_type: EdgeType::Sequence });
    wf.add_edge(Edge { from: bid.clone(), to: cid.clone(), edge_type: EdgeType::Sequence });
    let order = engine.topological_sort(&wf).unwrap();
    assert_eq!(order[0], aid);
    assert_eq!(order[1], bid);
    assert_eq!(order[2], cid);
}

#[test]
fn test_dag_visualize_mermaid() {
    let mut engine = DagEngine::new();
    let mut wf = Workflow::new("viz", "");
    let s1 = StepNode::new("Build", StepType::Noop);
    let s2 = StepNode::new("Deploy", StepType::Noop);
    let (id1, id2) = (s1.id.clone(), s2.id.clone());
    wf.add_step(s1); wf.add_step(s2);
    wf.add_edge(Edge { from: id1, to: id2, edge_type: EdgeType::Sequence });
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let mermaid = engine.visualize_mermaid(&wfid).unwrap();
    assert!(mermaid.contains("graph TD"));
    assert!(mermaid.contains("Build"));
    assert!(mermaid.contains("Deploy"));
}

#[test]
fn test_dag_empty_workflow_validates() {
    let engine = DagEngine::new();
    let wf = Workflow::new("empty", "");
    assert!(engine.validate_dag(&wf).is_ok());
}

#[test]
fn test_dag_single_step_validates() {
    let engine = DagEngine::new();
    let mut wf = Workflow::new("single", "");
    wf.add_step(StepNode::new("Only", StepType::Noop));
    assert!(engine.validate_dag(&wf).is_ok());
}

// === Execution Lifecycle Tests ===

#[test]
fn test_execution_start_and_progress() {
    let mut engine = DagEngine::new();
    let mut wf = Workflow::new("exec", "");
    wf.add_step(StepNode::new("S1", StepType::Noop));
    wf.add_step(StepNode::new("S2", StepType::Noop));
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let eid = engine.start_execution(&wfid).unwrap();
    let p = engine.get_progress(&eid).unwrap();
    assert_eq!(p.total_steps, 2);
    assert_eq!(p.pending_steps, 2);
    assert_eq!(p.percent_complete, 0.0);
}

#[test]
fn test_execution_pause_resume_cancel() {
    let mut engine = DagEngine::new();
    let wf = Workflow::new("lifecycle", "");
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let eid = engine.start_execution(&wfid).unwrap();
    engine.pause_execution(&eid).unwrap();
    assert!(engine.resume_execution(&eid).is_ok());
    assert!(engine.cancel_execution(&eid).is_ok());
}

#[test]
fn test_execution_double_pause_fails() {
    let mut engine = DagEngine::new();
    let wf = Workflow::new("dp", "");
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let eid = engine.start_execution(&wfid).unwrap();
    engine.pause_execution(&eid).unwrap();
    assert!(engine.pause_execution(&eid).is_err());
}

#[test]
fn test_execution_resume_not_paused_fails() {
    let mut engine = DagEngine::new();
    let wf = Workflow::new("rnp", "");
    let wfid = wf.id.clone();
    engine.register_workflow(wf).unwrap();
    let eid = engine.start_execution(&wfid).unwrap();
    assert!(engine.resume_execution(&eid).is_err());
}

#[test]
fn test_execution_nonexistent_fails() {
    let engine = DagEngine::new();
    assert!(engine.get_execution("no-such-id").is_err());
}

// === Scheduler Engine Tests ===

#[test]
fn test_schedule_create_cron() {
    let mut sched = SchedulerEngine::new();
    let sid = sched.create_schedule("wf-1", ScheduleExpression::Cron("0 8 * * 1-5".into()), ConflictPolicy::Skip, "UTC").unwrap();
    assert_eq!(sched.list_schedules().len(), 1);
    assert_eq!(sched.schedules_for_workflow("wf-1").len(), 1);
    assert!(sched.remove_schedule(&sid).is_ok());
}

#[test]
fn test_schedule_pause_resume() {
    let mut sched = SchedulerEngine::new();
    let sid = sched.create_schedule("wf-1", ScheduleExpression::Interval { every_ms: 60000 }, ConflictPolicy::Queue, "UTC").unwrap();
    sched.pause_schedule(&sid).unwrap();
    sched.resume_schedule(&sid).unwrap();
}

#[test]
fn test_schedule_adaptive_recommendation() {
    let mut sched = SchedulerEngine::new();
    let sid = sched.create_schedule("wf-1", ScheduleExpression::Cron("0 8 * * *".into()), ConflictPolicy::Skip, "UTC").unwrap();
    let rec = sched.get_adaptive_recommendation(&sid).unwrap();
    assert!(rec.success_rate_at_recommended > rec.success_rate_at_current);
}

// === Trigger Engine Tests ===

#[test]
fn test_trigger_create_all_types() {
    let mut engine = TriggerEngine::new();
    engine.create_trigger("manual", "wf-1", TriggerType::Manual, None, None).unwrap();
    engine.create_trigger("webhook", "wf-1", TriggerType::Webhook { endpoint: "/hook".into(), method: "POST".into() }, None, None).unwrap();
    assert_eq!(engine.list_triggers().len(), 2);
    assert_eq!(engine.triggers_for_workflow("wf-1").len(), 2);
}

#[test]
fn test_trigger_enable_disable() {
    let mut engine = TriggerEngine::new();
    let tid = engine.create_trigger("t1", "wf-1", TriggerType::Manual, None, None).unwrap();
    engine.set_enabled(&tid, false).unwrap();
    engine.set_enabled(&tid, true).unwrap();
}

#[test]
fn test_trigger_activation_history() {
    let mut engine = TriggerEngine::new();
    let tid = engine.create_trigger("t1", "wf-1", TriggerType::Manual, None, None).unwrap();
    engine.record_activation(&tid, "exec-1", serde_json::json!({}), true).unwrap();
    engine.record_activation(&tid, "exec-2", serde_json::json!({}), false).unwrap();
    assert_eq!(engine.activation_history(&tid).len(), 2);
}

#[test]
fn test_trigger_condition_no_condition_fires() {
    let mut engine = TriggerEngine::new();
    let tid = engine.create_trigger("t1", "wf-1", TriggerType::Manual, None, None).unwrap();
    assert!(engine.test_condition(&tid, &serde_json::json!({})).unwrap());
}

// === Batch Engine Tests ===

#[test]
fn test_batch_create_and_progress() {
    let mut engine = BatchEngine::new();
    let items: Vec<serde_json::Value> = (0..100).map(|i| serde_json::json!({"id": i})).collect();
    let bid = engine.create_batch("wf-1", items, 4, 10).unwrap();
    let p = engine.get_progress(&bid).unwrap();
    assert_eq!(p.total_items, 100);
    assert_eq!(p.pending, 100);
}

#[test]
fn test_batch_zero_items() {
    let mut engine = BatchEngine::new();
    let bid = engine.create_batch("wf-1", vec![], 1, 1).unwrap();
    let p = engine.get_progress(&bid).unwrap();
    assert_eq!(p.total_items, 0);
}

// === Stream Engine Tests ===

#[test]
fn test_stream_lifecycle() {
    let mut engine = StreamEngine::new();
    let sid = engine.create_processor("watcher", "wf-1", StreamSource::FileWatch { path: "/tmp".into(), pattern: None }, None, 100).unwrap();
    assert_eq!(engine.get_processor(&sid).unwrap().status, StreamStatus::Created);
    engine.start(&sid).unwrap();
    assert_eq!(engine.get_processor(&sid).unwrap().status, StreamStatus::Running);
    engine.pause(&sid).unwrap();
    engine.stop(&sid).unwrap();
    assert_eq!(engine.get_processor(&sid).unwrap().status, StreamStatus::Stopped);
}

#[test]
fn test_stream_checkpoint_and_fork() {
    let mut engine = StreamEngine::new();
    let sid = engine.create_processor("p1", "wf-1", StreamSource::FileWatch { path: "/tmp".into(), pattern: None }, None, 50).unwrap();
    engine.checkpoint(&sid, 42, 100).unwrap();
    engine.add_fork(&sid, "errors", "status == 'error'", "wf-error").unwrap();
}

// === FanOut Engine Tests ===

#[test]
fn test_fanout_branch_tracking() {
    let mut engine = FanOutEngine::new();
    let dests = vec![
        FanOutDestination { id: "d1".into(), name: "API 1".into(), step_config: serde_json::json!({}) },
        FanOutDestination { id: "d2".into(), name: "API 2".into(), step_config: serde_json::json!({}) },
        FanOutDestination { id: "d3".into(), name: "API 3".into(), step_config: serde_json::json!({}) },
    ];
    let fid = engine.create_fanout(dests, CompletionPolicy::WaitAll, ResultAggregator::Merge, None).unwrap();
    engine.start_execution(&fid, "exec-1").unwrap();
    let status = engine.get_status("exec-1").unwrap();
    assert_eq!(status.branches.len(), 3);
    assert!(!status.completed);
    engine.update_branch("exec-1", "d1", FanOutBranchStatus::Success, Some(serde_json::json!(1)), None, Some(100)).unwrap();
    engine.update_branch("exec-1", "d2", FanOutBranchStatus::Success, Some(serde_json::json!(2)), None, Some(200)).unwrap();
    engine.update_branch("exec-1", "d3", FanOutBranchStatus::Failed, None, Some("timeout".into()), None).unwrap();
    let status = engine.get_status("exec-1").unwrap();
    assert!(status.completed);
}

// === FSM Engine Tests ===

fn order_fsm() -> (Vec<State>, Vec<Transition>) {
    let states = vec![
        State { name: "Created".into(), description: None, entry_action: None, exit_action: None, is_terminal: false },
        State { name: "Paid".into(), description: None, entry_action: None, exit_action: None, is_terminal: false },
        State { name: "Shipped".into(), description: None, entry_action: None, exit_action: None, is_terminal: false },
        State { name: "Delivered".into(), description: None, entry_action: None, exit_action: None, is_terminal: true },
    ];
    let transitions = vec![
        Transition { from: "Created".into(), to: "Paid".into(), event: "pay".into(), guard: None, action: None },
        Transition { from: "Paid".into(), to: "Shipped".into(), event: "ship".into(), guard: None, action: None },
        Transition { from: "Shipped".into(), to: "Delivered".into(), event: "deliver".into(), guard: None, action: None },
    ];
    (states, transitions)
}

#[test]
fn test_fsm_full_lifecycle() {
    let mut engine = FsmEngine::new();
    let (states, transitions) = order_fsm();
    let fid = engine.create_fsm("order", states, transitions, "Created").unwrap();
    assert_eq!(engine.current_state(&fid).unwrap(), "Created");
    engine.transition(&fid, "pay").unwrap();
    assert_eq!(engine.current_state(&fid).unwrap(), "Paid");
    engine.transition(&fid, "ship").unwrap();
    engine.transition(&fid, "deliver").unwrap();
    assert_eq!(engine.current_state(&fid).unwrap(), "Delivered");
    assert_eq!(engine.get_history(&fid).unwrap().len(), 3);
}

#[test]
fn test_fsm_invalid_transition() {
    let mut engine = FsmEngine::new();
    let (states, transitions) = order_fsm();
    let fid = engine.create_fsm("order", states, transitions, "Created").unwrap();
    assert!(engine.transition(&fid, "ship").is_err()); // Can't ship before paying
}

#[test]
fn test_fsm_valid_next_transitions() {
    let mut engine = FsmEngine::new();
    let (states, transitions) = order_fsm();
    let fid = engine.create_fsm("order", states, transitions, "Created").unwrap();
    let valid = engine.valid_next(&fid).unwrap();
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0].event, "pay");
}

#[test]
fn test_fsm_diagram_output() {
    let mut engine = FsmEngine::new();
    let (states, transitions) = order_fsm();
    let fid = engine.create_fsm("order", states, transitions, "Created").unwrap();
    let diagram = engine.diagram(&fid).unwrap();
    assert!(diagram.contains("stateDiagram-v2"));
    assert!(diagram.contains("Created --> Paid : pay"));
    assert!(diagram.contains("Delivered --> [*]"));
}

#[test]
fn test_fsm_invalid_initial_state() {
    let mut engine = FsmEngine::new();
    let (states, transitions) = order_fsm();
    assert!(engine.create_fsm("bad", states, transitions, "Nonexistent").is_err());
}
