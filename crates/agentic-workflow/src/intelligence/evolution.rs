use crate::types::{
    ExecutionFingerprint, HealthIssue, IssueSeverity, WorkflowHealth,
    WorkflowResult,
};

/// Workflow evolution engine — health monitoring and optimization.
pub struct EvolutionEngine {
    fingerprints: Vec<ExecutionFingerprint>,
}

impl EvolutionEngine {
    pub fn new() -> Self {
        Self {
            fingerprints: Vec::new(),
        }
    }

    /// Ingest execution data.
    pub fn ingest(&mut self, fp: ExecutionFingerprint) {
        self.fingerprints.push(fp);
    }

    /// Get workflow health score.
    pub fn health(&self, workflow_id: &str) -> WorkflowResult<WorkflowHealth> {
        let fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if fps.is_empty() {
            return Ok(WorkflowHealth {
                workflow_id: workflow_id.to_string(),
                score: 1.0,
                success_rate: 1.0,
                avg_duration_ms: 0,
                drift_detected: false,
                issues: Vec::new(),
            });
        }

        let success_count = fps.iter().filter(|f| f.retry_count == 0).count();
        let success_rate = success_count as f64 / fps.len() as f64;

        let avg_duration = fps.iter().map(|f| f.total_duration_ms).sum::<u64>() / fps.len() as u64;

        let drift_detected = self.detect_drift(workflow_id);
        let mut issues = Vec::new();

        if success_rate < 0.8 {
            issues.push(HealthIssue {
                severity: IssueSeverity::Critical,
                step_id: None,
                description: format!("Success rate is {:.0}%", success_rate * 100.0),
                suggestion: "Review failing steps and retry policies".to_string(),
            });
        }

        if drift_detected {
            issues.push(HealthIssue {
                severity: IssueSeverity::Warning,
                step_id: None,
                description: "Performance drift detected".to_string(),
                suggestion: "Recent executions are taking longer than historical average"
                    .to_string(),
            });
        }

        let score = success_rate * if drift_detected { 0.8 } else { 1.0 };

        Ok(WorkflowHealth {
            workflow_id: workflow_id.to_string(),
            score,
            success_rate,
            avg_duration_ms: avg_duration,
            drift_detected,
            issues,
        })
    }

    /// Detect performance drift (recent executions significantly slower).
    pub fn detect_drift(&self, workflow_id: &str) -> bool {
        let fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if fps.len() < 6 {
            return false;
        }

        let split = fps.len() / 2;
        let old_avg: f64 = fps[..split]
            .iter()
            .map(|f| f.total_duration_ms as f64)
            .sum::<f64>()
            / split as f64;

        let new_avg: f64 = fps[split..]
            .iter()
            .map(|f| f.total_duration_ms as f64)
            .sum::<f64>()
            / (fps.len() - split) as f64;

        // Drift if recent is >50% slower
        new_avg > old_avg * 1.5
    }

    /// Suggest optimizations.
    pub fn suggest_optimizations(&self, workflow_id: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        let fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if fps.is_empty() {
            return suggestions;
        }

        let avg_retries: f64 =
            fps.iter().map(|f| f.retry_count as f64).sum::<f64>() / fps.len() as f64;

        if avg_retries > 2.0 {
            suggestions.push(format!(
                "Average retry count is {:.1} — consider optimizing retry policies",
                avg_retries
            ));
        }

        if self.detect_drift(workflow_id) {
            suggestions.push(
                "Performance is drifting upward — investigate recent step duration increases"
                    .to_string(),
            );
        }

        suggestions
    }

    /// Identify potentially outdated steps.
    pub fn outdated_steps(&self, workflow_id: &str) -> Vec<String> {
        let fps: Vec<&ExecutionFingerprint> = self
            .fingerprints
            .iter()
            .filter(|f| f.workflow_id == workflow_id)
            .collect();

        if fps.len() < 5 {
            return Vec::new();
        }

        // Find steps with increasing failure rates
        let recent = &fps[fps.len().saturating_sub(5)..];
        let mut step_fail_rates: std::collections::HashMap<&str, usize> =
            std::collections::HashMap::new();

        for fp in recent {
            for (sid, outcome) in &fp.step_outcomes {
                if *outcome == crate::types::StepLifecycle::Failed {
                    *step_fail_rates.entry(sid.as_str()).or_insert(0) += 1;
                }
            }
        }

        step_fail_rates
            .into_iter()
            .filter(|(_, fails)| *fails >= 3)
            .map(|(sid, _)| sid.to_string())
            .collect()
    }
}

impl Default for EvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}
