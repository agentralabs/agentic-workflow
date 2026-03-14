use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A trigger that starts workflow execution in response to events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub name: String,
    pub workflow_id: String,
    pub trigger_type: TriggerType,
    pub condition: Option<TriggerCondition>,
    pub debounce_ms: Option<u64>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of triggers supported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    /// File system change (create, modify, delete)
    FileSystem { path: String, events: Vec<FileEvent> },
    /// Incoming webhook
    Webhook { endpoint: String, method: String },
    /// Schedule-based (cron or natural language)
    Schedule { schedule_id: String },
    /// Manual trigger by user
    Manual,
    /// Output of another workflow
    WorkflowOutput { source_workflow_id: String },
    /// API callback
    ApiCallback { callback_id: String },
    /// Custom event type
    Custom { event_type: String, config: serde_json::Value },
}

/// File system event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileEvent {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Condition that must be met for trigger to fire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub expression: String,
    pub description: Option<String>,
}

/// Record of a trigger activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerActivation {
    pub trigger_id: String,
    pub execution_id: String,
    pub activated_at: DateTime<Utc>,
    pub event_data: serde_json::Value,
    pub condition_met: bool,
}
