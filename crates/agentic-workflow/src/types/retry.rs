use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Failure classification for intelligent retry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureClass {
    Transient,
    Permanent,
    RateLimit,
    ResourceContention,
    Authentication,
    Network,
    Timeout,
    Unknown,
}

/// Retry policy per failure type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub id: String,
    pub name: String,
    pub profiles: Vec<RetryProfile>,
    pub budget: Option<RetryBudget>,
    pub escalation: Option<RetryEscalation>,
}

/// Per-failure-type retry strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryProfile {
    pub failure_class: FailureClass,
    pub max_attempts: u32,
    pub strategy: RetryStrategy,
    pub jitter: bool,
}

/// Strategy for timing retries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    Immediate,
    FixedDelay { delay_ms: u64 },
    ExponentialBackoff { initial_ms: u64, max_ms: u64, multiplier: f64 },
    Linear { delay_ms: u64, increment_ms: u64 },
}

/// Budget limiting total retry cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryBudget {
    pub max_total_attempts: Option<u32>,
    pub max_total_time_ms: Option<u64>,
    pub max_cost_units: Option<f64>,
}

/// What to do after exhausting retries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryEscalation {
    pub after_attempts: u32,
    pub action: EscalationAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationAction {
    AlertHuman { channel: String },
    TryAlternateStep { step_id: String },
    FailWorkflow,
    SkipStep,
}

/// Statistics about retry behavior for a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStats {
    pub step_id: String,
    pub total_attempts: u32,
    pub successes_by_attempt: Vec<u32>,
    pub avg_delay_ms: f64,
    pub last_failure_class: Option<FailureClass>,
    pub last_retry_at: Option<DateTime<Utc>>,
}

/// Learned retry pattern from historical data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPattern {
    pub step_id: String,
    pub optimal_delay_ms: u64,
    pub success_rate_by_attempt: Vec<f64>,
    pub recommendation: String,
}
