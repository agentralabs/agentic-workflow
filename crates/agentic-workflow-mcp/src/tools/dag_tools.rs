use serde_json::json;

use crate::types::{ToolDefinition, ToolResult};
use super::registry::EngineState;

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "workflow_create".to_string(),
            description: "Create a new workflow DAG with a name and description".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Workflow name" },
                    "description": { "type": "string", "description": "Workflow description" }
                },
                "required": ["name", "description"]
            }),
        },
        ToolDefinition {
            name: "workflow_step_add".to_string(),
            description: "Add a step to an existing workflow".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Workflow ID" },
                    "name": { "type": "string", "description": "Step name" },
                    "step_type": { "type": "string", "description": "Step type: command, mcp_tool, http, sub_workflow, noop" },
                    "config": { "type": "object", "description": "Step type configuration" }
                },
                "required": ["workflow_id", "name"]
            }),
        },
        ToolDefinition {
            name: "workflow_step_remove".to_string(),
            description: "Remove a step from a workflow by step ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Workflow ID" },
                    "step_id": { "type": "string", "description": "Step ID to remove" }
                },
                "required": ["workflow_id", "step_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_edge_add".to_string(),
            description: "Add a directed edge between two workflow steps".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Workflow ID" },
                    "from": { "type": "string", "description": "Source step ID" },
                    "to": { "type": "string", "description": "Target step ID" },
                    "edge_type": { "type": "string", "description": "Edge type: sequence, parallel, conditional, loop" },
                    "expression": { "type": "string", "description": "Condition expression for conditional edges" }
                },
                "required": ["workflow_id", "from", "to"]
            }),
        },
        ToolDefinition {
            name: "workflow_validate".to_string(),
            description: "Validate a workflow DAG for cycles and missing dependencies".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Workflow ID" }
                },
                "required": ["workflow_id"]
            }),
        },
        ToolDefinition {
            name: "workflow_visualize".to_string(),
            description: "Generate a Mermaid diagram for a workflow".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "workflow_id": { "type": "string", "description": "Workflow ID" }
                },
                "required": ["workflow_id"]
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
        "workflow_create" => {
            let wf_name = params["name"].as_str().unwrap_or("unnamed");
            let desc = params["description"].as_str().unwrap_or("");
            let wf = agentic_workflow::types::Workflow::new(wf_name, desc);
            let id = wf.id.clone();
            match state.dag.register_workflow(wf) {
                Ok(()) => Ok(ToolResult::text(json!({
                    "workflow_id": id,
                    "status": "created"
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_step_add" => {
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            let step_name = params["name"].as_str().unwrap_or("step");
            let step_type = match params["step_type"].as_str().unwrap_or("noop") {
                "command" => {
                    let cmd = params["config"]["command"].as_str().unwrap_or("").to_string();
                    let args = params["config"]["args"]
                        .as_array()
                        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();
                    agentic_workflow::types::StepType::Command { command: cmd, args }
                }
                _ => agentic_workflow::types::StepType::Noop,
            };
            let step = agentic_workflow::types::StepNode::new(step_name, step_type);
            let step_id = step.id.clone();
            // We need to get the workflow, add the step, and re-register
            match state.dag.get_workflow(wf_id) {
                Ok(wf) => {
                    let mut wf = wf.clone();
                    wf.add_step(step);
                    let _ = state.dag.remove_workflow(wf_id);
                    match state.dag.register_workflow(wf) {
                        Ok(()) => Ok(ToolResult::text(json!({
                            "step_id": step_id,
                            "status": "added"
                        }).to_string())),
                        Err(e) => Ok(ToolResult::error(format!("{}", e))),
                    }
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_step_remove" => {
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            let step_id = params["step_id"].as_str().unwrap_or("");
            match state.dag.get_workflow(wf_id) {
                Ok(wf) => {
                    let mut wf = wf.clone();
                    wf.remove_step(step_id);
                    let _ = state.dag.remove_workflow(wf_id);
                    match state.dag.register_workflow(wf) {
                        Ok(()) => Ok(ToolResult::text(json!({
                            "step_id": step_id,
                            "status": "removed"
                        }).to_string())),
                        Err(e) => Ok(ToolResult::error(format!("{}", e))),
                    }
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_edge_add" => {
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            let from = params["from"].as_str().unwrap_or("").to_string();
            let to = params["to"].as_str().unwrap_or("").to_string();
            let edge_type = match params["edge_type"].as_str().unwrap_or("sequence") {
                "parallel" => agentic_workflow::types::EdgeType::Parallel,
                "conditional" => {
                    let expr = params["expression"].as_str().unwrap_or("true").to_string();
                    agentic_workflow::types::EdgeType::Conditional { expression: expr }
                }
                "loop" => agentic_workflow::types::EdgeType::Loop {
                    max_iterations: None,
                    condition: None,
                },
                _ => agentic_workflow::types::EdgeType::Sequence,
            };
            let edge = agentic_workflow::types::Edge { from, to, edge_type };
            match state.dag.get_workflow(wf_id) {
                Ok(wf) => {
                    let mut wf = wf.clone();
                    wf.add_edge(edge);
                    let _ = state.dag.remove_workflow(wf_id);
                    match state.dag.register_workflow(wf) {
                        Ok(()) => Ok(ToolResult::text(json!({
                            "status": "edge_added"
                        }).to_string())),
                        Err(e) => Ok(ToolResult::error(format!("{}", e))),
                    }
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_validate" => {
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            match state.dag.get_workflow(wf_id) {
                Ok(wf) => {
                    let wf = wf.clone();
                    match state.dag.validate_dag(&wf) {
                        Ok(()) => Ok(ToolResult::text(json!({
                            "valid": true,
                            "step_count": wf.steps.len(),
                            "edge_count": wf.edges.len()
                        }).to_string())),
                        Err(e) => Ok(ToolResult::text(json!({
                            "valid": false,
                            "error": format!("{}", e)
                        }).to_string())),
                    }
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_visualize" => {
            let wf_id = params["workflow_id"].as_str().unwrap_or("");
            match state.dag.visualize_mermaid(wf_id) {
                Ok(diagram) => Ok(ToolResult::text(diagram)),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        _ => Ok(ToolResult::error(format!("Unknown DAG tool: {}", name))),
    }
}
