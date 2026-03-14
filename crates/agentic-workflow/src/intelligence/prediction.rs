use std::collections::HashMap;

use crate::types::{
    CostPrediction, DurationPrediction, ExecutionFingerprint, ResourcePrediction,
    RiskFactor, StepDurationPrediction, StepLifecycle, SuccessPrediction,
    WorkflowResult,
};

/// Predictive execution engine — estimates from historical data.
pub struct PredictionEngine {
    fingerprints: Vec<ExecutionFingerprint>,
}

impl PredictionEngine {
    pub fn new() -> Self {
        Self {
            fingerprints: Vec::new(),
        }
    }

    /// Add historical execution data.
    pub fn ingest_fingerprint(&mut self, fp: ExecutionFingerprint) {
        self.fingerprints.push(fp);
    }

    /// Predict execution duration.
    pub fn predict_duration(&self, workflow_id: &str) -> WorkflowResult<DurationPrediction> {
        let fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if fps.is_empty() {
            return Ok(DurationPrediction {
                workflow_id: workflow_id.to_string(),
                predicted_ms: 0,
                confidence: 0.0,
                min_ms: 0,
                max_ms: 0,
                based_on_executions: 0,
                step_predictions: Vec::new(),
            });
        }

        let durations: Vec<u64> = fps.iter().map(|f| f.total_duration_ms).collect();
        let avg = durations.iter().sum::<u64>() / durations.len() as u64;
        let min = *durations.iter().min().unwrap();
        let max = *durations.iter().max().unwrap();

        let confidence = (fps.len() as f64 / 10.0).min(1.0);

        // Step-level predictions
        let mut step_totals: HashMap<&str, (u64, usize)> = HashMap::new();
        for fp in &fps {
            for (sid, dur) in &fp.step_durations {
                let e = step_totals.entry(sid.as_str()).or_insert((0, 0));
                e.0 += dur;
                e.1 += 1;
            }
        }

        let step_predictions: Vec<StepDurationPrediction> = step_totals
            .into_iter()
            .map(|(sid, (total, count))| StepDurationPrediction {
                step_id: sid.to_string(),
                predicted_ms: total / count as u64,
                confidence,
            })
            .collect();

        Ok(DurationPrediction {
            workflow_id: workflow_id.to_string(),
            predicted_ms: avg,
            confidence,
            min_ms: min,
            max_ms: max,
            based_on_executions: fps.len(),
            step_predictions,
        })
    }

    /// Predict success probability.
    pub fn predict_success(&self, workflow_id: &str) -> WorkflowResult<SuccessPrediction> {
        let fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if fps.is_empty() {
            return Ok(SuccessPrediction {
                workflow_id: workflow_id.to_string(),
                success_probability: 0.5,
                risk_factors: Vec::new(),
                based_on_executions: 0,
            });
        }

        // Count step failure rates
        let mut step_failures: HashMap<&str, (usize, usize)> = HashMap::new();
        for fp in &fps {
            for (sid, outcome) in &fp.step_outcomes {
                let e = step_failures.entry(sid.as_str()).or_insert((0, 0));
                e.1 += 1;
                if *outcome == StepLifecycle::Failed {
                    e.0 += 1;
                }
            }
        }

        let risk_factors: Vec<RiskFactor> = step_failures
            .into_iter()
            .filter(|(_, (fails, total))| *fails > 0 && *total > 0)
            .map(|(sid, (fails, total))| RiskFactor {
                step_id: sid.to_string(),
                risk: fails as f64 / total as f64,
                reason: format!("{}/{} executions failed", fails, total),
            })
            .collect();

        let total_success = fps
            .iter()
            .filter(|f| f.retry_count == 0 && f.step_outcomes.values().all(|o| *o == StepLifecycle::Success))
            .count();

        let probability = total_success as f64 / fps.len() as f64;

        Ok(SuccessPrediction {
            workflow_id: workflow_id.to_string(),
            success_probability: probability,
            risk_factors,
            based_on_executions: fps.len(),
        })
    }

    /// Predict resource consumption.
    pub fn predict_resources(&self, workflow_id: &str) -> WorkflowResult<ResourcePrediction> {
        let fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        let avg_steps = if !fps.is_empty() {
            fps.iter().map(|f| f.step_durations.len() as u64).sum::<u64>() / fps.len() as u64
        } else {
            0
        };

        Ok(ResourcePrediction {
            workflow_id: workflow_id.to_string(),
            estimated_api_calls: avg_steps,
            estimated_compute_seconds: avg_steps as f64 * 0.5,
            estimated_storage_bytes: avg_steps * 1024,
        })
    }

    /// Predict monetary cost.
    pub fn predict_cost(&self, workflow_id: &str) -> WorkflowResult<CostPrediction> {
        let resources = self.predict_resources(workflow_id)?;

        Ok(CostPrediction {
            workflow_id: workflow_id.to_string(),
            estimated_cost_usd: resources.estimated_api_calls as f64 * 0.001,
            breakdown: vec![crate::types::prediction::CostBreakdown {
                component: "API calls".to_string(),
                cost_usd: resources.estimated_api_calls as f64 * 0.001,
                quantity: resources.estimated_api_calls as f64,
                unit: "calls".to_string(),
            }],
            confidence: if self.fingerprints.is_empty() { 0.0 } else { 0.7 },
        })
    }
}

impl Default for PredictionEngine {
    fn default() -> Self {
        Self::new()
    }
}
