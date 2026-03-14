use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A parameterized workflow template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub parameters: Vec<TemplateParameter>,
    pub workflow_definition: serde_json::Value,
    pub tags: Vec<String>,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub rating: Option<f64>,
    pub usage_count: u64,
}

/// A typed parameter for a workflow template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub validation: Option<String>,
}

/// Supported parameter types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Enum { values: Vec<String> },
    List { item_type: Box<ParameterType> },
    Object { schema: HashMap<String, ParameterType> },
}

/// A natural language workflow request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageRequest {
    pub description: String,
    pub clarifications: Vec<Clarification>,
    pub synthesized_workflow: Option<serde_json::Value>,
}

/// A clarification question and answer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clarification {
    pub question: String,
    pub answer: Option<String>,
    pub options: Option<Vec<String>>,
}

/// Workflow composition operators.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionOperator {
    Sequence(Vec<String>),
    Parallel(Vec<String>),
    Conditional { predicate: String, if_true: String, if_false: String },
    Loop { workflow_id: String, condition: String, max_iterations: Option<u32> },
}

/// Meta-workflow composed of other workflows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaWorkflow {
    pub id: String,
    pub name: String,
    pub operators: Vec<CompositionOperator>,
    pub data_bridges: Vec<DataBridge>,
    pub created_at: DateTime<Utc>,
}

/// Connects output of one workflow to input of another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBridge {
    pub from_workflow_id: String,
    pub from_output: String,
    pub to_workflow_id: String,
    pub to_input: String,
    pub transform: Option<String>,
}

/// Community-shared workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWorkflow {
    pub id: String,
    pub template_id: String,
    pub shared_by: String,
    pub shared_at: DateTime<Utc>,
    pub rating: f64,
    pub download_count: u64,
    pub privacy_verified: bool,
}
