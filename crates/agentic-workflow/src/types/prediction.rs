use serde::{Deserialize, Serialize};

/// Predicted execution duration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationPrediction {
    pub workflow_id: String,
    pub predicted_ms: u64,
    pub confidence: f64,
    pub min_ms: u64,
    pub max_ms: u64,
    pub based_on_executions: usize,
    pub step_predictions: Vec<StepDurationPrediction>,
}

/// Per-step duration prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDurationPrediction {
    pub step_id: String,
    pub predicted_ms: u64,
    pub confidence: f64,
}

/// Predicted success probability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessPrediction {
    pub workflow_id: String,
    pub success_probability: f64,
    pub risk_factors: Vec<RiskFactor>,
    pub based_on_executions: usize,
}

/// A factor that may cause failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub step_id: String,
    pub risk: f64,
    pub reason: String,
}

/// Predicted resource consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePrediction {
    pub workflow_id: String,
    pub estimated_api_calls: u64,
    pub estimated_compute_seconds: f64,
    pub estimated_storage_bytes: u64,
}

/// Predicted monetary cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPrediction {
    pub workflow_id: String,
    pub estimated_cost_usd: f64,
    pub breakdown: Vec<CostBreakdown>,
    pub confidence: f64,
}

/// Cost breakdown by component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub component: String,
    pub cost_usd: f64,
    pub quantity: f64,
    pub unit: String,
}

/// Workflow health score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowHealth {
    pub workflow_id: String,
    pub score: f64,
    pub success_rate: f64,
    pub avg_duration_ms: u64,
    pub drift_detected: bool,
    pub issues: Vec<HealthIssue>,
}

/// A health issue detected in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub severity: IssueSeverity,
    pub step_id: Option<String>,
    pub description: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Critical,
}
