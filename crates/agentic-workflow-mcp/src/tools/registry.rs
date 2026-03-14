use std::sync::Arc;
use tokio::sync::Mutex;

use agentic_workflow::engine::*;
use agentic_workflow::resilience::*;
use agentic_workflow::governance::*;
use agentic_workflow::template::*;
use agentic_workflow::intelligence::*;

use crate::types::{ToolDefinition, ToolResult, TOOL_NOT_FOUND};

use super::{
    dag_tools, execution_tools, schedule_tools, trigger_tools,
    resilience_tools, governance_tools, processing_tools,
    state_tools, template_tools, intelligence_tools,
};

/// Shared state for all engines.
pub struct EngineState {
    pub dag: DagEngine,
    pub scheduler: SchedulerEngine,
    pub trigger: TriggerEngine,
    pub batch: BatchEngine,
    pub stream: StreamEngine,
    pub fanout: FanOutEngine,
    pub fsm: FsmEngine,
    pub retry: RetryEngine,
    pub rollback: RollbackEngine,
    pub circuit: CircuitBreakerEngine,
    pub dead_letter: DeadLetterEngine,
    pub idempotency: IdempotencyEngine,
    pub approval: ApprovalEngine,
    pub audit: AuditEngine,
    pub variable: VariableEngine,
    pub template: TemplateEngine,
    pub natural: NaturalLanguageEngine,
    pub composer: CompositionEngine,
    pub archaeology: ArchaeologyEngine,
    pub prediction: PredictionEngine,
    pub evolution: EvolutionEngine,
    pub dream: DreamEngine,
    pub collective: CollectiveEngine,
}

impl EngineState {
    pub fn new() -> Self {
        Self {
            dag: DagEngine::new(),
            scheduler: SchedulerEngine::new(),
            trigger: TriggerEngine::new(),
            batch: BatchEngine::new(),
            stream: StreamEngine::new(),
            fanout: FanOutEngine::new(),
            fsm: FsmEngine::new(),
            retry: RetryEngine::new(),
            rollback: RollbackEngine::new(),
            circuit: CircuitBreakerEngine::new(),
            dead_letter: DeadLetterEngine::new(),
            idempotency: IdempotencyEngine::new(),
            approval: ApprovalEngine::new(),
            audit: AuditEngine::new(),
            variable: VariableEngine::new(),
            template: TemplateEngine::new(),
            natural: NaturalLanguageEngine::new(),
            composer: CompositionEngine::new(),
            archaeology: ArchaeologyEngine::new(),
            prediction: PredictionEngine::new(),
            evolution: EvolutionEngine::new(),
            dream: DreamEngine::new(),
            collective: CollectiveEngine::new(),
        }
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP Tool Registry — provides all 124 tools.
pub struct ToolRegistry {
    state: Arc<Mutex<EngineState>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(EngineState::new())),
        }
    }

    /// Get all tool definitions.
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        let mut tools = Vec::new();
        tools.extend(dag_tools::definitions());
        tools.extend(execution_tools::definitions());
        tools.extend(schedule_tools::definitions());
        tools.extend(trigger_tools::definitions());
        tools.extend(resilience_tools::definitions());
        tools.extend(governance_tools::definitions());
        tools.extend(processing_tools::definitions());
        tools.extend(state_tools::definitions());
        tools.extend(template_tools::definitions());
        tools.extend(intelligence_tools::definitions());
        tools
    }

    /// Dispatch a tool call.
    pub async fn call_tool(
        &self,
        name: &str,
        params: serde_json::Value,
    ) -> Result<ToolResult, (i32, String)> {
        let mut state = self.state.lock().await;

        match name {
            // DAG tools (Invention 1)
            n if n.starts_with("workflow_create") || n.starts_with("workflow_step_")
                || n.starts_with("workflow_edge_") || n == "workflow_validate"
                || n == "workflow_visualize" => {
                dag_tools::dispatch(n, params, &mut state)
            }

            // Execution tools (Invention 2)
            n if n == "workflow_run" || n == "workflow_status" || n == "workflow_progress"
                || n == "workflow_observe" || n == "workflow_pause" || n == "workflow_resume"
                || n == "workflow_cancel" || n == "workflow_intervene" => {
                execution_tools::dispatch(n, params, &mut state)
            }

            // Schedule tools (Invention 3)
            n if n.starts_with("workflow_schedule") => {
                schedule_tools::dispatch(n, params, &mut state)
            }

            // Trigger tools (Invention 4)
            n if n.starts_with("workflow_trigger") => {
                trigger_tools::dispatch(n, params, &mut state)
            }

            // Resilience tools (Inventions 5-8, 14)
            n if n.starts_with("workflow_retry") || n.starts_with("workflow_rollback")
                || n.starts_with("workflow_circuit") || n.starts_with("workflow_dead_letter")
                || n.starts_with("workflow_idempotency") => {
                resilience_tools::dispatch(n, params, &mut state)
            }

            // Governance tools (Inventions 12-13, 16)
            n if n.starts_with("workflow_approve") || n.starts_with("workflow_audit")
                || n.starts_with("workflow_var") => {
                governance_tools::dispatch(n, params, &mut state)
            }

            // Processing tools (Inventions 9-11)
            n if n.starts_with("workflow_batch") || n.starts_with("workflow_stream")
                || n.starts_with("workflow_fanout") => {
                processing_tools::dispatch(n, params, &mut state)
            }

            // State tools (Invention 15)
            n if n.starts_with("workflow_fsm") => {
                state_tools::dispatch(n, params, &mut state)
            }

            // Template tools (Inventions 17-18, 23-24)
            n if n.starts_with("workflow_template") || n.starts_with("workflow_natural")
                || n.starts_with("workflow_compose") || n.starts_with("workflow_collective") => {
                template_tools::dispatch(n, params, &mut state)
            }

            // Intelligence tools (Inventions 19-22)
            n if n.starts_with("workflow_archaeology") || n.starts_with("workflow_predict")
                || n.starts_with("workflow_evolve") || n.starts_with("workflow_dream") => {
                intelligence_tools::dispatch(n, params, &mut state)
            }

            _ => Err((TOOL_NOT_FOUND, format!("Unknown tool: {}", name))),
        }
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
