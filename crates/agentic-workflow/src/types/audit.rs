use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A structured audit event recording an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub execution_id: String,
    pub workflow_id: String,
    pub step_id: Option<String>,
    pub event_type: AuditEventType,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub resource: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub outcome: AuditOutcome,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of auditable events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    WorkflowCreated,
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    StepExecuted,
    StepRetried,
    StepRolledBack,
    ApprovalRequested,
    ApprovalDecided,
    VariableSet,
    TriggerFired,
    ScheduleModified,
    ConfigChanged,
    Custom(String),
}

/// Outcome of an audited action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditOutcome {
    Success,
    Failure { reason: String },
    Skipped { reason: String },
    Pending,
}

/// A structured query against the audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub workflow_id: Option<String>,
    pub execution_id: Option<String>,
    pub event_types: Option<Vec<AuditEventType>>,
    pub actor: Option<String>,
    pub resource: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

/// Audit trail retention policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRetention {
    pub retain_days: u32,
    pub compliance_preset: Option<CompliancePreset>,
    pub archive_after_days: Option<u32>,
}

/// Pre-configured retention for compliance standards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompliancePreset {
    Sox,
    Gdpr,
    Hipaa,
    Custom { retain_days: u32, description: String },
}

/// Impact analysis result — all workflows that touched a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditImpact {
    pub resource: String,
    pub workflow_ids: Vec<String>,
    pub execution_ids: Vec<String>,
    pub event_count: usize,
    pub first_touch: DateTime<Utc>,
    pub last_touch: DateTime<Utc>,
}
