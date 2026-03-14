use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A stream processor definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamProcessor {
    pub id: String,
    pub name: String,
    pub workflow_id: String,
    pub source: StreamSource,
    pub window: Option<ProcessingWindow>,
    pub backpressure: BackpressureConfig,
    pub status: StreamStatus,
    pub created_at: DateTime<Utc>,
}

/// Source of continuous data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamSource {
    FileWatch { path: String, pattern: Option<String> },
    HttpPoll { url: String, interval_ms: u64 },
    Webhook { endpoint: String },
    Queue { queue_name: String, connection: String },
    Custom { source_type: String, config: serde_json::Value },
}

/// How to batch incoming items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingWindow {
    TimeBased { window_ms: u64 },
    CountBased { count: usize },
    Sliding { window_ms: u64, slide_ms: u64 },
}

/// Backpressure handling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpressureConfig {
    pub max_queue_size: usize,
    pub strategy: BackpressureStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackpressureStrategy {
    SlowDown,
    DropOldest,
    DropNewest,
    Block,
}

/// Status of a stream processor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Error,
}

/// Stream checkpoint for exactly-once processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamCheckpoint {
    pub stream_id: String,
    pub offset: u64,
    pub items_processed: u64,
    pub checkpoint_at: DateTime<Utc>,
}

/// Stream fork — split stream by condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamFork {
    pub id: String,
    pub stream_id: String,
    pub condition: String,
    pub target_workflow_id: String,
    pub name: String,
}
