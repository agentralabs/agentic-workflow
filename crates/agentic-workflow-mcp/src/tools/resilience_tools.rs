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
        // Retry (5)
        def("workflow_retry_configure", "Configure a retry policy with failure-classified profiles",
            json!({ "name": s("Policy name"), "max_attempts": i("Max retries"), "strategy": s("immediate, fixed, exponential, linear") })),
        def("workflow_retry_stats", "Get retry statistics for a step",
            json!({ "step_id": s("Step ID") })),
        def("workflow_retry_patterns", "Get learned retry patterns across all steps",
            json!({})),
        def("workflow_retry_budget", "Check remaining retry budget for a step",
            json!({ "policy_id": s("Policy ID"), "step_id": s("Step ID") })),
        def("workflow_retry_escalate", "Escalate a step that has exhausted retries",
            json!({ "step_id": s("Step ID"), "reason": s("Escalation reason") })),
        // Rollback (5)
        def("workflow_rollback_define", "Define a rollback action for a workflow step",
            json!({ "step_id": s("Step ID"), "action_type": s("command, api_call, not_possible"), "description": s("Description") })),
        def("workflow_rollback_execute", "Execute rollback for a failed execution",
            json!({ "execution_id": s("Execution ID"), "scope": s("full, from_step, selective"), "step_id": s("Step ID for from_step scope") })),
        def("workflow_rollback_preview", "Preview which steps would be rolled back",
            json!({ "scope": s("full, from_step, selective"), "step_ids": json!({ "type": "array", "items": { "type": "string" }, "description": "Completed step IDs" }) })),
        def("workflow_rollback_verify", "Verify rollback receipts for an execution",
            json!({ "execution_id": s("Execution ID") })),
        def("workflow_rollback_partial", "Execute partial rollback for specific steps",
            json!({ "execution_id": s("Execution ID"), "step_ids": json!({ "type": "array", "items": { "type": "string" }, "description": "Steps to rollback" }) })),
        // Circuit breaker (4)
        def("workflow_circuit_status", "Get circuit breaker status for all services",
            json!({})),
        def("workflow_circuit_reset", "Force reset a circuit breaker to closed state",
            json!({ "service_id": s("Service ID") })),
        def("workflow_circuit_preflight", "Run preflight check on services needed by a workflow",
            json!({ "workflow_id": s("Workflow ID"), "service_ids": json!({ "type": "array", "items": { "type": "string" }, "description": "Service IDs" }) })),
        def("workflow_circuit_queue", "Queue a workflow to run when a service recovers",
            json!({ "workflow_id": s("Workflow ID"), "execution_id": s("Execution ID"), "service_id": s("Service ID"), "priority": i("Priority") })),
        // Dead letter (5)
        def("workflow_dead_letter_list", "List all items in the dead letter queue",
            json!({})),
        def("workflow_dead_letter_summary", "Get a summary of dead letter items grouped by failure class",
            json!({})),
        def("workflow_dead_letter_retry", "Retry dead letter items matching a failure class",
            json!({ "failure_class": s("Failure class to retry") })),
        def("workflow_dead_letter_purge", "Purge expired items from the dead letter queue",
            json!({})),
        def("workflow_dead_letter_policy", "View or update the dead letter retention policy",
            json!({ "retention_days": i("Retention days"), "max_items": i("Max queue size") })),
        // Idempotency (5)
        def("workflow_idempotency_configure", "Configure idempotency for a workflow step",
            json!({ "step_id": s("Step ID"), "key_strategy": s("input_hash, expression, field_path"), "window": s("forever, duration, until_next") })),
        def("workflow_idempotency_check", "Check if an execution would be deduplicated",
            json!({ "step_id": s("Step ID"), "workflow_id": s("Workflow ID"), "input": json!({ "type": "object", "description": "Input data" }) })),
        def("workflow_idempotency_cache", "View cached idempotency entries",
            json!({ "step_id": s("Filter by step ID") })),
        def("workflow_idempotency_purge", "Purge expired idempotency cache entries",
            json!({})),
        def("workflow_idempotency_report", "Get deduplication statistics report",
            json!({})),
    ]
}

