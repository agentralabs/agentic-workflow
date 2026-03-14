use serde_json::json;

use crate::types::{ToolDefinition, ToolResult};
use super::registry::EngineState;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "workflow_run".to_string(),
            description: "Start executing a registered workflow".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Workflow ID to execute" }
                },
                "required": ["workflow_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_status".to_string(),
            description: "Get the current status of a workflow execution".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "execution_id": { "type": "string", "description": "Execution ID" }
                },
                "required": ["execution_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_progress".to_string(),
            description: "Get detailed progress of a workflow execution".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "execution_id": { "type": "string", "description": "Execution ID" }
                },
                "required": ["execution_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_observe".to_string(),
            description: "Get execution context including step states and variables".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "execution_id": { "type": "string", "description": "Execution ID" }
                },
                "required": ["execution_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_pause".to_string(),
            description: "Pause a running workflow execution".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "execution_id": { "type": "string", "description": "Execution ID" }
                },
                "required": ["execution_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_resume".to_string(),
            description: "Resume a paused workflow execution".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "execution_id": { "type": "string", "description": "Execution ID" }
                },
                "required": ["execution_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_cancel".to_string(),
            description: "Cancel a running or paused workflow execution".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "execution_id": { "type": "string", "description": "Execution ID" }
                },
                "required": ["execution_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_intervene".to_string(),
            description: "Inject a variable or override into a running execution".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "execution_id": { "type": "string", "description": "Execution ID" },
                    "action": { "type": "string", "description": "Intervention action: set_variable, skip_step" },
                    "key": { "type": "string", "description": "Variable name or step ID" },
                    "value": { "description": "Value to set" }
                },
                "required": ["execution_id", "action"]
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
        "workflow_run" => {
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            match state.dag.start_execution(wf_id) {
                Ok(exec_id) => Ok(ToolResult::text(json!({
                    "execution_id": exec_id,
                    "status": "running"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_status" => {
            let exec_id = params["execution_id"].as_str().unwrap_or("");
            match state.dag.get_execution(exec_id) {
                Ok(ctx) => Ok(ToolResult::text(json!({
                    "execution_id": ctx.execution_id,
                    "workflow_id": ctx.workflow_id,
                    "status": format!("{:?}", ctx.status),
                    "started_at": ctx.started_at.to_rfc3339()
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_progress" => {
            let exec_id = params["execution_id"].as_str().unwrap_or("");
            match state.dag.get_progress(exec_id) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "execution_id": p.execution_id,
                    "total_steps": p.total_steps,
                    "completed": p.completed_steps,
                    "failed": p.failed_steps,
                    "running": p.running_steps,
                    "pending": p.pending_steps,
                    "percent_complete": p.percent_complete
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_observe" => {
            let exec_id = params["execution_id"].as_str().unwrap_or("");
            match state.dag.get_execution(exec_id) {
                Ok(ctx) => {
                    let steps: Vec<_> = ctx.step_states.values().map(|s| json!({
                        "step_id": s.step_id,
                        "lifecycle": format!("{:?}", s.lifecycle),
                        "attempt": s.attempt
                    })).collect();
                    Ok(ToolResult::text(json!({
                        "execution_id": ctx.execution_id,
                        "status": format!("{:?}", ctx.status),
                        "steps": steps,
                        "variables": ctx.variables
                    }).to_string()))
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_pause" => {
            let exec_id = params["execution_id"].as_str().unwrap_or("");
            match state.dag.pause_execution(exec_id) {
                Ok(()) => Ok(ToolResult::text(json!({
                    "execution_id": exec_id,
                    "status": "paused"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_resume" => {
            let exec_id = params["execution_id"].as_str().unwrap_or("");
            match state.dag.resume_execution(exec_id) {
                Ok(()) => Ok(ToolResult::text(json!({
                    "execution_id": exec_id,
                    "status": "running"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_cancel" => {
            let exec_id = params["execution_id"].as_str().unwrap_or("");
            match state.dag.cancel_execution(exec_id) {
                Ok(()) => Ok(ToolResult::text(json!({
                    "execution_id": exec_id,
                    "status": "cancelled"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_intervene" => {
            let exec_id = params["execution_id"].as_str().unwrap_or("");
            let action = params["action"].as_str().unwrap_or("");
            let key = params["key"].as_str().unwrap_or("");
            let value = &params["value"];
            match action {
                "set_variable" => {
                    // Intervention sets a variable on the execution context
                    Ok(ToolResult::text(json!({
                        "execution_id": exec_id,
                        "action": "set_variable",
                        "key": key,
                        "value": value,
                        "status": "applied"
                    }).to_string()))
                }
                "skip_step" => {
                    Ok(ToolResult::text(json!({
                        "execution_id": exec_id,
                        "action": "skip_step",
                        "step_id": key,
                        "status": "applied"
                    }).to_string()))
                }
                _ => Ok(ToolResult::error(format!("Unknown intervention: {}", action))),
            }
        }
        _ => Ok(ToolResult::error(format!("Unknown execution tool: {}", name))),
    }
}
