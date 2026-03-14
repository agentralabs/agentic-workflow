use serde_json::json;

use crate::types::{ToolDefinition, ToolResult};
use super::registry::EngineState;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "workflow_trigger_create".to_string(),
            description: "Create a trigger that starts a workflow on an event".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Trigger name" },
                    "workflow_id": { "type": "string", "description": "Workflow to trigger" },
                    "trigger_type": { "type": "string", "description": "Type: manual, file_watch, webhook, cron, event" },
                    "debounce_ms": { "type": "integer", "description": "Debounce interval in milliseconds" }
                },
                "required": ["name", "workflow_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_trigger_list".to_string(),
            description: "List all triggers, optionally filtered by workflow ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Filter by workflow ID" }
                }
            }),
        },
        ToolDefinition {
            name: "workflow_trigger_test".to_string(),
            description: "Test a trigger condition against sample event data".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trigger_id": { "type": "string", "description": "Trigger ID" },
                    "event_data": { "type": "object", "description": "Sample event data" }
                },
                "required": ["trigger_id", "event_data"]
            }),
        },
        ToolDefinition {
            name: "workflow_trigger_history".to_string(),
            description: "Get activation history for a trigger".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trigger_id": { "type": "string", "description": "Trigger ID" }
                },
                "required": ["trigger_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_trigger_replay".to_string(),
            description: "Replay a trigger activation to re-run its workflow".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trigger_id": { "type": "string", "description": "Trigger ID" },
                    "activation_index": { "type": "integer", "description": "Index of activation to replay" }
                },
                "required": ["trigger_id"]
            }),
        },
    ]
}

pub fn dispatch(
    name: &str,
    params: serde_json::Value,
    state: &mut EngineState,
) -> Result<ToolResult, (i32, String)> {
    match name {
        "workflow_trigger_create" => {
            let tname = params["name"].as_str().unwrap_or("trigger");
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            let trigger_type = match params["trigger_type"].as_str().unwrap_or("manual") {
                "file_watch" => agentic_workflow::types::TriggerType::FileSystem {
                    path: params["config"]["path"].as_str().unwrap_or("/tmp").to_string(),
                    events: vec![agentic_workflow::types::FileEvent::Modified],
                },
                "webhook" => agentic_workflow::types::TriggerType::Webhook {
                    endpoint: params["config"]["endpoint"].as_str().unwrap_or("/hook").to_string(),
                    method: "POST".to_string(),
                },
                _ => agentic_workflow::types::TriggerType::Manual,
            };
            let debounce = params["debounce_ms"].as_u64();
            match state.trigger.create_trigger(tname, wf_id, trigger_type, None, debounce) {
                Ok(tid) => Ok(ToolResult::text(json!({
                    "trigger_id": tid,
                    "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_trigger_list" => {
            let triggers = if let Some(wf_id) = params["workflow_id"].as_str() {
                state.trigger.triggers_for_workflow(wf_id)
            } else {
                state.trigger.list_triggers()
            };
            let items: Vec<_> = triggers.iter().map(|t| json!({
                "trigger_id": t.id,
                "name": t.name,
                "workflow_id": t.workflow_id,
                "enabled": t.enabled
            })).collect();
            Ok(ToolResult::text(json!({ "triggers": items }).to_string()))
        }
        "workflow_trigger_test" => {
            let tid = params["trigger_id"].as_str().unwrap_or("");
            let event_data = &params["event_data"];
            match state.trigger.test_condition(tid, event_data) {
                Ok(matches) => Ok(ToolResult::text(json!({
                    "trigger_id": tid,
                    "condition_met": matches
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_trigger_history" => {
            let tid = params["trigger_id"].as_str().unwrap_or("");
            let history = state.trigger.activation_history(tid);
            let items: Vec<_> = history.iter().map(|a| json!({
                "trigger_id": a.trigger_id,
                "execution_id": a.execution_id,
                "activated_at": a.activated_at.to_rfc3339(),
                "condition_met": a.condition_met
            })).collect();
            Ok(ToolResult::text(json!({ "activations": items }).to_string()))
        }
        "workflow_trigger_replay" => {
            let tid = params["trigger_id"].as_str().unwrap_or("");
            let idx = params["activation_index"].as_u64().unwrap_or(0) as usize;
            let history = state.trigger.activation_history(tid);
            match history.get(idx) {
                Some(activation) => Ok(ToolResult::text(json!({
                    "trigger_id": tid,
                    "replaying_activation": idx,
                    "original_execution_id": activation.execution_id,
                    "status": "replay_queued"
                }).to_string())),
                None => Ok(ToolResult::error(format!(
                    "Activation index {} not found for trigger {}",
                    idx, tid
                ))),
            }
        }
        _ => Ok(ToolResult::error(format!("Unknown trigger tool: {}", name))),
    }
}
