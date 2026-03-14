use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A scheduled execution plan for a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub workflow_id: String,
    pub expression: ScheduleExpression,
    pub conflict_policy: ConflictPolicy,
    pub enabled: bool,
    pub next_fire_at: Option<DateTime<Utc>>,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub timezone: String,
    pub created_at: DateTime<Utc>,
}

/// Schedule expression — cron syntax or natural language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleExpression {
    Cron(String),
    Natural(String),
    Interval { every_ms: u64 },
    Once { at: DateTime<Utc> },
}

/// What to do when a schedule fires and previous execution is still running.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictPolicy {
    Skip,
    Queue,
    CancelPrevious,
    Wait,
}

/// Learned optimal schedule based on execution history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveSchedule {
    pub schedule_id: String,
    pub recommended_time: String,
    pub reason: String,
    pub success_rate_at_recommended: f64,
    pub success_rate_at_current: f64,
}
