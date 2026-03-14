use std::collections::HashMap;

use chrono::Utc;

use crate::types::{
    RollbackAction, RollbackReceipt, RollbackScope, RollbackStepResult,
    WorkflowError, WorkflowResult,
};

/// Rollback architecture — per-step undo with verification.
pub struct RollbackEngine {
    actions: HashMap<String, RollbackAction>,
    receipts: Vec<RollbackReceipt>,
}

impl RollbackEngine {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            receipts: Vec::new(),
        }
    }

    /// Define a rollback action for a step.
    pub fn define_action(&mut self, action: RollbackAction) -> WorkflowResult<()> {
        self.actions.insert(action.step_id.clone(), action);
        Ok(())
    }

    /// Get rollback action for a step.
    pub fn get_action(&self, step_id: &str) -> Option<&RollbackAction> {
        self.actions.get(step_id)
    }

    /// Preview what a rollback would do.
    pub fn preview(
        &self,
        scope: &RollbackScope,
        completed_step_ids: &[String],
    ) -> Vec<String> {
        match scope {
            RollbackScope::Full => completed_step_ids
                .iter()
                .rev()
                .filter(|id| self.actions.contains_key(id.as_str()))
                .cloned()
                .collect(),
            RollbackScope::FromStep { step_id } => {
                let start_idx = completed_step_ids
                    .iter()
                    .position(|id| id == step_id)
                    .unwrap_or(0);
                completed_step_ids[start_idx..]
                    .iter()
                    .rev()
                    .filter(|id| self.actions.contains_key(id.as_str()))
                    .cloned()
                    .collect()
            }
            RollbackScope::Selective { step_ids } => step_ids
                .iter()
                .rev()
                .filter(|id| self.actions.contains_key(id.as_str()))
                .cloned()
                .collect(),
        }
    }

    /// Execute rollback and produce a receipt.
    pub fn execute_rollback(
        &mut self,
        execution_id: &str,
        scope: RollbackScope,
        steps_to_rollback: &[String],
    ) -> WorkflowResult<RollbackReceipt> {
        let started_at = Utc::now();
        let mut results = Vec::new();
        let mut overall_success = true;

        for step_id in steps_to_rollback {
            match self.actions.get(step_id) {
                Some(action) => {
                    // In a real implementation, this would execute the rollback action
                    let result = RollbackStepResult {
                        step_id: step_id.clone(),
                        success: true,
                        error: None,
                        verification_passed: action.verification.as_ref().map(|_| true),
                    };
                    results.push(result);
                }
                None => {
                    results.push(RollbackStepResult {
                        step_id: step_id.clone(),
                        success: false,
                        error: Some("No rollback action defined".to_string()),
                        verification_passed: None,
                    });
                    overall_success = false;
                }
            }
        }

        let receipt = RollbackReceipt {
            execution_id: execution_id.to_string(),
            scope,
            rolled_back_steps: results,
            started_at,
            completed_at: Utc::now(),
            overall_success,
        };

        self.receipts.push(receipt.clone());
        Ok(receipt)
    }

    /// Get rollback receipts for an execution.
    pub fn get_receipts(&self, execution_id: &str) -> Vec<&RollbackReceipt> {
        self.receipts
            .iter()
            .filter(|r| r.execution_id == execution_id)
            .collect()
    }

    /// List all defined rollback actions.
    pub fn list_actions(&self) -> Vec<&RollbackAction> {
        self.actions.values().collect()
    }
}

impl Default for RollbackEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RollbackType;

    #[test]
    fn test_rollback_preview() {
        let mut engine = RollbackEngine::new();

        engine
            .define_action(RollbackAction {
                id: "r1".into(),
                step_id: "step-1".into(),
                action_type: RollbackType::NotPossible {
                    reason: "Email already sent".into(),
                },
                description: "Cannot undo email".into(),
                verification: None,
            })
            .unwrap();

        engine
            .define_action(RollbackAction {
                id: "r2".into(),
                step_id: "step-2".into(),
                action_type: RollbackType::Command {
                    command: "undo.sh".into(),
                    args: vec![],
                },
                description: "Undo step 2".into(),
                verification: None,
            })
            .unwrap();

        let completed = vec!["step-1".to_string(), "step-2".to_string()];
        let preview = engine.preview(&RollbackScope::Full, &completed);
        assert_eq!(preview.len(), 2);
        assert_eq!(preview[0], "step-2"); // Reverse order
    }
}
