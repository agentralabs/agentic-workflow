use std::collections::HashMap;

use crate::types::{
    ExecutionFingerprint, WorkflowError, WorkflowResult,
};

/// Execution archaeology — compare, analyze, and diagnose executions.
pub struct ArchaeologyEngine {
    fingerprints: Vec<ExecutionFingerprint>,
}

impl ArchaeologyEngine {
    pub fn new() -> Self {
        Self {
            fingerprints: Vec::new(),
        }
    }

    /// Store an execution fingerprint.
    pub fn record_fingerprint(&mut self, fp: ExecutionFingerprint) {
        self.fingerprints.push(fp);
    }

    /// Compare two executions.
    pub fn compare(
        &self,
        exec_a: &str,
        exec_b: &str,
    ) -> WorkflowResult<ExecutionComparison> {
        let fp_a = self
            .fingerprints
            .iter()
            .find(|f| f.execution_id == exec_a)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(exec_a.to_string()))?;

        let fp_b = self
            .fingerprints
            .iter()
            .find(|f| f.execution_id == exec_b)
            .ok_or_else(|| WorkflowError::ExecutionNotFound(exec_b.to_string()))?;

        let mut step_diffs = Vec::new();
        for (step_id, dur_a) in &fp_a.step_durations {
            if let Some(dur_b) = fp_b.step_durations.get(step_id) {
                let ratio = *dur_b as f64 / *dur_a as f64;
                if ratio > 1.5 || ratio < 0.5 {
                    step_diffs.push(StepDiff {
                        step_id: step_id.clone(),
                        duration_a_ms: *dur_a,
                        duration_b_ms: *dur_b,
                        ratio,
                    });
                }
            }
        }

        Ok(ExecutionComparison {
            execution_a: exec_a.to_string(),
            execution_b: exec_b.to_string(),
            duration_a_ms: fp_a.total_duration_ms,
            duration_b_ms: fp_b.total_duration_ms,
            duration_ratio: fp_b.total_duration_ms as f64 / fp_a.total_duration_ms.max(1) as f64,
            significant_step_diffs: step_diffs,
        })
    }

    /// Detect anomalous executions for a workflow.
    pub fn detect_anomalies(&self, workflow_id: &str) -> Vec<Anomaly> {
        let wf_fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if wf_fps.len() < 3 {
            return Vec::new();
        }

        let avg_duration: f64 =
            wf_fps.iter().map(|f| f.total_duration_ms as f64).sum::<f64>() / wf_fps.len() as f64;

        let mut anomalies = Vec::new();
        for fp in &wf_fps {
            let ratio = fp.total_duration_ms as f64 / avg_duration;
            if ratio > 3.0 || ratio < 0.1 {
                anomalies.push(Anomaly {
                    execution_id: fp.execution_id.clone(),
                    metric: "duration".to_string(),
                    actual: fp.total_duration_ms as f64,
                    expected: avg_duration,
                    deviation_factor: ratio,
                });
            }
        }

        anomalies
    }

    /// Identify bottleneck steps across executions.
    pub fn bottlenecks(&self, workflow_id: &str) -> Vec<Bottleneck> {
        let wf_fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if wf_fps.is_empty() {
            return Vec::new();
        }

        // Average duration per step
        let mut step_totals: HashMap<&str, (u64, usize)> = HashMap::new();
        let mut total_workflow_time: u64 = 0;

        for fp in &wf_fps {
            total_workflow_time += fp.total_duration_ms;
            for (step_id, dur) in &fp.step_durations {
                let entry = step_totals.entry(step_id.as_str()).or_insert((0, 0));
                entry.0 += dur;
                entry.1 += 1;
            }
        }

        let mut bottlenecks: Vec<Bottleneck> = step_totals
            .into_iter()
            .map(|(step_id, (total, count))| {
                let avg = total as f64 / count as f64;
                let pct = if total_workflow_time > 0 {
                    (total as f64 / total_workflow_time as f64) * 100.0
                } else {
                    0.0
                };
                Bottleneck {
                    step_id: step_id.to_string(),
                    avg_duration_ms: avg as u64,
                    percent_of_total: pct,
                }
            })
            .collect();

        bottlenecks.sort_by(|a, b| b.percent_of_total.partial_cmp(&a.percent_of_total).unwrap());
        bottlenecks
    }

    /// Get fingerprints for a workflow.
    pub fn get_fingerprints(&self, workflow_id: &str) -> Vec<&ExecutionFingerprint> {
        self.fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect()
    }
}

impl Default for ArchaeologyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Comparison between two executions.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionComparison {
    pub execution_a: String,
    pub execution_b: String,
    pub duration_a_ms: u64,
    pub duration_b_ms: u64,
    pub duration_ratio: f64,
    pub significant_step_diffs: Vec<StepDiff>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct StepDiff {
    pub step_id: String,
    pub duration_a_ms: u64,
    pub duration_b_ms: u64,
    pub ratio: f64,
}

/// An execution anomaly.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Anomaly {
    pub execution_id: String,
    pub metric: String,
    pub actual: f64,
    pub expected: f64,
    pub deviation_factor: f64,
}

/// A bottleneck step.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Bottleneck {
    pub step_id: String,
    pub avg_duration_ms: u64,
    pub percent_of_total: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_anomaly_detection() {
        let mut engine = ArchaeologyEngine::new();

        for i in 0..5 {
            engine.record_fingerprint(ExecutionFingerprint {
                execution_id: format!("exec-{}", i),
                workflow_id: "wf-1".to_string(),
                total_duration_ms: 1000,
                step_durations: HashMap::new(),
                step_outcomes: HashMap::new(),
                retry_count: 0,
                completed_at: Utc::now(),
            });
        }

        // Add an anomalous execution
        engine.record_fingerprint(ExecutionFingerprint {
            execution_id: "exec-outlier".to_string(),
            workflow_id: "wf-1".to_string(),
            total_duration_ms: 100_000,
            step_durations: HashMap::new(),
            step_outcomes: HashMap::new(),
            retry_count: 0,
            completed_at: Utc::now(),
        });

        let anomalies = engine.detect_anomalies("wf-1");
        assert!(!anomalies.is_empty());
    }
}
