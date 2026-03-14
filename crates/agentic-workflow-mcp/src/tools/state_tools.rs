use serde_json::json;

use crate::types::{ToolDefinition, ToolResult};
use super::registry::EngineState;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "workflow_fsm_create".to_string(),
            description: "Create a finite state machine with states and transitions".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "FSM name" },
                    "states": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "is_terminal": { "type": "boolean" }
                            }
                        },
                        "description": "List of states"
                    },
                    "transitions": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": { "type": "string" },
                                "to": { "type": "string" },
                                "event": { "type": "string" }
                            }
                        },
                        "description": "List of transitions"
                    },
                    "initial_state": { "type": "string", "description": "Starting state name" }
                },
                "required": ["name", "states", "transitions", "initial_state"]
            }),
        },
        ToolDefinition {
            name: "workflow_fsm_transition".to_string(),
            description: "Trigger a state transition by sending an event".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fsm_id": { "type": "string", "description": "FSM ID" },
                    "event": { "type": "string", "description": "Event to trigger" }
                },
                "required": ["fsm_id", "event"]
            }),
        },
        ToolDefinition {
            name: "workflow_fsm_state".to_string(),
            description: "Get the current state of a finite state machine".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fsm_id": { "type": "string", "description": "FSM ID" }
                },
                "required": ["fsm_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_fsm_valid_next".to_string(),
            description: "List valid next transitions from the current state".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fsm_id": { "type": "string", "description": "FSM ID" }
                },
                "required": ["fsm_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_fsm_history".to_string(),
            description: "Get the transition history of a finite state machine".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fsm_id": { "type": "string", "description": "FSM ID" }
                },
                "required": ["fsm_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_fsm_diagram".to_string(),
            description: "Generate a Mermaid state diagram for an FSM".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "fsm_id": { "type": "string", "description": "FSM ID" }
                },
                "required": ["fsm_id"]
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
        "workflow_fsm_create" => {
            let fsm_name = params["name"].as_str().unwrap_or("fsm");
            let initial = params["initial_state"].as_str().unwrap_or("");
            let states: Vec<agentic_workflow::types::State> = params["states"]
                .as_array()
                .map(|arr| arr.iter().map(|s| agentic_workflow::types::State {
                    name: s["name"].as_str().unwrap_or("").to_string(),
                    description: s["description"].as_str().map(String::from),
                    entry_action: None,
                    exit_action: None,
                    is_terminal: s["is_terminal"].as_bool().unwrap_or(false),
                }).collect())
                .unwrap_or_default();
            let transitions: Vec<agentic_workflow::types::Transition> = params["transitions"]
                .as_array()
                .map(|arr| arr.iter().map(|t| agentic_workflow::types::Transition {
                    from: t["from"].as_str().unwrap_or("").to_string(),
                    to: t["to"].as_str().unwrap_or("").to_string(),
                    event: t["event"].as_str().unwrap_or("").to_string(),
                    guard: None,
                    action: None,
                }).collect())
                .unwrap_or_default();
            match state.fsm.create_fsm(fsm_name, states, transitions, initial) {
                Ok(fid) => Ok(ToolResult::text(json!({
                    "fsm_id": fid,
                    "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fsm_transition" => {
            let fid = params["fsm_id"].as_str().unwrap_or("");
            let event = params["event"].as_str().unwrap_or("");
            match state.fsm.transition(fid, event) {
                Ok(new_state) => Ok(ToolResult::text(json!({
                    "fsm_id": fid,
                    "new_state": new_state
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fsm_state" => {
            let fid = params["fsm_id"].as_str().unwrap_or("");
            match state.fsm.current_state(fid) {
                Ok(s) => Ok(ToolResult::text(json!({
                    "fsm_id": fid,
                    "current_state": s
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fsm_valid_next" => {
            let fid = params["fsm_id"].as_str().unwrap_or("");
            match state.fsm.valid_next(fid) {
                Ok(transitions) => {
                    let items: Vec<_> = transitions.iter().map(|t| json!({
                        "event": t.event,
                        "to": t.to
                    })).collect();
                    Ok(ToolResult::text(json!({
                        "fsm_id": fid,
                        "valid_transitions": items
                    }).to_string()))
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fsm_history" => {
            let fid = params["fsm_id"].as_str().unwrap_or("");
            match state.fsm.get_history(fid) {
                Ok(records) => {
                    let items: Vec<_> = records.iter().map(|r| json!({
                        "from": r.from_state,
                        "to": r.to_state,
                        "event": r.event,
                        "timestamp": r.timestamp.to_rfc3339()
                    })).collect();
                    Ok(ToolResult::text(json!({
                        "fsm_id": fid,
                        "history": items
                    }).to_string()))
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_fsm_diagram" => {
            let fid = params["fsm_id"].as_str().unwrap_or("");
            match state.fsm.diagram(fid) {
                Ok(diagram) => Ok(ToolResult::text(diagram)),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        _ => Ok(ToolResult::error(format!("Unknown FSM tool: {}", name))),
    }
}