pub fn dispatch(
    name: &str,
    params: serde_json::Value,
    state: &mut EngineState,
) -> Result<ToolResult, (i32, String)> {
    match name {
        // --- Retry ---
        "workflow_retry_configure" => {
            let pname = params["name"].as_str().unwrap_or("default");
            match state.retry.configure_policy(pname, Vec::new(), None) {
                Ok(pid) => Ok(ToolResult::text(json!({ "policy_id": pid, "status": "configured" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_retry_stats" => {
            let sid = params["step_id"].as_str().unwrap_or("");
            match state.retry.get_stats(sid) {
                Some(s) => Ok(ToolResult::text(json!({
                    "step_id": s.step_id, "total_attempts": s.total_attempts,
                    "avg_delay_ms": s.avg_delay_ms
                }).to_string())),
                None => Ok(ToolResult::text(json!({ "step_id": sid, "total_attempts": 0 }).to_string())),
            }
        }
        "workflow_retry_patterns" => {
            let patterns = state.retry.get_patterns();
            let items: Vec<_> = patterns.iter().map(|p| json!({
                "step_id": p.step_id, "optimal_delay_ms": p.optimal_delay_ms,
                "recommendation": p.recommendation
            })).collect();
            Ok(ToolResult::text(json!({ "patterns": items }).to_string()))
        }
        "workflow_retry_budget" => {
            let pid = params["policy_id"].as_str().unwrap_or("");
            let sid = params["step_id"].as_str().unwrap_or("");
            match state.retry.within_budget(pid, sid) {
                Ok(within) => Ok(ToolResult::text(json!({ "within_budget": within }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_retry_escalate" => {
            let sid = params["step_id"].as_str().unwrap_or("");
            let reason = params["reason"].as_str().unwrap_or("retries exhausted");
            Ok(ToolResult::text(json!({
                "step_id": sid, "escalated": true, "reason": reason
            }).to_string()))
        }
        // --- Rollback ---
        "workflow_rollback_define" => {
            let sid = params["step_id"].as_str().unwrap_or("");
            let desc = params["description"].as_str().unwrap_or("");
            let action_type = match params["action_type"].as_str().unwrap_or("not_possible") {
                "command" => agentic_workflow::types::RollbackType::Command {
                    command: "rollback".to_string(), args: vec![],
                },
                _ => agentic_workflow::types::RollbackType::NotPossible {
                    reason: desc.to_string(),
                },
            };
            let action = agentic_workflow::types::RollbackAction {
                id: uuid::Uuid::new_v4().to_string(), step_id: sid.to_string(),
                action_type, description: desc.to_string(), verification: None,
            };
            match state.rollback.define_action(action) {
                Ok(()) => Ok(ToolResult::text(json!({ "step_id": sid, "status": "defined" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_rollback_execute" => {
            let eid = params["execution_id"].as_str().unwrap_or("");
            let scope = match params["scope"].as_str().unwrap_or("full") {
                "from_step" => agentic_workflow::types::RollbackScope::FromStep {
                    step_id: params["step_id"].as_str().unwrap_or("").to_string(),
                },
                "selective" => agentic_workflow::types::RollbackScope::Selective { step_ids: vec![] },
                _ => agentic_workflow::types::RollbackScope::Full,
            };
            let steps: Vec<String> = state.rollback.list_actions().iter().map(|a| a.step_id.clone()).collect();
            match state.rollback.execute_rollback(eid, scope, &steps) {
                Ok(r) => Ok(ToolResult::text(json!({
                    "execution_id": eid, "overall_success": r.overall_success,
                    "rolled_back": r.rolled_back_steps.len()
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_rollback_preview" => {
            let scope = match params["scope"].as_str().unwrap_or("full") {
                "selective" => agentic_workflow::types::RollbackScope::Selective { step_ids: vec![] },
                _ => agentic_workflow::types::RollbackScope::Full,
            };
            let step_ids: Vec<String> = params["step_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let preview = state.rollback.preview(&scope, &step_ids);
            Ok(ToolResult::text(json!({ "steps_to_rollback": preview }).to_string()))
        }
        "workflow_rollback_verify" => {
            let eid = params["execution_id"].as_str().unwrap_or("");
            let receipts = state.rollback.get_receipts(eid);
            let items: Vec<_> = receipts.iter().map(|r| json!({
                "overall_success": r.overall_success, "steps": r.rolled_back_steps.len()
            })).collect();
            Ok(ToolResult::text(json!({ "receipts": items }).to_string()))
        }
        "workflow_rollback_partial" => {
            let eid = params["execution_id"].as_str().unwrap_or("");
            let step_ids: Vec<String> = params["step_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let scope = agentic_workflow::types::RollbackScope::Selective { step_ids: step_ids.clone() };
            match state.rollback.execute_rollback(eid, scope, &step_ids) {
                Ok(r) => Ok(ToolResult::text(json!({
                    "execution_id": eid, "overall_success": r.overall_success,
                    "rolled_back": r.rolled_back_steps.len()
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        // --- Circuit breaker ---
        "workflow_circuit_status" => {
            let statuses = state.circuit.all_statuses();
            let items: Vec<_> = statuses.iter().map(|b| json!({
                "service_id": b.service_id, "state": format!("{:?}", b.state),
                "failure_count": b.failure_count
            })).collect();
            Ok(ToolResult::text(json!({ "circuit_breakers": items }).to_string()))
        }
        "workflow_circuit_reset" => {
            let sid = params["service_id"].as_str().unwrap_or("");
            match state.circuit.reset(sid) {
                Ok(()) => Ok(ToolResult::text(json!({ "service_id": sid, "state": "closed" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_circuit_preflight" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let sids: Vec<String> = params["service_ids"].as_array()
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            let result = state.circuit.preflight_check(wid, &sids);
            Ok(ToolResult::text(json!({
                "all_healthy": result.all_services_healthy,
                "services": result.service_states.len()
            }).to_string()))
        }
        "workflow_circuit_queue" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let eid = params["execution_id"].as_str().unwrap_or("");
            let sid = params["service_id"].as_str().unwrap_or("");
            let pri = params["priority"].as_u64().unwrap_or(0) as u32;
            state.circuit.queue_workflow(wid, eid, sid, pri);
            Ok(ToolResult::text(json!({ "status": "queued", "service_id": sid }).to_string()))
        }
        // --- Dead letter ---
        "workflow_dead_letter_list" => {
            let items = state.dead_letter.list_items();
            let list: Vec<_> = items.iter().map(|i| json!({
                "id": i.id, "workflow_id": i.workflow_id, "step_id": i.step_id,
                "failure_class": i.failure_class, "resurrectable": i.resurrectable
            })).collect();
            Ok(ToolResult::text(json!({ "items": list, "total": list.len() }).to_string()))
        }
        "workflow_dead_letter_summary" => {
            let summary = state.dead_letter.summary();
            Ok(ToolResult::text(json!({
                "total": summary.total_items, "auto_retryable": summary.auto_retryable,
                "needs_human": summary.needs_human
            }).to_string()))
        }
        "workflow_dead_letter_retry" => {
            let fc = params["failure_class"].as_str().unwrap_or("");
            let items = state.dead_letter.retryable_items(fc);
            Ok(ToolResult::text(json!({ "retryable_count": items.len(), "failure_class": fc }).to_string()))
        }
        "workflow_dead_letter_purge" => {
            let purged = state.dead_letter.purge_expired();
            Ok(ToolResult::text(json!({ "purged": purged }).to_string()))
        }
        "workflow_dead_letter_policy" => {
            if params["retention_days"].is_number() {
                let days = params["retention_days"].as_u64().unwrap_or(30) as u32;
                let max = params["max_items"].as_u64().map(|v| v as usize);
                let policy = agentic_workflow::types::DeadLetterPolicy {
                    retention_days: days, auto_resurrect_on_recovery: true,
                    max_items: max, alert_threshold: Some(100),
                };
                state.dead_letter.set_policy(policy);
                Ok(ToolResult::text(json!({ "status": "updated", "retention_days": days }).to_string()))
            } else {
                let p = state.dead_letter.get_policy();
                Ok(ToolResult::text(json!({
                    "retention_days": p.retention_days, "max_items": p.max_items,
                    "auto_resurrect": p.auto_resurrect_on_recovery
                }).to_string()))
            }
        }
        // --- Idempotency ---
        "workflow_idempotency_configure" => {
            let sid = params["step_id"].as_str().unwrap_or("");
            let key_strat = match params["key_strategy"].as_str().unwrap_or("input_hash") {
                "expression" => agentic_workflow::types::idempotency::KeyStrategy::Expression("default".to_string()),
                "field_path" => agentic_workflow::types::idempotency::KeyStrategy::FieldPath("/id".to_string()),
                _ => agentic_workflow::types::idempotency::KeyStrategy::InputHash,
            };
            let window = match params["window"].as_str().unwrap_or("forever") {
                "duration" => agentic_workflow::types::idempotency::IdempotencyWindow::Duration { ms: 3600000 },
                "until_next" => agentic_workflow::types::idempotency::IdempotencyWindow::UntilNextExecution,
                _ => agentic_workflow::types::idempotency::IdempotencyWindow::Forever,
            };
            let conflict = agentic_workflow::types::idempotency::ConflictResolution::ReturnCached;
            match state.idempotency.configure(sid, key_strat, window, conflict) {
                Ok(()) => Ok(ToolResult::text(json!({ "step_id": sid, "status": "configured" }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_idempotency_check" => {
            let sid = params["step_id"].as_str().unwrap_or("");
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let input = &params["input"];
            match state.idempotency.compute_key(sid, wid, input) {
                Ok(key) => {
                    let cached = state.idempotency.check(&key).is_some();
                    Ok(ToolResult::text(json!({ "key": key, "would_deduplicate": cached }).to_string()))
                }
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_idempotency_cache" => {
            Ok(ToolResult::text(json!({ "status": "cache_listed" }).to_string()))
        }
        "workflow_idempotency_purge" => {
            let purged = state.idempotency.purge_expired();
            Ok(ToolResult::text(json!({ "purged": purged }).to_string()))
        }
        "workflow_idempotency_report" => {
            let report = state.idempotency.report();
            Ok(ToolResult::text(json!({
                "total_entries": report.total_entries,
                "deduplicated_count": report.deduplicated_count,
                "cache_hit_rate": report.cache_hit_rate
            }).to_string()))
        }
        _ => Ok(ToolResult::error(format!("Unknown resilience tool: {}", name))),
    }
}
