use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Idempotency configuration for a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyConfig {
    pub step_id: String,
    pub key_strategy: KeyStrategy,
    pub window: IdempotencyWindow,
    pub conflict_resolution: ConflictResolution,
}

/// How to compute the idempotency key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStrategy {
    /// workflow_id + step_id + input_hash
    InputHash,
    /// Custom expression-based key
    Expression(String),
    /// User-provided key field
    FieldPath(String),
}

/// How long to keep cached results for deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdempotencyWindow {
    Duration { ms: u64 },
    Forever,
    UntilNextExecution,
}

/// When same key appears with different inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    ReturnCached,
    RejectNew,
    ReplaceOld,
}

/// A cached execution result for deduplication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyEntry {
    pub key: String,
    pub step_id: String,
    pub execution_id: String,
    pub input_hash: String,
    pub output: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Deduplication statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyReport {
    pub total_entries: usize,
    pub deduplicated_count: u64,
    pub cache_hit_rate: f64,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub by_step: Vec<StepIdempotencyStats>,
}

/// Per-step idempotency stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepIdempotencyStats {
    pub step_id: String,
    pub entries: usize,
    pub hits: u64,
    pub saved_executions: u64,
}
