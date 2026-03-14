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

pub fn definitions() -> Vec<ToolDefinition> {
    vec![
        // Approval (6)
        def("workflow_approve_gate", "Define an approval gate for a workflow step",
            json!({ "gate_id": s("Gate ID"), "step_id": s("Step ID"), "workflow_id": s("Workflow ID"), "approvers": json!({ "type": "array", "items": { "type": "string" }, "description": "Approver identities" }) })),
        def("workflow_approve_pending", "List all pending approval requests",
            json!({})),
        def("workflow_approve_decide", "Approve or reject a pending approval",
            json!({ "pending_id": s("Pending approval ID"), "decision": s("approved or rejected"), "decided_by": s("Decider identity"), "reason": s("Decision reason") })),
        def("workflow_approve_escalate", "Escalate a pending approval to the next approver",
            json!({ "pending_id": s("Pending approval ID") })),
        def("workflow_approve_delegate", "Delegate approval authority to another person",
            json!({ "pending_id": s("Pending approval ID"), "delegate_to": s("Delegate identity") })),
        def("workflow_approve_audit", "Get approval audit trail for a gate",
            json!({ "gate_id": s("Gate ID") })),
        // Audit (5)
        def("workflow_audit_query", "Query the audit trail with filters",
            json!({ "workflow_id": s("Filter by workflow ID"), "execution_id": s("Filter by execution ID"), "actor": s("Filter by actor"), "resource": s("Filter by resource") })),
        def("workflow_audit_timeline", "Get chronological timeline of audit events",
            json!({ "execution_id": s("Execution ID"), "limit": json!({ "type": "integer", "description": "Max events to return" }) })),
        def("workflow_audit_impact", "Analyze all workflows that touched a resource",
            json!({ "resource": s("Resource identifier") })),
        def("workflow_audit_export", "Export audit trail as JSON",
            json!({ "workflow_id": s("Filter by workflow ID") })),
        def("workflow_audit_retention", "View or update audit retention policy",
            json!({ "retain_days": json!({ "type": "integer", "description": "Retention period in days" }) })),
        // Variables (5)
        def("workflow_var_set", "Set a variable in a workflow scope",
            json!({ "scope_id": s("Scope ID"), "name": s("Variable name"), "value": json!({ "description": "Variable value" }), "type": s("string, integer, boolean, json"), "set_by": s("Who set it") })),
        def("workflow_var_get", "Get a variable value respecting scope hierarchy",
            json!({ "scope_id": s("Scope ID"), "name": s("Variable name") })),
        def("workflow_var_list", "List all variables in a scope",
            json!({ "scope_id": s("Scope ID") })),
        def("workflow_var_promote", "Promote a variable from child scope to parent scope",
            json!({ "scope_id": s("Child scope ID"), "name": s("Variable name") })),
        def("workflow_var_type_check", "Type-check all variables across scopes",
            json!({})),
    ]
}

