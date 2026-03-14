use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    Trigger, TriggerActivation, TriggerCondition, TriggerType,
    WorkflowError, WorkflowResult,
};

/// Universal trigger engine — any event can start a workflow.
pub struct TriggerEngine {
    triggers: HashMap<String, Trigger>,
    activations: Vec<TriggerActivation>,
}

impl TriggerEngine {
    pub fn new() -> Self {
        Self {
            triggers: HashMap::new(),
            activations: Vec::new(),
        }
    }

    /// Create a new trigger.
    pub fn create_trigger(
        &mut self,
        name: &str,
        workflow_id: &str,
        trigger_type: TriggerType,
        condition: Option<TriggerCondition>,
        debounce_ms: Option<u64>,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let trigger = Trigger {
            id: id.clone(),
            name: name.to_string(),
            workflow_id: workflow_id.to_string(),
            trigger_type,
            condition,
            debounce_ms,
            enabled: true,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };

        self.triggers.insert(id.clone(), trigger);
        Ok(id)
    }

    /// List all triggers.
    pub fn list_triggers(&self) -> Vec<&Trigger> {
        self.triggers.values().collect()
    }

    /// List triggers for a specific workflow.
    pub fn triggers_for_workflow(&self, workflow_id: &str) -> Vec<&Trigger> {
        self.triggers
            .values()
            .filter(|t| t.workflow_id == workflow_id)
            .collect()
    }

    /// Enable or disable a trigger.
    pub fn set_enabled(&mut self, trigger_id: &str, enabled: bool) -> WorkflowResult<()> {
        let trigger = self
            .triggers
            .get_mut(trigger_id)
            .ok_or_else(|| WorkflowError::TriggerError(format!("Not found: {}", trigger_id)))?;

        trigger.enabled = enabled;
        Ok(())
    }

    /// Remove a trigger.
    pub fn remove_trigger(&mut self, trigger_id: &str) -> WorkflowResult<Trigger> {
        self.triggers
            .remove(trigger_id)
            .ok_or_else(|| WorkflowError::TriggerError(format!("Not found: {}", trigger_id)))
    }

    /// Record a trigger activation.
    pub fn record_activation(
        &mut self,
        trigger_id: &str,
        execution_id: &str,
        event_data: serde_json::Value,
        condition_met: bool,
    ) -> WorkflowResult<()> {
        if !self.triggers.contains_key(trigger_id) {
            return Err(WorkflowError::TriggerError(format!(
                "Not found: {}",
                trigger_id
            )));
        }

        self.activations.push(TriggerActivation {
            trigger_id: trigger_id.to_string(),
            execution_id: execution_id.to_string(),
            activated_at: Utc::now(),
            event_data,
            condition_met,
        });

        Ok(())
    }

    /// Get activation history for a trigger.
    pub fn activation_history(&self, trigger_id: &str) -> Vec<&TriggerActivation> {
        self.activations
            .iter()
            .filter(|a| a.trigger_id == trigger_id)
            .collect()
    }

    /// Test a trigger condition against sample event data.
    pub fn test_condition(
        &self,
        trigger_id: &str,
        event_data: &serde_json::Value,
    ) -> WorkflowResult<bool> {
        let trigger = self
            .triggers
            .get(trigger_id)
            .ok_or_else(|| WorkflowError::TriggerError(format!("Not found: {}", trigger_id)))?;

        match &trigger.condition {
            None => Ok(true),
            Some(_condition) => {
                // Expression evaluation would go here
                // For now, return true (condition evaluation is an LLM task per CLAUDE.md)
                Ok(true)
            }
        }
    }
}

impl Default for TriggerEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_lifecycle() {
        let mut engine = TriggerEngine::new();
        let tid = engine
            .create_trigger("on-file-change", "wf-1", TriggerType::Manual, None, None)
            .unwrap();

        assert_eq!(engine.list_triggers().len(), 1);
        engine.set_enabled(&tid, false).unwrap();
        assert!(!engine.triggers.get(&tid).unwrap().enabled);
        engine.remove_trigger(&tid).unwrap();
        assert_eq!(engine.list_triggers().len(), 0);
    }
}
