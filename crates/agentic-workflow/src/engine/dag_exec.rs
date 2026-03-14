use std::collections::HashMap;
use std::time::Instant;

use chrono::Utc;

use crate::types::{
    Edge, EdgeType, ExecutionContext, ExecutionEvent, ExecutionEventType,
    ExecutionFingerprint, ExecutionStatus, StepLifecycle, StepState, StepType,
    Workflow, WorkflowError, WorkflowResult,
};

/// Step execution result with output and timing.
#[derive(Debug, Clone)]
pub struct StepExecutionResult {
    pub step_id: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Execute a single step and capture its result.
pub fn execute_step(
    step_id: &str,
    step_type: &StepType,
    inputs: &HashMap<String, serde_json::Value>,
    timeout_ms: Option<u64>,
) -> StepExecutionResult {
    let start = Instant::now();

    let result = match step_type {
        StepType::Noop => Ok(serde_json::json!({"status": "noop"})),

        StepType::Expression { expression } => {
            Ok(serde_json::json!({
                "expression": expression,
                "evaluated": true,
                "inputs": inputs,
            }))
        }

        StepType::Command { command, args } => {
            // Build command description (don't execute in library — that's the runner's job)
            Ok(serde_json::json!({
                "command": command,
                "args": args,
                "status": "prepared",
                "note": "Execution delegated to step runner"
            }))
        }

        StepType::McpTool { sister, tool, params } => {
            Ok(serde_json::json!({
                "sister": sister,
                "tool": tool,
                "params": params,
                "status": "prepared",
                "note": "Execution delegated to MCP dispatcher"
            }))
        }

        StepType::HttpRequest { method, url, headers, body } => {
            Ok(serde_json::json!({
                "method": method,
                "url": url,
                "headers": headers,
                "body": body,
                "status": "prepared",
                "note": "Execution delegated to HTTP runner"
            }))
        }

        StepType::SubWorkflow { workflow_id } => {
            Ok(serde_json::json!({
                "sub_workflow_id": workflow_id,
                "status": "prepared",
                "note": "Execution delegated to sub-workflow runner"
            }))
        }

        StepType::FanOut { destinations, completion_policy } => {
            Ok(serde_json::json!({
                "destinations": destinations.len(),
                "completion_policy": format!("{:?}", completion_policy),
                "status": "prepared"
            }))
        }

        StepType::ApprovalGate { approvers, timeout_ms } => {
            Ok(serde_json::json!({
                "approvers": approvers,
                "timeout_ms": timeout_ms,
                "status": "waiting_approval"
            }))
        }
    };

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // Check timeout
    if let Some(timeout) = timeout_ms {
        if duration_ms > timeout {
            return StepExecutionResult {
                step_id: step_id.to_string(),
                success: false,
                output: None,
                error: Some(format!("Step timed out after {}ms (limit: {}ms)", duration_ms, timeout)),
                duration_ms,
            };
        }
    }

    match result {
        Ok(output) => StepExecutionResult {
            step_id: step_id.to_string(),
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
        },
        Err(e) => StepExecutionResult {
            step_id: step_id.to_string(),
            success: false,
            output: None,
            error: Some(e),
            duration_ms,
        },
    }
}

/// Apply a step execution result to the execution context.
pub fn apply_step_result(
    ctx: &mut ExecutionContext,
    result: &StepExecutionResult,
) {
    if let Some(state) = ctx.step_states.get_mut(&result.step_id) {
        state.lifecycle = if result.success {
            StepLifecycle::Success
        } else {
            StepLifecycle::Failed
        };
        state.completed_at = Some(Utc::now());
        state.duration_ms = Some(result.duration_ms);
        state.output = result.output.clone();
        state.error = result.error.clone();
        state.attempt += 1;
    }
}

/// Get the next ready steps (all dependencies satisfied).
pub fn next_ready_steps(
    workflow: &Workflow,
    ctx: &ExecutionContext,
) -> Vec<String> {
    let mut ready = Vec::new();

    for step in &workflow.steps {
        let state = ctx.step_states.get(&step.id);
        if state.map_or(true, |s| s.lifecycle != StepLifecycle::Pending) {
            continue; // Not pending — skip
        }

        // Check all incoming edges are satisfied
        let deps_satisfied = workflow
            .edges
            .iter()
            .filter(|e| e.to == step.id)
            .all(|e| {
                ctx.step_states
                    .get(&e.from)
                    .map_or(false, |s| {
                        s.lifecycle == StepLifecycle::Success
                            || s.lifecycle == StepLifecycle::Skipped
                    })
            });

        if deps_satisfied {
            ready.push(step.id.clone());
        }
    }

    ready
}

/// Check if execution is complete (all steps done).
pub fn is_execution_complete(ctx: &ExecutionContext) -> bool {
    ctx.step_states.values().all(|s| {
        matches!(
            s.lifecycle,
            StepLifecycle::Success
                | StepLifecycle::Failed
                | StepLifecycle::Skipped
                | StepLifecycle::Cancelled
        )
    })
}

/// Determine overall execution status from step states.
pub fn compute_execution_status(ctx: &ExecutionContext) -> ExecutionStatus {
    if !is_execution_complete(ctx) {
        return ctx.status.clone();
    }

    let has_failures = ctx
        .step_states
        .values()
        .any(|s| s.lifecycle == StepLifecycle::Failed);

    if has_failures {
        ExecutionStatus::Failed {
            error: "One or more steps failed".to_string(),
        }
    } else {
        ExecutionStatus::Succeeded
    }
}

/// Build an execution fingerprint from a completed execution.
pub fn build_fingerprint(ctx: &ExecutionContext) -> ExecutionFingerprint {
    let step_durations: HashMap<String, u64> = ctx
        .step_states
        .iter()
        .filter_map(|(id, s)| s.duration_ms.map(|d| (id.clone(), d)))
        .collect();

    let step_outcomes: HashMap<String, StepLifecycle> = ctx
        .step_states
        .iter()
        .map(|(id, s)| (id.clone(), s.lifecycle.clone()))
        .collect();

    let total_duration: u64 = step_durations.values().sum();
    let retry_count: u32 = ctx
        .step_states
        .values()
        .map(|s| s.attempt.saturating_sub(1))
        .sum();

    ExecutionFingerprint {
        execution_id: ctx.execution_id.clone(),
        workflow_id: ctx.workflow_id.clone(),
        total_duration_ms: total_duration,
        step_durations,
        step_outcomes,
        retry_count,
        completed_at: ctx.completed_at.unwrap_or_else(Utc::now),
    }
}

/// Generate execution events for observability.
pub fn emit_step_event(
    ctx: &ExecutionContext,
    step_id: &str,
    event_type: ExecutionEventType,
) -> ExecutionEvent {
    ExecutionEvent {
        execution_id: ctx.execution_id.clone(),
        step_id: Some(step_id.to_string()),
        event_type,
        timestamp: Utc::now(),
        data: None,
    }
}

/// Pass outputs from completed steps to dependent steps as inputs.
pub fn propagate_outputs(
    workflow: &Workflow,
    ctx: &ExecutionContext,
    target_step_id: &str,
) -> HashMap<String, serde_json::Value> {
    let mut inputs = HashMap::new();

    for edge in &workflow.edges {
        if edge.to != target_step_id {
            continue;
        }

        if let Some(state) = ctx.step_states.get(&edge.from) {
            if let Some(output) = &state.output {
                inputs.insert(edge.from.clone(), output.clone());
            }
        }
    }

    inputs
}

// Tests for dag_exec are in tests/phase8_persistence.rs
