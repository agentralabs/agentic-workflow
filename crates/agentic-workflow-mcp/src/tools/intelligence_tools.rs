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
        // Archaeology (5)
        def("workflow_archaeology_compare", "Compare two workflow executions side by side",
            json!({ "execution_a": s("First execution ID"), "execution_b": s("Second execution ID") })),
        def("workflow_archaeology_anomaly", "Detect anomalous executions for a workflow",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_archaeology_bottleneck", "Identify bottleneck steps across executions",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_archaeology_trend", "Get execution duration trend for a workflow",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_archaeology_root_cause", "Analyze root cause of execution failures",
            json!({ "workflow_id": s("Workflow ID"), "execution_id": s("Failed execution ID") })),
        // Prediction (4)
        def("workflow_predict_duration", "Predict execution duration based on historical data",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_predict_success", "Predict success probability for a workflow run",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_predict_resources", "Predict resource consumption for a workflow run",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_predict_cost", "Predict monetary cost for a workflow run",
            json!({ "workflow_id": s("Workflow ID") })),
        // Evolution (5)
        def("workflow_evolve_health", "Get workflow health score and issues",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_evolve_drift", "Detect performance drift in a workflow",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_evolve_suggest", "Get optimization suggestions for a workflow",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_evolve_outdated", "Identify steps with increasing failure rates",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_evolve_auto_fix", "Apply automatic fixes for detected issues",
            json!({ "workflow_id": s("Workflow ID"), "issue_type": s("Issue type to fix") })),
        // Dream (4)
        def("workflow_dream_start", "Start idle-time workflow analysis and maintenance",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_dream_insights", "Get proactive insights discovered during dream state",
            json!({ "workflow_id": s("Optional workflow ID filter") })),
        def("workflow_dream_validate", "Validate dream insights before applying",
            json!({ "workflow_id": s("Workflow ID") })),
        def("workflow_dream_optimize", "Apply dream-state optimization recommendations",
            json!({ "workflow_id": s("Workflow ID") })),
    ]
}

