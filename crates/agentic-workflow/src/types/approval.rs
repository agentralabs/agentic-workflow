use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An approval gate definition within a workflow step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalGate {
    pub id: String,
    pub step_id: String,
    pub workflow_id: String,
    pub approver_chain: Vec<Approver>,
    pub condition: Option<ConditionalApproval>,
    pub timeout: Option<TimeBoundApproval>,
    pub delegation: Option<DelegationRule>,
}

/// An approver in the approval chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approver {
    pub identity: String,
    pub role: Option<String>,
    pub priority: u32,
}

/// Auto-approve if conditions are met.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalApproval {
    pub expression: String,
    pub description: String,
}

/// Time-bounded approval — auto-approve or auto-deny after timeout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBoundApproval {
    pub timeout_ms: u64,
    pub on_timeout: TimeoutAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeoutAction {
    AutoApprove,
    AutoDeny,
    Escalate,
}

/// Delegation rule when primary approver is unavailable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRule {
    pub delegate_to: String,
    pub condition: Option<String>,
}

/// A pending approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingApproval {
    pub gate_id: String,
    pub execution_id: String,
    pub step_id: String,
    pub requested_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub current_approver: String,
    pub context: serde_json::Value,
}

/// Cryptographic proof of who approved what and when.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalReceipt {
    pub gate_id: String,
    pub execution_id: String,
    pub decision: ApprovalDecision,
    pub decided_by: String,
    pub decided_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approved,
    Denied,
    Escalated,
    TimedOut,
    Delegated { to: String },
}
