use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    ScopeType, ScopedVariable, TypeCheckError, TypeCheckResult, VariableScope,
    VariableType, WorkflowError, WorkflowResult,
};

/// Hierarchical variable scoping engine.
pub struct VariableEngine {
    scopes: HashMap<String, VariableScope>,
}

impl VariableEngine {
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }

    /// Create a new scope.
    pub fn create_scope(
        &mut self,
        scope_type: ScopeType,
        parent_scope_id: Option<&str>,
    ) -> String {
        let id = Uuid::new_v4().to_string();
        let scope = VariableScope {
            scope_id: id.clone(),
            scope_type,
            parent_scope_id: parent_scope_id.map(|s| s.to_string()),
            variables: HashMap::new(),
        };

        self.scopes.insert(id.clone(), scope);
        id
    }

    /// Set a variable in a scope.
    pub fn set(
        &mut self,
        scope_id: &str,
        name: &str,
        value: serde_json::Value,
        var_type: VariableType,
        set_by: &str,
    ) -> WorkflowResult<()> {
        let scope = self
            .scopes
            .get_mut(scope_id)
            .ok_or_else(|| WorkflowError::VariableNotFound(format!("Scope: {}", scope_id)))?;

        // Check immutability
        if let Some(existing) = scope.variables.get(name) {
            if existing.immutable {
                return Err(WorkflowError::Internal(format!(
                    "Variable '{}' is immutable",
                    name
                )));
            }
        }

        // Type check
        if !var_type.matches(&value) {
            return Err(WorkflowError::VariableTypeMismatch {
                expected: format!("{:?}", var_type),
                actual: format!("{}", value),
            });
        }

        scope.variables.insert(
            name.to_string(),
            ScopedVariable {
                name: name.to_string(),
                value,
                var_type,
                immutable: false,
                set_at: Utc::now(),
                set_by: set_by.to_string(),
            },
        );

        Ok(())
    }

    /// Get a variable, respecting scope hierarchy (child → parent cascade).
    pub fn get(&self, scope_id: &str, name: &str) -> WorkflowResult<&ScopedVariable> {
        let mut current_scope_id = Some(scope_id.to_string());

        while let Some(sid) = current_scope_id {
            if let Some(scope) = self.scopes.get(&sid) {
                if let Some(var) = scope.variables.get(name) {
                    return Ok(var);
                }
                current_scope_id = scope.parent_scope_id.clone();
            } else {
                break;
            }
        }

        Err(WorkflowError::VariableNotFound(name.to_string()))
    }

    /// List all variables in a scope (not including parent).
    pub fn list(&self, scope_id: &str) -> WorkflowResult<Vec<&ScopedVariable>> {
        let scope = self
            .scopes
            .get(scope_id)
            .ok_or_else(|| WorkflowError::VariableNotFound(format!("Scope: {}", scope_id)))?;

        Ok(scope.variables.values().collect())
    }

    /// Promote a variable from child scope to parent scope.
    pub fn promote(&mut self, scope_id: &str, name: &str) -> WorkflowResult<()> {
        let (parent_id, var) = {
            let scope = self
                .scopes
                .get(scope_id)
                .ok_or_else(|| WorkflowError::VariableNotFound(format!("Scope: {}", scope_id)))?;

            let var = scope
                .variables
                .get(name)
                .ok_or_else(|| WorkflowError::VariableNotFound(name.to_string()))?
                .clone();

            let parent_id = scope
                .parent_scope_id
                .clone()
                .ok_or_else(|| WorkflowError::Internal("No parent scope".to_string()))?;

            (parent_id, var)
        };

        let parent = self
            .scopes
            .get_mut(&parent_id)
            .ok_or_else(|| WorkflowError::Internal("Parent scope not found".to_string()))?;

        parent.variables.insert(name.to_string(), var);
        Ok(())
    }

    /// Type check all variables across scopes.
    pub fn type_check(&self) -> TypeCheckResult {
        let mut errors = Vec::new();

        for scope in self.scopes.values() {
            for var in scope.variables.values() {
                if !var.var_type.matches(&var.value) {
                    errors.push(TypeCheckError {
                        variable_name: var.name.clone(),
                        scope_id: scope.scope_id.clone(),
                        expected: var.var_type.clone(),
                        actual: format!("{}", var.value),
                        message: format!(
                            "Variable '{}' expected {:?} but got {}",
                            var.name, var.var_type, var.value
                        ),
                    });
                }
            }
        }

        TypeCheckResult {
            valid: errors.is_empty(),
            errors,
        }
    }

    /// Make a variable immutable.
    pub fn make_immutable(&mut self, scope_id: &str, name: &str) -> WorkflowResult<()> {
        let scope = self
            .scopes
            .get_mut(scope_id)
            .ok_or_else(|| WorkflowError::VariableNotFound(format!("Scope: {}", scope_id)))?;

        let var = scope
            .variables
            .get_mut(name)
            .ok_or_else(|| WorkflowError::VariableNotFound(name.to_string()))?;

        var.immutable = true;
        Ok(())
    }
}

impl Default for VariableEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_scope_hierarchy() {
        let mut engine = VariableEngine::new();
        let parent_id = engine.create_scope(ScopeType::Workflow, None);
        let child_id = engine.create_scope(ScopeType::Step, Some(&parent_id));

        engine
            .set(&parent_id, "config", serde_json::json!("prod"), VariableType::String, "system")
            .unwrap();

        // Child can read parent's variable
        let var = engine.get(&child_id, "config").unwrap();
        assert_eq!(var.value, serde_json::json!("prod"));
    }

    #[test]
    fn test_immutability() {
        let mut engine = VariableEngine::new();
        let sid = engine.create_scope(ScopeType::Workflow, None);

        engine
            .set(&sid, "frozen", serde_json::json!(42), VariableType::Integer, "system")
            .unwrap();
        engine.make_immutable(&sid, "frozen").unwrap();

        let result = engine.set(&sid, "frozen", serde_json::json!(99), VariableType::Integer, "system");
        assert!(result.is_err());
    }
}
