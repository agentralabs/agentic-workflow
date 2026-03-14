use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    CompositionOperator, DataBridge, MetaWorkflow,
    WorkflowError, WorkflowResult,
};

/// Workflow composition algebra engine.
pub struct CompositionEngine {
    meta_workflows: HashMap<String, MetaWorkflow>,
}

impl CompositionEngine {
    pub fn new() -> Self {
        Self {
            meta_workflows: HashMap::new(),
        }
    }

    /// Create a sequence composition: A → B → C.
    pub fn sequence(&mut self, name: &str, workflow_ids: Vec<String>) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let meta = MetaWorkflow {
            id: id.clone(),
            name: name.to_string(),
            operators: vec![CompositionOperator::Sequence(workflow_ids)],
            data_bridges: Vec::new(),
            created_at: Utc::now(),
        };

        self.meta_workflows.insert(id.clone(), meta);
        Ok(id)
    }

    /// Create a parallel composition: A || B || C.
    pub fn parallel(&mut self, name: &str, workflow_ids: Vec<String>) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let meta = MetaWorkflow {
            id: id.clone(),
            name: name.to_string(),
            operators: vec![CompositionOperator::Parallel(workflow_ids)],
            data_bridges: Vec::new(),
            created_at: Utc::now(),
        };

        self.meta_workflows.insert(id.clone(), meta);
        Ok(id)
    }

    /// Create a conditional composition: if pred then A else B.
    pub fn conditional(
        &mut self,
        name: &str,
        predicate: &str,
        if_true: &str,
        if_false: &str,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let meta = MetaWorkflow {
            id: id.clone(),
            name: name.to_string(),
            operators: vec![CompositionOperator::Conditional {
                predicate: predicate.to_string(),
                if_true: if_true.to_string(),
                if_false: if_false.to_string(),
            }],
            data_bridges: Vec::new(),
            created_at: Utc::now(),
        };

        self.meta_workflows.insert(id.clone(), meta);
        Ok(id)
    }

    /// Add a data bridge between composed workflows.
    pub fn add_bridge(
        &mut self,
        meta_id: &str,
        from_workflow_id: &str,
        from_output: &str,
        to_workflow_id: &str,
        to_input: &str,
        transform: Option<String>,
    ) -> WorkflowResult<()> {
        let meta = self
            .meta_workflows
            .get_mut(meta_id)
            .ok_or_else(|| WorkflowError::Internal(format!("Meta-workflow not found: {}", meta_id)))?;

        meta.data_bridges.push(DataBridge {
            from_workflow_id: from_workflow_id.to_string(),
            from_output: from_output.to_string(),
            to_workflow_id: to_workflow_id.to_string(),
            to_input: to_input.to_string(),
            transform,
        });

        Ok(())
    }

    /// Validate a composed meta-workflow.
    pub fn validate(&self, meta_id: &str) -> WorkflowResult<Vec<String>> {
        let meta = self
            .meta_workflows
            .get(meta_id)
            .ok_or_else(|| WorkflowError::Internal(format!("Meta-workflow not found: {}", meta_id)))?;

        let mut warnings = Vec::new();

        // Check for empty compositions
        for op in &meta.operators {
            match op {
                CompositionOperator::Sequence(ids) if ids.is_empty() => {
                    warnings.push("Empty sequence composition".to_string());
                }
                CompositionOperator::Parallel(ids) if ids.is_empty() => {
                    warnings.push("Empty parallel composition".to_string());
                }
                _ => {}
            }
        }

        Ok(warnings)
    }

    /// Get a meta-workflow.
    pub fn get_meta(&self, meta_id: &str) -> WorkflowResult<&MetaWorkflow> {
        self.meta_workflows
            .get(meta_id)
            .ok_or_else(|| WorkflowError::Internal(format!("Meta-workflow not found: {}", meta_id)))
    }

    /// List all meta-workflows.
    pub fn list_meta(&self) -> Vec<&MetaWorkflow> {
        self.meta_workflows.values().collect()
    }
}

impl Default for CompositionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_composition() {
        let mut engine = CompositionEngine::new();
        let mid = engine
            .sequence(
                "deploy-pipeline",
                vec!["build".into(), "test".into(), "deploy".into()],
            )
            .unwrap();

        let meta = engine.get_meta(&mid).unwrap();
        assert_eq!(meta.name, "deploy-pipeline");

        let warnings = engine.validate(&mid).unwrap();
        assert!(warnings.is_empty());
    }
}