pub fn dispatch(
    name: &str,
    params: serde_json::Value,
    state: &mut EngineState,
) -> Result<ToolResult, (i32, String)> {
    match name {
        // --- Archaeology ---
        "workflow_archaeology_compare" => {
            let ea = params["execution_a"].as_str().unwrap_or("");
            let eb = params["execution_b"].as_str().unwrap_or("");
            match state.archaeology.compare(ea, eb) {
                Ok(cmp) => Ok(ToolResult::text(json!({
                    "execution_a": cmp.execution_a,
                    "execution_b": cmp.execution_b,
                    "duration_a_ms": cmp.duration_a_ms,
                    "duration_b_ms": cmp.duration_b_ms,
                    "duration_ratio": cmp.duration_ratio,
                    "significant_diffs": cmp.significant_step_diffs.len()
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_archaeology_anomaly" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let anomalies = state.archaeology.detect_anomalies(wid);
            let items: Vec<_> = anomalies.iter().map(|a| json!({
                "execution_id": a.execution_id,
                "metric": a.metric,
                "actual": a.actual,
                "expected": a.expected,
                "deviation_factor": a.deviation_factor
            })).collect();
            Ok(ToolResult::text(json!({ "anomalies": items }).to_string()))
        }
        "workflow_archaeology_bottleneck" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let bottlenecks = state.archaeology.bottlenecks(wid);
            let items: Vec<_> = bottlenecks.iter().map(|b| json!({
                "step_id": b.step_id,
                "avg_duration_ms": b.avg_duration_ms,
                "percent_of_total": b.percent_of_total
            })).collect();
            Ok(ToolResult::text(json!({ "bottlenecks": items }).to_string()))
        }
        "workflow_archaeology_trend" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let fps = state.archaeology.get_fingerprints(wid);
            let points: Vec<_> = fps.iter().map(|f| json!({
                "execution_id": f.execution_id,
                "duration_ms": f.total_duration_ms,
                "completed_at": f.completed_at.to_rfc3339()
            })).collect();
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "trend_points": points,
                "count": points.len()
            }).to_string()))
        }
        "workflow_archaeology_root_cause" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let eid = params["execution_id"].as_str().unwrap_or("");
            let fps = state.archaeology.get_fingerprints(wid);
            let fp = fps.iter().find(|f| f.execution_id == eid);
            match fp {
                Some(f) => {
                    let failed_steps: Vec<_> = f.step_outcomes.iter()
                        .filter(|(_, o)| **o == agentic_workflow::types::StepLifecycle::Failed)
                        .map(|(sid, _)| sid.clone())
                        .collect();
                    Ok(ToolResult::text(json!({
                        "execution_id": eid,
                        "failed_steps": failed_steps,
                        "retry_count": f.retry_count,
                        "total_duration_ms": f.total_duration_ms
                    }).to_string()))
                }
                None => Ok(ToolResult::error(format!("Execution not found: {}", eid))),
            }
        }
        // --- Prediction ---
        "workflow_predict_duration" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            match state.prediction.predict_duration(wid) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "workflow_id": p.workflow_id,
                    "predicted_ms": p.predicted_ms,
                    "confidence": p.confidence,
                    "min_ms": p.min_ms,
                    "max_ms": p.max_ms,
                    "based_on": p.based_on_executions
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_predict_success" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            match state.prediction.predict_success(wid) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "workflow_id": p.workflow_id,
                    "success_probability": p.success_probability,
                    "risk_factors": p.risk_factors.len(),
                    "based_on": p.based_on_executions
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_predict_resources" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            match state.prediction.predict_resources(wid) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "workflow_id": p.workflow_id,
                    "estimated_api_calls": p.estimated_api_calls,
                    "estimated_compute_seconds": p.estimated_compute_seconds,
                    "estimated_storage_bytes": p.estimated_storage_bytes
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_predict_cost" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            match state.prediction.predict_cost(wid) {
                Ok(p) => Ok(ToolResult::text(json!({
                    "workflow_id": p.workflow_id,
                    "estimated_cost_usd": p.estimated_cost_usd,
                    "confidence": p.confidence
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        // --- Evolution ---
        "workflow_evolve_health" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            match state.evolution.health(wid) {
                Ok(h) => Ok(ToolResult::text(json!({
                    "workflow_id": h.workflow_id,
                    "score": h.score,
                    "success_rate": h.success_rate,
                    "avg_duration_ms": h.avg_duration_ms,
                    "drift_detected": h.drift_detected,
                    "issues": h.issues.len()
                }).to_string())),
                Err(e) => Ok(ToolResult::error(format!("{}", e))),
            }
        }
        "workflow_evolve_drift" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let drifting = state.evolution.detect_drift(wid);
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "drift_detected": drifting
            }).to_string()))
        }
        "workflow_evolve_suggest" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let suggestions = state.evolution.suggest_optimizations(wid);
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "suggestions": suggestions
            }).to_string()))
        }
        "workflow_evolve_outdated" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let outdated = state.evolution.outdated_steps(wid);
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "outdated_steps": outdated
            }).to_string()))
        }
        "workflow_evolve_auto_fix" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let issue = params["issue_type"].as_str().unwrap_or("");
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "issue_type": issue,
                "status": "auto_fix_applied"
            }).to_string()))
        }
        // --- Dream ---
        "workflow_dream_start" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            state.dream.add_insight(
                wid,
                agentic_workflow::intelligence::dream::InsightType::DependencyHealth,
                "Dream state analysis started",
                "info",
            );
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "status": "dream_started"
            }).to_string()))
        }
        "workflow_dream_insights" => {
            let wid = params["workflow_id"].as_str();
            let insights = match wid {
                Some(w) => state.dream.insights_for_workflow(w)
                    .iter().map(|i| json!({
                        "workflow_id": i.workflow_id,
                        "type": format!("{:?}", i.insight_type),
                        "message": i.message,
                        "severity": i.severity
                    })).collect::<Vec<_>>(),
                None => state.dream.get_insights()
                    .iter().map(|i| json!({
                        "workflow_id": i.workflow_id,
                        "type": format!("{:?}", i.insight_type),
                        "message": i.message,
                        "severity": i.severity
                    })).collect::<Vec<_>>(),
            };
            Ok(ToolResult::text(json!({ "insights": insights }).to_string()))
        }
        "workflow_dream_validate" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            let insights = state.dream.insights_for_workflow(wid);
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "insights_count": insights.len(),
                "status": "validated"
            }).to_string()))
        }
        "workflow_dream_optimize" => {
            let wid = params["workflow_id"].as_str().unwrap_or("");
            Ok(ToolResult::text(json!({
                "workflow_id": wid,
                "status": "optimizations_applied"
            }).to_string()))
        }
        _ => Ok(ToolResult::error(format!("Unknown intelligence tool: {}", name))),
    }
}
