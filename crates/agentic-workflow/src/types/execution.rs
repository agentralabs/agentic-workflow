use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete state of a running or completed workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub workflow_id: String,
    pub status: ExecutionStatus,
    pub step_states: HashMap<String, StepState>,
    pub variables: HashMap<String, serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub trigger_info: Option<TriggerInfo>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Overall execution status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Paused,
    Succeeded,
    Failed { error: String },
    Cancelled,
    RollingBack,
    RolledBack,
}

/// State of an individual step within an execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepState {
    pub step_id: String,
    pub lifecycle: StepLifecycle,
    pub attempt: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Lifecycle stages of a step execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepLifecycle {
    Pending,
    Queued,
    Running,
    Success,
    Failed,
    Skipped,
    Cancelled,
    WaitingApproval,
    RolledBack,
}

/// Information about what triggered this execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerInfo {
    pub trigger_type: String,
    pub trigger_id: Option<String>,
    pub event_data: Option<serde_json::Value>,
    pub triggered_at: DateTime<Utc>,
}

/// Progress tracking for an execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProgress {
    pub execution_id: String,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub skipped_steps: usize,
    pub running_steps: usize,
    pub pending_steps: usize,
    pub estimated_remaining_ms: Option<u64>,
    pub percent_complete: f64,
}

/// An event emitted during execution for live observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub execution_id: String,
    pub step_id: Option<String>,
    pub event_type: ExecutionEventType,
    pub timestamp: DateTime<Utc>,
    pub data: Option<serde_json::Value>,
}

/// Types of events that occur during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEventType {
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowPaused,
    WorkflowResumed,
    WorkflowCancelled,
    StepStarted,
    StepCompleted,
    StepFailed,
    StepSkipped,
    StepRetrying { attempt: u32 },
    ApprovalRequested,
    ApprovalGranted,
    ApprovalDenied,
    VariableSet { name: String },
    RollbackStarted,
    RollbackCompleted,
}

/// Execution fingerprint for comparison and analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFingerprint {
    pub execution_id: String,
    pub workflow_id: String,
    pub total_duration_ms: u64,
    pub step_durations: HashMap<String, u64>,
    pub step_outcomes: HashMap<String, StepLifecycle>,
    pub retry_count: u32,
    pub completed_at: DateTime<Utc>,
}
