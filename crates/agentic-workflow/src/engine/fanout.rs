use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    CompletionPolicy, FanOutBranch, FanOutBranchStatus, FanOutDestination,
    FanOutStatus, FanOutStep, ResultAggregator,
    WorkflowError, WorkflowResult,
};

/// Fan-out/fan-in engine for parallel distribution and result collection.
pub struct FanOutEngine {
    steps: HashMap<String, FanOutStep>,
    statuses: HashMap<String, FanOutStatus>,
}

impl FanOutEngine {
    pub fn new() -> Self {
        Self {
            steps: HashMap::new(),
            statuses: HashMap::new(),
        }
    }

    /// Create a fan-out step.
    pub fn create_fanout(
        &mut self,
        destinations: Vec<FanOutDestination>,
        completion_policy: CompletionPolicy,
        aggregator: ResultAggregator,
        timeout_ms: Option<u64>,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let step = FanOutStep {
            id: id.clone(),
            destinations,
            completion_policy,
            aggregator,
            partial_success_threshold: None,
            timeout_ms,
        };

        self.steps.insert(id.clone(), step);
        Ok(id)
    }

    /// Start executing a fan-out — creates branch tracking.
    pub fn start_execution(
        &mut self,
        fanout_id: &str,
        execution_id: &str,
    ) -> WorkflowResult<()> {
        let step = self
            .steps
            .get(fanout_id)
            .ok_or_else(|| WorkflowError::Internal(format!("FanOut not found: {}", fanout_id)))?;

        let branches: Vec<FanOutBranch> = step
            .destinations
            .iter()
            .map(|d| FanOutBranch {
                destination_id: d.id.clone(),
                status: FanOutBranchStatus::Pending,
                output: None,
                error: None,
                duration_ms: None,
            })
            .collect();

        let status = FanOutStatus {
            fanout_id: fanout_id.to_string(),
            execution_id: execution_id.to_string(),
            branches,
            started_at: Utc::now(),
            completed: false,
        };

        self.statuses.insert(execution_id.to_string(), status);
        Ok(())
    }

    /// Update a branch status.
    pub fn update_branch(
        &mut self,
        execution_id: &str,
        destination_id: &str,
        status: FanOutBranchStatus,
        output: Option<serde_json::Value>,
        error: Option<String>,
        duration_ms: Option<u64>,
    ) -> WorkflowResult<()> {
        let fanout_status = self
            .statuses
            .get_mut(execution_id)
            .ok_or_else(|| {
                WorkflowError::ExecutionNotFound(execution_id.to_string())
            })?;

        if let Some(branch) = fanout_status
            .branches
            .iter_mut()
            .find(|b| b.destination_id == destination_id)
        {
            branch.status = status;
            branch.output = output;
            branch.error = error;
            branch.duration_ms = duration_ms;
        }

        // Check if fan-out is complete
        let all_done = fanout_status
            .branches
            .iter()
            .all(|b| matches!(
                b.status,
                FanOutBranchStatus::Success
                    | FanOutBranchStatus::Failed
                    | FanOutBranchStatus::TimedOut
                    | FanOutBranchStatus::Cancelled
            ));

        if all_done {
            fanout_status.completed = true;
        }

        Ok(())
    }

    /// Get fan-out status.
    pub fn get_status(&self, execution_id: &str) -> WorkflowResult<&FanOutStatus> {
        self.statuses
            .get(execution_id)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(execution_id.to_string()))
    }

    /// Get a fan-out step definition.
    pub fn get_step(&self, fanout_id: &str) -> WorkflowResult<&FanOutStep> {
        self.steps
            .get(fanout_id)
            .ok_or_else(|| WorkflowError::Internal(format!("FanOut not found: {}", fanout_id)))
    }
}

impl Default for FanOutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fanout_creation() {
        let mut engine = FanOutEngine::new();
        let dests = vec![
            FanOutDestination {
                id: "d1".to_string(),
                name: "API 1".to_string(),
                step_config: serde_json::json!({}),
            },
            FanOutDestination {
                id: "d2".to_string(),
                name: "API 2".to_string(),
                step_config: serde_json::json!({}),
            },
        ];

        let fid = engine
            .create_fanout(dests, CompletionPolicy::WaitAll, ResultAggregator::Merge, None)
            .unwrap();

        engine.start_execution(&fid, "exec-1").unwrap();
        let status = engine.get_status("exec-1").unwrap();
        assert_eq!(status.branches.len(), 2);
        assert!(!status.completed);
    }
}
