use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    SharedWorkflow, TemplateParameter, WorkflowTemplate,
    WorkflowError, WorkflowResult,
};

/// Template engine — parameterized, reusable workflow patterns.
pub struct TemplateEngine {
    templates: HashMap<String, WorkflowTemplate>,
    shared: Vec<SharedWorkflow>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            shared: Vec::new(),
        }
    }

    /// Register a template.
    pub fn register(&mut self, template: WorkflowTemplate) -> WorkflowResult<()> {
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// Create a template from a workflow definition.
    pub fn create_template(
        &mut self,
        name: &str,
        description: &str,
        parameters: Vec<TemplateParameter>,
        workflow_definition: serde_json::Value,
        tags: Vec<String>,
        author: &str,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let template = WorkflowTemplate {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            version: "1.0.0".to_string(),
            parameters,
            workflow_definition,
            tags,
            author: author.to_string(),
            created_at: now,
            updated_at: now,
            rating: None,
            usage_count: 0,
        };

        self.templates.insert(id.clone(), template);
        Ok(id)
    }

    /// Instantiate a workflow from a template with parameters.
    pub fn instantiate(
        &mut self,
        template_id: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> WorkflowResult<serde_json::Value> {
        let template = self
            .templates
            .get_mut(template_id)
            .ok_or_else(|| WorkflowError::TemplateNotFound(template_id.to_string()))?;

        // Validate required parameters
        for param in &template.parameters {
            if param.required && !params.contains_key(&param.name) && param.default.is_none() {
                return Err(WorkflowError::Internal(format!(
                    "Missing required parameter: {}",
                    param.name
                )));
            }
        }

        // Apply parameters to definition (simple string replacement)
        let mut definition = serde_json::to_string(&template.workflow_definition)
            .map_err(|e| WorkflowError::SerializationError(e.to_string()))?;

        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            let replacement = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            definition = definition.replace(&placeholder, &replacement);
        }

        // Apply defaults for missing params
        for param in &template.parameters {
            if !params.contains_key(&param.name) {
                if let Some(default) = &param.default {
                    let placeholder = format!("{{{{{}}}}}", param.name);
                    let replacement = match default {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    definition = definition.replace(&placeholder, &replacement);
                }
            }
        }

        template.usage_count += 1;

        serde_json::from_str(&definition)
            .map_err(|e| WorkflowError::SerializationError(e.to_string()))
    }

    /// List available templates.
    pub fn list_templates(&self) -> Vec<&WorkflowTemplate> {
        self.templates.values().collect()
    }

    /// Search templates by tag.
    pub fn search_by_tag(&self, tag: &str) -> Vec<&WorkflowTemplate> {
        self.templates
            .values()
            .filter(|t| t.tags.iter().any(|tt| tt == tag))
            .collect()
    }

    /// Get a template by ID.
    pub fn get_template(&self, template_id: &str) -> WorkflowResult<&WorkflowTemplate> {
        self.templates
            .get(template_id)
            .ok_or_else(|| WorkflowError::TemplateNotFound(template_id.to_string()))
    }

    /// Share a template.
    pub fn share_template(
        &mut self,
        template_id: &str,
        shared_by: &str,
    ) -> WorkflowResult<String> {
        if !self.templates.contains_key(template_id) {
            return Err(WorkflowError::TemplateNotFound(template_id.to_string()));
        }

        let shared_id = Uuid::new_v4().to_string();
        self.shared.push(SharedWorkflow {
            id: shared_id.clone(),
            template_id: template_id.to_string(),
            shared_by: shared_by.to_string(),
            shared_at: Utc::now(),
            rating: 0.0,
            download_count: 0,
            privacy_verified: false,
        });

        Ok(shared_id)
    }

    /// List shared workflows.
    pub fn list_shared(&self) -> &[SharedWorkflow] {
        &self.shared
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_instantiation() {
        let mut engine = TemplateEngine::new();

        let params = vec![TemplateParameter {
            name: "app_name".to_string(),
            description: "Application name".to_string(),
            param_type: crate::types::template::ParameterType::String,
            required: true,
            default: None,
            validation: None,
        }];

        let tid = engine
            .create_template(
                "deploy",
                "Deploy an app",
                params,
                serde_json::json!({"app": "{{app_name}}", "action": "deploy"}),
                vec!["deployment".to_string()],
                "team",
            )
            .unwrap();

        let mut p = HashMap::new();
        p.insert("app_name".to_string(), serde_json::json!("my-service"));

        let result = engine.instantiate(&tid, &p).unwrap();
        assert_eq!(result["app"], "my-service");
    }
}
