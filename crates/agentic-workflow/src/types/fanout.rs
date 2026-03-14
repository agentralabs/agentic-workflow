use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::workflow::CompletionPolicy;

/// Fan-out/fan-in step definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanOutStep {
    pub id: String,
    pub destinations: Vec<FanOutDestination>,
    pub completion_policy: CompletionPolicy,
    pub aggregator: ResultAggregator,
    pub partial_success_threshold: Option<f64>,
    pub timeout_ms: Option<u64>,
}

/// A single destination in a fan-out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanOutDestination {
    pub id: String,
    pub name: String,
    pub step_config: serde_json::Value,
}

/// How to merge results from fan-out branches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultAggregator {
    Concat,
    Merge,
    Reduce { expression: String },
    First,
    Last,
    Custom { function: String },
}

/// Status of an executing fan-out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanOutStatus {
    pub fanout_id: String,
    pub execution_id: String,
    pub branches: Vec<FanOutBranch>,
    pub started_at: DateTime<Utc>,
    pub completed: bool,
}

/// Status of a single fan-out branch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanOutBranch {
    pub destination_id: String,
    pub status: FanOutBranchStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FanOutBranchStatus {
    Pending,
    Running,
    Success,
    Failed,
    TimedOut,
    Cancelled,
}
