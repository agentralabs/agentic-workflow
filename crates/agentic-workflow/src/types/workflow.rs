use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A workflow definition — a DAG of steps with typed edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub steps: Vec<StepNode>,
    pub edges: Vec<Edge>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl Workflow {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            version: 1,
            steps: Vec::new(),
            edges: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }

    pub fn add_step(&mut self, step: StepNode) {
        self.steps.push(step);
        self.updated_at = Utc::now();
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
        self.updated_at = Utc::now();
    }

    pub fn remove_step(&mut self, step_id: &str) {
        self.steps.retain(|s| s.id != step_id);
        self.edges.retain(|e| e.from != step_id && e.to != step_id);
        self.updated_at = Utc::now();
    }

    pub fn step_by_id(&self, step_id: &str) -> Option<&StepNode> {
        self.steps.iter().find(|s| s.id == step_id)
    }

    pub fn step_ids(&self) -> Vec<&str> {
        self.steps.iter().map(|s| s.id.as_str()).collect()
    }
}

/// An executable unit in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub step_type: StepType,
    pub inputs: HashMap<String, serde_json::Value>,
    pub timeout_ms: Option<u64>,
    pub retry_policy: Option<RetryPolicyRef>,
    pub rollback_action: Option<RollbackActionRef>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl StepNode {
    pub fn new(name: impl Into<String>, step_type: StepType) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            step_type,
            inputs: HashMap::new(),
            timeout_ms: None,
            retry_policy: None,
            rollback_action: None,
            metadata: HashMap::new(),
        }
    }
}

/// The type of action a step performs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    /// Execute a shell command
    Command { command: String, args: Vec<String> },
    /// Call an MCP tool on a sister
    McpTool { sister: String, tool: String, params: serde_json::Value },
    /// Call an HTTP API
    HttpRequest { method: String, url: String, headers: HashMap<String, String>, body: Option<String> },
    /// Execute a sub-workflow
    SubWorkflow { workflow_id: String },
    /// Fan-out to multiple destinations
    FanOut { destinations: Vec<String>, completion_policy: CompletionPolicy },
    /// Approval gate — pauses for human decision
    ApprovalGate { approvers: Vec<String>, timeout_ms: Option<u64> },
    /// Evaluate an expression and set variables
    Expression { expression: String },
    /// No-op placeholder
    Noop,
}

/// Completion policy for fan-out operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionPolicy {
    WaitAll,
    WaitAny,
    WaitN(usize),
    WaitTimeout(u64),
}

/// Directed edge between two steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

/// Type of dependency between steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    Sequence,
    Parallel,
    Conditional { expression: String },
    Loop { max_iterations: Option<u32>, condition: Option<String> },
}

/// Reference to a retry policy (details in resilience module).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyRef {
    pub policy_id: String,
}

/// Reference to a rollback action (details in resilience module).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackActionRef {
    pub action_id: String,
}
