use serde_json::json;

use crate::types::{ToolDefinition, ToolResult};
use super::registry::EngineState;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "workflow_schedule".to_string(),
            description: "Create a schedule for a workflow with cron or interval expression".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Workflow ID" },
                    "expression": { "type": "string", "description": "Cron expression or interval like '5m', '1h'" },
                    "timezone": { "type": "string", "description": "Timezone, e.g. UTC" },
                    "conflict_policy": { "type": "string", "description": "Policy when overlapping: skip, queue, cancel_previous" }
                },
                "required": ["workflow_id", "expression"]
            }),
        },
        ToolDefinition {
            name: "workflow_schedule_list".to_string(),
            description: "List all workflow schedules, optionally filtered by workflow ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Filter by workflow ID" }
                }
            }),
        },
        ToolDefinition {
            name: "workflow_schedule_next".to_string(),
            description: "Get the next scheduled fire time for a schedule".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "schedule_id": { "type": "string", "description": "Schedule ID" }
                },
                "required": ["schedule_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_schedule_pause".to_string(),
            description: "Pause or resume a workflow schedule".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "schedule_id": { "type": "string", "description": "Schedule ID" },
                    "enabled": { "type": "boolean", "description": "True to enable, false to pause" }
                },
                "required": ["schedule_id", "enabled"]
            }),
        },
        ToolDefinition {
            name: "workflow_schedule_adapt".to_string(),
            description: "Get adaptive schedule recommendations based on execution history".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "schedule_id": { "type": "string", "description": "Schedule ID" }
                },
                "required": ["schedule_id"]
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
        "workflow_schedule" => {
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            let expr = params["expression"].as_str().unwrap_or("0 * * * *");
            let tz = params["timezone"].as_str().unwrap_or("UTC");
            let conflict = match params["conflict_policy"].as_str().unwrap_or("skip") {
                "queue" => agentic_workflow::types::ConflictPolicy::Queue,
                "cancel_previous" => agentic_workflow::types::ConflictPolicy::CancelPrevious,
                _ => agentic_workflow::types::ConflictPolicy::Skip,
            };
            let expression = agentic_workflow::types::ScheduleExpression::Cron(expr.to_string());
            match state.scheduler.create_schedule(wf_id, expression, conflict, tz) {
                Ok(sid) => Ok(ToolResult::text(json!({
                    "schedule_id": sid,
                    "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_schedule_list" => {
            let schedules = if let Some(wf_id) = params["workflow_id"].as_str() {
                state.scheduler.schedules_for_workflow(wf_id)
            } else {
                state.scheduler.list_schedules()
            };
            let items: Vec<_> = schedules.iter().map(|s| json!({
                "schedule_id": s.id,
                "workflow_id": s.workflow_id,
                "enabled": s.enabled,
                "timezone": s.timezone
            })).collect();
            Ok(ToolResult::text(json!({ "schedules": items }).to_string()))
        }
        "workflow_schedule_next" => {
            let sid = params["schedule_id"].as_str().unwrap_or("");
            let schedules = state.scheduler.list_schedules();
            match schedules.iter().find(|s| s.id == sid) {
                Some(s) => Ok(ToolResult::text(json!({
                    "schedule_id": s.id,
                    "next_fire_at": s.next_fire_at.map(|t| t.to_rfc3339()),
                    "last_fired_at": s.last_fired_at.map(|t| t.to_rfc3339()),
                    "enabled": s.enabled
                }).to_string())),
                None => Ok(ToolResult::error(format!("Schedule not found: {}", sid))),
            }
        }
        "workflow_schedule_pause" => {
            let sid = params["schedule_id"].as_str().unwrap_or("");
            let enabled = params["enabled"].as_bool().unwrap_or(false);
            let result = if enabled {
                state.scheduler.resume_schedule(sid)
            } else {
                state.scheduler.pause_schedule(sid)
            };
            match result {
                Ok(()) => Ok(ToolResult::text(json!({
                    "schedule_id": sid,
                    "enabled": enabled
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_schedule_adapt" => {
            let sid = params["schedule_id"].as_str().unwrap_or("");
            match state.scheduler.get_adaptive_recommendation(sid) {
                Ok(rec) => Ok(ToolResult::text(json!({
                    "schedule_id": rec.schedule_id,
                    "recommended_time": rec.recommended_time,
                    "reason": rec.reason,
                    "success_rate_at_recommended": rec.success_rate_at_recommended,
                    "success_rate_at_current": rec.success_rate_at_current
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        _ => Ok(ToolResult::error(format!("Unknown schedule tool: {}", name))),
    }
}