pub fn dispatch(
    name: &str,
    params: serde_json::Value,
    state: &mut EngineState,
) -> Result<ToolResult, (i32, String)> {
    match name {
        // --- Approval ---
        "workflow_approve_gate" => {
            let gid = params["gate_id"].as_str().unwrap_or("");
            let sid = params["step_id"].as_str().unwrap_or("");
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let approvers: Vec<agentic_workflow::types::Approver> = params["approvers"]
                .as_array()
                .map(|a| a.iter().enumerate().map(|(i, v)| {
                    agentic_workflow::types::Approver {
                        identity: v.as_str().unwrap_or("unknown").to_string(),
                        role: None,
                        priority: i as u32 + 1,
                    }
                }).collect())
                .unwrap_or_default();
            let gate = agentic_workflow::types::ApprovalGate {
                id: gid.to_string(), step_id: sid.to_string(),
                workflow_id: wid.to_string(), approver_chain: approvers,
                condition: None, timeout: None, delegation: None,
            };
            match state.approval.define_gate(gate) {
                Ok(()) => Ok(ToolResult::text(json!({ "gate_id": gid, "status": "defined" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_approve_pending" => {
            let pending = state.approval.list_pending();
            let items: Vec<_> = pending.iter().map(|(id, p)| json!({
                "pending_id": id, "gate_id": p.gate_id,
                "execution_id": p.execution_id, "current_approver": p.current_approver
            })).collect();
            Ok(ToolResult::text(json!({ "pending": items }).to_string()))
        }
        "workflow_approve_decide" => {
            let pid = params["pending_id"].as_str().unwrap_or("");
            let decision = match params["decision"].as_str().unwrap_or("rejected") {
                "approved" => agentic_workflow::types::ApprovalDecision::Approved,
                _ => agentic_workflow::types::ApprovalDecision::Denied,
            };
            let by = params["decided_by"].as_str().unwrap_or("system");
            let reason = params["reason"].as_str().map(String::from);
            match state.approval.decide(pid, decision, by, reason) {
                Ok(r) => Ok(ToolResult::text(json!({
                    "gate_id": r.gate_id, "decision": format!("{:?}", r.decision),
                    "decided_by": r.decided_by
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_approve_escalate" => {
            let pid = params["pending_id"].as_str().unwrap_or("");
            match state.approval.escalate(pid) {
                Ok(()) => Ok(ToolResult::text(json!({ "pending_id": pid, "status": "escalated" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_approve_delegate" => {
            let pid = params["pending_id"].as_str().unwrap_or("");
            let delegate = params["delegate_to"].as_str().unwrap_or("");
            Ok(ToolResult::text(json!({
                "pending_id": pid, "delegated_to": delegate, "status": "delegated"
            }).to_string()))
        }
        "workflow_approve_audit" => {
            let gid = params["gate_id"].as_str();
            let receipts = state.approval.get_receipts(gid);
            let items: Vec<_> = receipts.iter().map(|r| json!({
                "gate_id": r.gate_id, "decision": format!("{:?}", r.decision),
                "decided_by": r.decided_by, "decided_at": r.decided_at.to_rfc3339()
            })).collect();
            Ok(ToolResult::text(json!({ "audit_trail": items }).to_string()))
        }
        // --- Audit ---
        "workflow_audit_query" => {
            let q = agentic_workflow::types::AuditQuery {
                workflow_id: params["workflow_id"].as_str().map(String::from),
                execution_id: params["execution_id"].as_str().map(String::from),
                event_types: None,
                actor: params["actor"].as_str().map(String::from),
                resource: params["resource"].as_str().map(String::from),
                from: None, to: None, limit: Some(100),
            };
            let events = state.audit.query(&q);
            let items: Vec<_> = events.iter().map(|e| json!({
                "event_id": e.event_id, "event_type": format!("{:?}", e.event_type),
                "actor": e.actor, "timestamp": e.timestamp.to_rfc3339()
            })).collect();
            Ok(ToolResult::text(json!({ "events": items, "count": items.len() }).to_string()))
        }
        "workflow_audit_timeline" => {
            let eid = params["execution_id"].as_str();
            let limit = params["limit"].as_u64().unwrap_or(50) as usize;
            let events = state.audit.timeline(eid, limit);
            let items: Vec<_> = events.iter().map(|e| json!({
                "event_id": e.event_id, "event_type": format!("{:?}", e.event_type),
                "timestamp": e.timestamp.to_rfc3339()
            })).collect();
            Ok(ToolResult::text(json!({ "timeline": items }).to_string()))
        }
        "workflow_audit_impact" => {
            let resource = params["resource"].as_str().unwrap_or("");
            let impact = state.audit.impact_analysis(resource);
            Ok(ToolResult::text(json!({
                "resource": impact.resource, "event_count": impact.event_count,
                "workflow_ids": impact.workflow_ids
            }).to_string()))
        }
        "workflow_audit_export" => {
            let q = agentic_workflow::types::AuditQuery {
                workflow_id: params["workflow_id"].as_str().map(String::from),
                execution_id: None, event_types: None, actor: None,
                resource: None, from: None, to: None, limit: None,
            };
            match state.audit.export(&q) {
                Ok(exported) => Ok(ToolResult::text(exported)),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_audit_retention" => {
            if let Some(days) = params["retain_days"].as_u64() {
                let retention = agentic_workflow::types::AuditRetention {
                    retain_days: days as u32,
                    compliance_preset: None,
                    archive_after_days: Some(365),
                };
                state.audit.set_retention(retention);
                Ok(ToolResult::text(json!({ "status": "updated", "retain_days": days }).to_string()))
            } else {
                let r = state.audit.get_retention();
                Ok(ToolResult::text(json!({
                    "retain_days": r.retain_days,
                    "archive_after_days": r.archive_after_days
                }).to_string()))
            }
        }
        // --- Variables ---
        "workflow_var_set" => {
            let scope_id = params["scope_id"].as_str().unwrap_or("");
            let vname = params["name"].as_str().unwrap_or("");
            let value = params["value"].clone();
            let var_type = match params["type"].as_str().unwrap_or("string") {
                "integer" => agentic_workflow::types::VariableType::Integer,
                "boolean" => agentic_workflow::types::VariableType::Boolean,
                "json" | "object" => agentic_workflow::types::VariableType::Object,
                _ => agentic_workflow::types::VariableType::String,
            };
            let set_by = params["set_by"].as_str().unwrap_or("system");
            match state.variable.set(scope_id, vname, value, var_type, set_by) {
                Ok(()) => Ok(ToolResult::text(json!({ "name": vname, "status": "set" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_var_get" => {
            let scope_id = params["scope_id"].as_str().unwrap_or("");
            let vname = params["name"].as_str().unwrap_or("");
            match state.variable.get(scope_id, vname) {
                Ok(v) => Ok(ToolResult::text(json!({
                    "name": v.name, "value": v.value, "type": format!("{:?}", v.var_type)
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_var_list" => {
            let scope_id = params["scope_id"].as_str().unwrap_or("");
            match state.variable.list(scope_id) {
                Ok(vars) => {
                    let items: Vec<_> = vars.iter().map(|v| json!({
                        "name": v.name, "value": v.value, "immutable": v.immutable
                    })).collect();
                    Ok(ToolResult::text(json!({ "variables": items }).to_string()))
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_var_promote" => {
            let scope_id = params["scope_id"].as_str().unwrap_or("");
            let vname = params["name"].as_str().unwrap_or("");
            match state.variable.promote(scope_id, vname) {
                Ok(()) => Ok(ToolResult::text(json!({ "name": vname, "status": "promoted" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_var_type_check" => {
            let result = state.variable.type_check();
            Ok(ToolResult::text(json!({
                "valid": result.valid,
                "error_count": result.errors.len()
            }).to_string()))
        }
        _ => Ok(ToolResult::error(format!("Unknown governance tool: {}", name))),
    }
}
