use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Rollback action defined for a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackAction {
    pub id: String,
    pub step_id: String,
    pub action_type: RollbackType,
    pub description: String,
    pub verification: Option<RollbackVerification>,
}

/// Type of rollback/compensation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackType {
    /// Execute a command to undo
    Command { command: String, args: Vec<String> },
    /// Call an MCP tool
    McpTool { sister: String, tool: String, params: serde_json::Value },
    /// HTTP call to undo
    HttpRequest { method: String, url: String },
    /// Compensating transaction (when true undo isn't possible)
    Compensate { description: String, action: serde_json::Value },
    /// No rollback possible — documented reason
    NotPossible { reason: String },
}

/// How to verify system state after rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackVerification {
    pub check_type: VerificationType,
    pub expected_state: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationType {
    Command { command: String },
    HttpCheck { url: String, expected_status: u16 },
    Expression { expression: String },
}

/// Scope of rollback to execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackScope {
    Full,
    FromStep { step_id: String },
    Selective { step_ids: Vec<String> },
}

/// Receipt documenting what was rolled back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackReceipt {
    pub execution_id: String,
    pub scope: RollbackScope,
    pub rolled_back_steps: Vec<RollbackStepResult>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub overall_success: bool,
}

/// Result of rolling back an individual step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStepResult {
    pub step_id: String,
    pub success: bool,
    pub error: Option<String>,
    pub verification_passed: Option<bool>,
}
