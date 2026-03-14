use serde_json::json;

use crate::types::{ToolDefinition, ToolResult};
use super::registry::EngineState;

fn def(name: &str, desc: &str, props: serde_json::Value) -> ToolDefinition {
    ToolDefinition {
        name: name.to_string(),
        description: desc.to_string(),
        input_schema: json!({ "type": "object", "properties": props }),
    }
}

fn s(d: &str) -> serde_json::Value { json!({ "type": "string", "description": d }) }
fn i(d: &str) -> serde_json::Value { json!({ "type": "integer", "description": d }) }

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Batch (5)
        def("workflow_batch_create", "Create a batch job to process multiple items through a workflow",
            json!({ "workflow_id": s("Workflow ID"), "items": json!({ "type": "array", "description": "Items to process" }), "concurrency": i("Max parallel items"), "checkpoint_every": i("Checkpoint interval") })),
        def("workflow_batch_run", "Start executing a batch job",
            json!({ "batch_id": s("Batch job ID") })),
        def("workflow_batch_progress", "Get progress of a batch job",
            json!({ "batch_id": s("Batch job ID") })),
        def("workflow_batch_resume", "Resume a batch job from its last checkpoint",
            json!({ "batch_id": s("Batch job ID") })),
        def("workflow_batch_report", "Get completion report for a batch job",
            json!({ "batch_id": s("Batch job ID") })),
        // Stream (6)
        def("workflow_stream_create", "Create a stream processor for continuous event processing",
            json!({ "name": s("Processor name"), "workflow_id": s("Workflow ID"), "source_type": s("file_watch, queue, webhook"), "max_queue_size": i("Max queue size") })),
        def("workflow_stream_start", "Start consuming from a stream",
            json!({ "stream_id": s("Stream processor ID") })),
        def("workflow_stream_status", "Get status of a stream processor",
            json!({ "stream_id": s("Stream processor ID") })),
        def("workflow_stream_pause", "Pause stream consumption",
            json!({ "stream_id": s("Stream processor ID") })),
        def("workflow_stream_checkpoint", "Force a checkpoint at the current stream position",
            json!({ "stream_id": s("Stream processor ID"), "offset": i("Current offset"), "items_processed": i("Items processed so far") })),
        def("workflow_stream_fork", "Add a fork to split stream events by condition",
            json!({ "stream_id": s("Stream processor ID"), "name": s("Fork name"), "condition": s("Fork condition"), "target_workflow_id": s("Target workflow for matching events") })),
        // Fan-out (4)
        def("workflow_fanout_create", "Create a fan-out step for parallel distribution",
            json!({ "destinations": json!({ "type": "array", "items": { "type": "object" }, "description": "Destination configs" }), "completion_policy": s("wait_all, wait_any, wait_n"), "timeout_ms": i("Timeout") })),
        def("workflow_fanout_execute", "Start executing a fan-out step",
            json!({ "fanout_id": s("Fan-out step ID"), "execution_id": s("Execution ID") })),
        def("workflow_fanout_status", "Get status of a fan-out execution",
            json!({ "execution_id": s("Execution ID") })),
        def("workflow_fanout_policy", "Get the fan-out step definition and completion policy",
            json!({ "fanout_id": s("Fan-out step ID") })),
    ]
}

