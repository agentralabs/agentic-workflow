use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A failed workflow item in the dead letter queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterItem {
    pub id: String,
    pub execution_id: String,
    pub workflow_id: String,
    pub step_id: String,
    pub failure_class: String,
    pub error_message: String,
    pub input_data: serde_json::Value,
    pub attempt_count: u32,
    pub failed_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub resurrectable: bool,
}

/// Summary of dead letter items grouped by failure type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterSummary {
    pub total_items: usize,
    pub by_failure_class: Vec<FailureGroup>,
    pub auto_retryable: usize,
    pub needs_human: usize,
    pub oldest_item: Option<DateTime<Utc>>,
}

/// Group of dead letter items with same failure type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureGroup {
    pub failure_class: String,
    pub count: usize,
    pub auto_retryable: bool,
    pub sample_error: String,
}

/// Dead letter retention policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterPolicy {
    pub retention_days: u32,
    pub auto_resurrect_on_recovery: bool,
    pub max_items: Option<usize>,
    pub alert_threshold: Option<usize>,
}
