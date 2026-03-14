use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A batch job — collection of items processed with the same workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    pub id: String,
    pub workflow_id: String,
    pub items: Vec<BatchItem>,
    pub concurrency: usize,
    pub checkpoint_every: usize,
    pub status: BatchStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// An individual item in a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    pub index: usize,
    pub input: serde_json::Value,
    pub status: BatchItemStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Status of the overall batch job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Running,
    Paused,
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
}

/// Status of a single batch item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchItemStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

/// Batch progress report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub batch_id: String,
    pub total_items: usize,
    pub completed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub running: usize,
    pub pending: usize,
    pub percent_complete: f64,
    pub estimated_remaining_ms: Option<u64>,
    pub last_checkpoint_index: Option<usize>,
}

/// Batch completion report with error analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReport {
    pub batch_id: String,
    pub total_items: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
    pub total_duration_ms: u64,
    pub avg_item_duration_ms: f64,
    pub error_summary: Vec<BatchErrorGroup>,
}

/// Group of batch errors with same type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchErrorGroup {
    pub error_pattern: String,
    pub count: usize,
    pub sample_indices: Vec<usize>,
}
