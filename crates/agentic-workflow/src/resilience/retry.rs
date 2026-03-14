use std::collections::HashMap;

use uuid::Uuid;

use crate::types::{
    FailureClass, RetryBudget, RetryPattern, RetryPolicy, RetryProfile,
    RetryStats, RetryStrategy, WorkflowError, WorkflowResult,
};

/// Failure-classified retry engine.
pub struct RetryEngine {
    policies: HashMap<String, RetryPolicy>,
    stats: HashMap<String, RetryStats>,
}

impl RetryEngine {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            stats: HashMap::new(),
        }
    }

    /// Configure a retry policy.
    pub fn configure_policy(
        &mut self,
        name: &str,
        profiles: Vec<RetryProfile>,
        budget: Option<RetryBudget>,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let policy = RetryPolicy {
            id: id.clone(),
            name: name.to_string(),
            profiles,
            budget,
            escalation: None,
        };

        self.policies.insert(id.clone(), policy);
        Ok(id)
    }

    /// Get a retry policy.
    pub fn get_policy(&self, policy_id: &str) -> WorkflowResult<&RetryPolicy> {
        self.policies
            .get(policy_id)
            .ok_or_else(|| WorkflowError::Internal(format!("Policy not found: {}", policy_id)))
    }

    /// Get retry profile for a specific failure class.
    pub fn get_profile_for_failure(
        &self,
        policy_id: &str,
        failure_class: &FailureClass,
    ) -> WorkflowResult<Option<&RetryProfile>> {
        let policy = self.get_policy(policy_id)?;
        Ok(policy
            .profiles
            .iter()
            .find(|p| p.failure_class == *failure_class))
    }

    /// Calculate delay for next retry attempt.
    pub fn calculate_delay(
        &self,
        strategy: &RetryStrategy,
        attempt: u32,
    ) -> u64 {
        match strategy {
            RetryStrategy::Immediate => 0,
            RetryStrategy::FixedDelay { delay_ms } => *delay_ms,
            RetryStrategy::ExponentialBackoff {
                initial_ms,
                max_ms,
                multiplier,
            } => {
                let delay = (*initial_ms as f64) * multiplier.powi(attempt as i32);
                (delay as u64).min(*max_ms)
            }
            RetryStrategy::Linear {
                delay_ms,
                increment_ms,
            } => delay_ms + (increment_ms * attempt as u64),
        }
    }

    /// Check if retry is within budget.
    pub fn within_budget(
        &self,
        policy_id: &str,
        step_id: &str,
    ) -> WorkflowResult<bool> {
        let policy = self.get_policy(policy_id)?;

        if let Some(budget) = &policy.budget {
            if let Some(stats) = self.stats.get(step_id) {
                if let Some(max) = budget.max_total_attempts {
                    if stats.total_attempts >= max {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Record a retry attempt.
    pub fn record_attempt(
        &mut self,
        step_id: &str,
        failure_class: FailureClass,
    ) {
        let stats = self.stats.entry(step_id.to_string()).or_insert_with(|| {
            RetryStats {
                step_id: step_id.to_string(),
                total_attempts: 0,
                successes_by_attempt: Vec::new(),
                avg_delay_ms: 0.0,
                last_failure_class: None,
                last_retry_at: None,
            }
        });

        stats.total_attempts += 1;
        stats.last_failure_class = Some(failure_class);
        stats.last_retry_at = Some(chrono::Utc::now());
    }

    /// Get retry stats for a step.
    pub fn get_stats(&self, step_id: &str) -> Option<&RetryStats> {
        self.stats.get(step_id)
    }

    /// Get learned retry patterns.
    pub fn get_patterns(&self) -> Vec<RetryPattern> {
        self.stats
            .values()
            .map(|s| RetryPattern {
                step_id: s.step_id.clone(),
                optimal_delay_ms: s.avg_delay_ms as u64,
                success_rate_by_attempt: s
                    .successes_by_attempt
                    .iter()
                    .map(|&v| v as f64)
                    .collect(),
                recommendation: if s.total_attempts > 10 {
                    "Consider optimizing retry strategy based on patterns".to_string()
                } else {
                    "Insufficient data for recommendation".to_string()
                },
            })
            .collect()
    }

    /// List all policies.
    pub fn list_policies(&self) -> Vec<&RetryPolicy> {
        self.policies.values().collect()
    }
}

impl Default for RetryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let engine = RetryEngine::new();
        let strategy = RetryStrategy::ExponentialBackoff {
            initial_ms: 100,
            max_ms: 10000,
            multiplier: 2.0,
        };

        assert_eq!(engine.calculate_delay(&strategy, 0), 100);
        assert_eq!(engine.calculate_delay(&strategy, 1), 200);
        assert_eq!(engine.calculate_delay(&strategy, 2), 400);
        assert_eq!(engine.calculate_delay(&strategy, 10), 10000); // capped
    }
}