pub fn dispatch(
    name: &str,
    params: serde_json::Value,
    state: &mut EngineState,
) -> Result<ToolResult, (i32, String)> {
    match name {
        // --- Batch ---
        "workflow_batch_create" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let items: Vec<serde_json::Value> = params["items"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let concurrency = params["concurrency"].as_u64().unwrap_or(1) as usize;
            let checkpoint = params["checkpoint_every"].as_u64().unwrap_or(10) as usize;
            match state.batch.create_batch(wid, items, concurrency, checkpoint) {
                Ok(bid) => Ok(ToolResult::text(json!({
                    "batch_id": bid, "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_batch_run" => {
            let bid = params["batch_id"].as_str().unwrap_or("");
            match state.batch.get_job(bid) {
                Ok(_) => Ok(ToolResult::text(json!({
                    "batch_id": bid, "status": "running"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_batch_progress" => {
            let bid = params["batch_id"].as_str().unwrap_or("");
            match state.batch.get_progress(bid) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "batch_id": p.batch_id, "total": p.total_items,
                    "completed": p.completed, "failed": p.failed,
                    "percent_complete": p.percent_complete
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_batch_resume" => {
            let bid = params["batch_id"].as_str().unwrap_or("");
            match state.batch.get_progress(bid) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "batch_id": bid, "status": "resumed",
                    "resuming_from_checkpoint": p.last_checkpoint_index
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_batch_report" => {
            let bid = params["batch_id"].as_str().unwrap_or("");
            match state.batch.get_report(bid) {
                Ok(r) => Ok(ToolResult::text(json!({
                    "batch_id": r.batch_id, "total": r.total_items,
                    "success": r.success_count, "failed": r.fail_count,
                    "avg_duration_ms": r.avg_item_duration_ms
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        // --- Stream ---
        "workflow_stream_create" => {
            let sname = params["name"].as_str().unwrap_or("stream");
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let source = match params["source_type"].as_str().unwrap_or("file_watch") {
                "queue" => agentic_workflow::types::StreamSource::Queue {
                    queue_name: "default".to_string(),
                    connection: "localhost".to_string(),
                },
                "webhook" => agentic_workflow::types::StreamSource::Webhook {
                    endpoint: "/stream".to_string(),
                },
                _ => agentic_workflow::types::StreamSource::FileWatch {
                    path: "/tmp".to_string(),
                    pattern: None,
                },
            };
            let max_q = params["max_queue_size"].as_u64().unwrap_or(100) as usize;
            match state.stream.create_processor(sname, wid, source, None, max_q) {
                Ok(sid) => Ok(ToolResult::text(json!({
                    "stream_id": sid, "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_stream_start" => {
            let sid = params["stream_id"].as_str().unwrap_or("");
            match state.stream.start(sid) {
                Ok(()) => Ok(ToolResult::text(json!({ "stream_id": sid, "status": "running" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_stream_status" => {
            let sid = params["stream_id"].as_str().unwrap_or("");
            match state.stream.get_processor(sid) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "stream_id": p.id, "name": p.name,
                    "status": format!("{:?}", p.status)
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_stream_pause" => {
            let sid = params["stream_id"].as_str().unwrap_or("");
            match state.stream.pause(sid) {
                Ok(()) => Ok(ToolResult::text(json!({ "stream_id": sid, "status": "paused" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_stream_checkpoint" => {
            let sid = params["stream_id"].as_str().unwrap_or("");
            let offset = params["offset"].as_u64().unwrap_or(0);
            let items = params["items_processed"].as_u64().unwrap_or(0);
            match state.stream.checkpoint(sid, offset, items) {
                Ok(()) => Ok(ToolResult::text(json!({
                    "stream_id": sid, "offset": offset, "status": "checkpointed"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_stream_fork" => {
            let sid = params["stream_id"].as_str().unwrap_or("");
            let fname = params["name"].as_str().unwrap_or("fork");
            let condition = params["condition"].as_str().unwrap_or("true");
            let target = params["target_workflow_id"].as_str().unwrap_or("");
            match state.stream.add_fork(sid, fname, condition, target) {
                Ok(fid) => Ok(ToolResult::text(json!({
                    "fork_id": fid, "stream_id": sid, "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        // --- Fan-out ---
        "workflow_fanout_create" => {
            let dests: Vec<agentic_workflow::types::fanout::FanOutDestination> = params["destinations"]
                .as_array()
                .map(|arr| arr.iter().enumerate().map(|(i, d)| {
                    agentic_workflow::types::FanOutDestination {
                        id: d["id"].as_str().unwrap_or(&format!("d{}", i)).to_string(),
                        name: d["name"].as_str().unwrap_or("dest").to_string(),
                        step_config: d["config"].clone(),
                    }
                }).collect())
                .unwrap_or_default();
            let policy = match params["completion_policy"].as_str().unwrap_or("wait_all") {
                "wait_any" => agentic_workflow::types::CompletionPolicy::WaitAny,
                "wait_n" => agentic_workflow::types::CompletionPolicy::WaitN(1),
                _ => agentic_workflow::types::CompletionPolicy::WaitAll,
            };
            let agg = agentic_workflow::types::ResultAggregator::Merge;
            let timeout = params["timeout_ms"].as_u64();
            match state.fanout.create_fanout(dests, policy, agg, timeout) {
                Ok(fid) => Ok(ToolResult::text(json!({
                    "fanout_id": fid, "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fanout_execute" => {
            let fid = params["fanout_id"].as_str().unwrap_or("");
            let eid = params["execution_id"].as_str().unwrap_or("");
            match state.fanout.start_execution(fid, eid) {
                Ok(()) => Ok(ToolResult::text(json!({
                    "fanout_id": fid, "execution_id": eid, "status": "executing"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fanout_status" => {
            let eid = params["execution_id"].as_str().unwrap_or("");
            match state.fanout.get_status(eid) {
                Ok(st) => {
                    let branches: Vec<_> = st.branches.iter().map(|b| json!({
                        "destination_id": b.destination_id,
                        "status": format!("{:?}", b.status)
                    })).collect();
                    Ok(ToolResult::text(json!({
                        "execution_id": eid, "completed": st.completed,
                        "branches": branches
                    }).to_string()))
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fanout_policy" => {
            let fid = params["fanout_id"].as_str().unwrap_or("");
            match state.fanout.get_step(fid) {
                Ok(step) => Ok(ToolResult::text(json!({
                    "fanout_id": step.id,
                    "destinations": step.destinations.len(),
                    "timeout_ms": step.timeout_ms
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        _ => Ok(ToolResult::error(format!("Unknown processing tool: {}", name))),
    }
}
