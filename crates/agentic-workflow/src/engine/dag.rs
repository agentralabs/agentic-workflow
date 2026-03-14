use std::collections::{HashMap, HashSet, VecDeque};

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    Edge, EdgeType, ExecutionContext, ExecutionEvent, ExecutionEventType,
    ExecutionProgress, ExecutionStatus, StepLifecycle, StepState, Workflow,
    WorkflowError, WorkflowResult,
};

/// DAG execution engine — validates and runs workflow graphs.
pub struct DagEngine {
    workflows: HashMap<String, Workflow>,
    executions: HashMap<String, ExecutionContext>,
}

impl DagEngine {
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            executions: HashMap::new(),
        }
    }

    /// Register a workflow definition.
    pub fn register_workflow(&mut self, workflow: Workflow) -> WorkflowResult<()> {
        self.validate_dag(&workflow)?;
        self.workflows.insert(workflow.id.clone(), workflow);
        Ok(())
    }

    /// Get a workflow by ID.
    pub fn get_workflow(&self, id: &str) -> WorkflowResult<&Workflow> {
        self.workflows
            .get(id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound(id.to_string()))
    }

    /// Remove a workflow.
    pub fn remove_workflow(&mut self, id: &str) -> WorkflowResult<Workflow> {
        self.workflows
            .remove(id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound(id.to_string()))
    }

    /// List all registered workflows.
    pub fn list_workflows(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    /// Validate the DAG — check for cycles and unsatisfied dependencies.
    pub fn validate_dag(&self, workflow: &Workflow) -> WorkflowResult<()> {
        let step_ids: HashSet<&str> = workflow.steps.iter().map(|s| s.id.as_str()).collect();

        // Check all edges reference valid steps
        for edge in &workflow.edges {
            if !step_ids.contains(edge.from.as_str()) {
                return Err(WorkflowError::StepNotFound(edge.from.clone()));
            }
            if !step_ids.contains(edge.to.as_str()) {
                return Err(WorkflowError::StepNotFound(edge.to.clone()));
            }
        }

        // Topological sort to detect cycles
        self.topological_sort(workflow)?;
        Ok(())
    }

    /// Topological sort of steps — returns execution order or error if cycle.
    pub fn topological_sort(&self, workflow: &Workflow) -> WorkflowResult<Vec<String>> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for step in &workflow.steps {
            in_degree.entry(step.id.as_str()).or_insert(0);
            adjacency.entry(step.id.as_str()).or_default();
        }

        for edge in &workflow.edges {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
            adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::new();

        while let Some(node) = queue.pop_front() {
            order.push(node.to_string());
            if let Some(neighbors) = adjacency.get(node) {
                for &neighbor in neighbors {
                    if let Some(deg) = in_degree.get_mut(neighbor) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        if order.len() != workflow.steps.len() {
            return Err(WorkflowError::CycleDetected(
                "DAG contains a cycle".to_string(),
            ));
        }

        Ok(order)
    }

    /// Start a new execution of a workflow.
    pub fn start_execution(&mut self, workflow_id: &str) -> WorkflowResult<String> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound(workflow_id.to_string()))?
            .clone();

        let execution_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let mut step_states = HashMap::new();
        for step in &workflow.steps {
            step_states.insert(
                step.id.clone(),
                StepState {
                    step_id: step.id.clone(),
                    lifecycle: StepLifecycle::Pending,
                    attempt: 0,
                    started_at: None,
                    completed_at: None,
                    duration_ms: None,
                    output: None,
                    error: None,
                },
            );
        }

        let ctx = ExecutionContext {
            execution_id: execution_id.clone(),
            workflow_id: workflow_id.to_string(),
            status: ExecutionStatus::Running,
            step_states,
            variables: HashMap::new(),
            started_at: now,
            completed_at: None,
            trigger_info: None,
            metadata: HashMap::new(),
        };

        self.executions.insert(execution_id.clone(), ctx);
        Ok(execution_id)
    }

    /// Get execution progress.
    pub fn get_progress(&self, execution_id: &str) -> WorkflowResult<ExecutionProgress> {
        let ctx = self
            .executions
            .get(execution_id)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(execution_id.to_string()))?;

        let total = ctx.step_states.len();
        let completed = ctx.step_states.values().filter(|s| s.lifecycle == StepLifecycle::Success).count();
        let failed = ctx.step_states.values().filter(|s| s.lifecycle == StepLifecycle::Failed).count();
        let skipped = ctx.step_states.values().filter(|s| s.lifecycle == StepLifecycle::Skipped).count();
        let running = ctx.step_states.values().filter(|s| s.lifecycle == StepLifecycle::Running).count();
        let pending = ctx.step_states.values().filter(|s| s.lifecycle == StepLifecycle::Pending || s.lifecycle == StepLifecycle::Queued).count();

        let percent = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(ExecutionProgress {
            execution_id: execution_id.to_string(),
            total_steps: total,
            completed_steps: completed,
            failed_steps: failed,
            skipped_steps: skipped,
            running_steps: running,
            pending_steps: pending,
            estimated_remaining_ms: None,
            percent_complete: percent,
        })
    }

    /// Pause a running execution.
    pub fn pause_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let ctx = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(execution_id.to_string()))?;

        if ctx.status != ExecutionStatus::Running {
            return Err(WorkflowError::Internal(format!(
                "Cannot pause execution in state {:?}",
                ctx.status
            )));
        }

        ctx.status = ExecutionStatus::Paused;
        Ok(())
    }

    /// Resume a paused execution.
    pub fn resume_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let ctx = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(execution_id.to_string()))?;

        if ctx.status != ExecutionStatus::Paused {
            return Err(WorkflowError::ExecutionNotPaused(execution_id.to_string()));
        }

        ctx.status = ExecutionStatus::Running;
        Ok(())
    }

    /// Cancel a running execution.
    pub fn cancel_execution(&mut self, execution_id: &str) -> WorkflowResult<()> {
        let ctx = self
            .executions
            .get_mut(execution_id)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(execution_id.to_string()))?;

        ctx.status = ExecutionStatus::Cancelled;
        ctx.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Get execution context.
    pub fn get_execution(&self, execution_id: &str) -> WorkflowResult<&ExecutionContext> {
        self.executions
            .get(execution_id)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(execution_id.to_string()))
    }

    /// Generate a Mermaid diagram for a workflow.
    pub fn visualize_mermaid(&self, workflow_id: &str) -> WorkflowResult<String> {
        let wf = self.get_workflow(workflow_id)?;
        let mut lines = vec!["graph TD".to_string()];

        for step in &wf.steps {
            lines.push(format!("    {}[{}]", step.id, step.name));
        }

        for edge in &wf.edges {
            let label = match &edge.edge_type {
                EdgeType::Sequence => "".to_string(),
                EdgeType::Parallel => "|parallel|".to_string(),
                EdgeType::Conditional { expression } => format!("|{}|", expression),
                EdgeType::Loop { .. } => "|loop|".to_string(),
            };
            lines.push(format!("    {} -->{}  {}", edge.from, label, edge.to));
        }

        Ok(lines.join("\n"))
    }
}

