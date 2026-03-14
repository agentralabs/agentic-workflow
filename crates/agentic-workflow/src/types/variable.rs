use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hierarchical variable scope: workflow → branch → step → iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableScope {
    pub scope_id: String,
    pub scope_type: ScopeType,
    pub parent_scope_id: Option<String>,
    pub variables: HashMap<String, ScopedVariable>,
}

/// Type of scope in the hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeType {
    Workflow,
    Branch,
    Step,
    Iteration,
}

/// A variable with scope, type, and visibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedVariable {
    pub name: String,
    pub value: serde_json::Value,
    pub var_type: VariableType,
    pub immutable: bool,
    pub set_at: DateTime<Utc>,
    pub set_by: String,
}

/// Supported variable types for type checking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Null,
    Any,
}

impl VariableType {
    /// Check if a JSON value matches this type.
    pub fn matches(&self, value: &serde_json::Value) -> bool {
        match self {
            Self::String => value.is_string(),
            Self::Integer => value.is_i64() || value.is_u64(),
            Self::Float => value.is_f64(),
            Self::Boolean => value.is_boolean(),
            Self::Array => value.is_array(),
            Self::Object => value.is_object(),
            Self::Null => value.is_null(),
            Self::Any => true,
        }
    }
}

/// Result of variable type checking across a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCheckResult {
    pub valid: bool,
    pub errors: Vec<TypeCheckError>,
}

/// A type mismatch error found during type checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeCheckError {
    pub variable_name: String,
    pub scope_id: String,
    pub expected: VariableType,
    pub actual: String,
    pub message: String,
}