impl Default for DagEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{StepNode, StepType};

    #[test]
    fn test_create_and_validate_workflow() {
        let mut engine = DagEngine::new();
        let mut wf = Workflow::new("test-wf", "A test workflow");

        let step1 = StepNode::new("Step 1", StepType::Noop);
        let step2 = StepNode::new("Step 2", StepType::Noop);
        let s1_id = step1.id.clone();
        let s2_id = step2.id.clone();

        wf.add_step(step1);
        wf.add_step(step2);
        wf.add_edge(Edge {
            from: s1_id,
            to: s2_id,
            edge_type: EdgeType::Sequence,
        });

        assert!(engine.register_workflow(wf).is_ok());
    }

    #[test]
    fn test_cycle_detection() {
        let engine = DagEngine::new();
        let mut wf = Workflow::new("cyclic", "Cyclic workflow");

        let s1 = StepNode::new("A", StepType::Noop);
        let s2 = StepNode::new("B", StepType::Noop);
        let s1_id = s1.id.clone();
        let s2_id = s2.id.clone();

        wf.add_step(s1);
        wf.add_step(s2);
        wf.add_edge(Edge {
            from: s1_id.clone(),
            to: s2_id.clone(),
            edge_type: EdgeType::Sequence,
        });
        wf.add_edge(Edge {
            from: s2_id,
            to: s1_id,
            edge_type: EdgeType::Sequence,
        });

        assert!(engine.validate_dag(&wf).is_err());
    }

    #[test]
    fn test_execution_lifecycle() {
        let mut engine = DagEngine::new();
        let wf = Workflow::new("lifecycle", "Test lifecycle");
        let wf_id = wf.id.clone();
        engine.register_workflow(wf).unwrap();

        let exec_id = engine.start_execution(&wf_id).unwrap();
        assert!(engine.pause_execution(&exec_id).is_ok());
        assert!(engine.resume_execution(&exec_id).is_ok());
        assert!(engine.cancel_execution(&exec_id).is_ok());
    }
}
