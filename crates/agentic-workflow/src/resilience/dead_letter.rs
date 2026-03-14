use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    DeadLetterItem, DeadLetterPolicy, DeadLetterSummary, FailureGroup,
    WorkflowError, WorkflowResult,
};

/// Intelligent dead letter management engine.
pub struct DeadLetterEngine {
    items: HashMap<String, DeadLetterItem>,
    policy: DeadLetterPolicy,
}

impl DeadLetterEngine {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            policy: DeadLetterPolicy {
                retention_days: 30,
                auto_resurrect_on_recovery: true,
                max_items: Some(10_000),
                alert_threshold: Some(100),
            },
        }
    }

    /// Add a failed item to the dead letter queue.
    pub fn add_item(
        &mut self,
        execution_id: &str,
        workflow_id: &str,
        step_id: &str,
        failure_class: &str,
        error_message: &str,
        input_data: serde_json::Value,
        attempt_count: u32,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let expires_at = chrono::Duration::days(self.policy.retention_days as i64)
            .checked_add(&chrono::Duration::zero())
            .map(|d| now + d);

        let resurrectable = failure_class != "permanent" && failure_class != "authentication";

        let item = DeadLetterItem {
            id: id.clone(),
            execution_id: execution_id.to_string(),
            workflow_id: workflow_id.to_string(),
            step_id: step_id.to_string(),
            failure_class: failure_class.to_string(),
            error_message: error_message.to_string(),
            input_data,
            attempt_count,
            failed_at: now,
            expires_at,
            resurrectable,
        };

        self.items.insert(id.clone(), item);

        // Check alert threshold
        if let Some(threshold) = self.policy.alert_threshold {
            if self.items.len() >= threshold {
                eprintln!(
                    "Dead letter queue alert: {} items (threshold: {})",
                    self.items.len(),
                    threshold
                );
            }
        }

        Ok(id)
    }

    /// List all dead letter items.
    pub fn list_items(&self) -> Vec<&DeadLetterItem> {
        self.items.values().collect()
    }

    /// Get a summary grouped by failure class.
    pub fn summary(&self) -> DeadLetterSummary {
        let mut groups: HashMap<&str, (usize, bool, String)> = HashMap::new();

        for item in self.items.values() {
            let entry = groups
                .entry(&item.failure_class)
                .or_insert((0, item.resurrectable, item.error_message.clone()));
            entry.0 += 1;
        }

        let by_failure_class: Vec<FailureGroup> = groups
            .into_iter()
            .map(|(class, (count, auto_retryable, sample_error))| FailureGroup {
                failure_class: class.to_string(),
                count,
                auto_retryable,
                sample_error,
            })
            .collect();

        let auto_retryable = self.items.values().filter(|i| i.resurrectable).count();
        let needs_human = self.items.len() - auto_retryable;
        let oldest = self.items.values().map(|i| i.failed_at).min();

        DeadLetterSummary {
            total_items: self.items.len(),
            by_failure_class,
            auto_retryable,
            needs_human,
            oldest_item: oldest,
        }
    }

    /// Remove an item (after successful retry or manual resolution).
    pub fn remove_item(&mut self, item_id: &str) -> WorkflowResult<DeadLetterItem> {
        self.items.remove(item_id).ok_or_else(|| {
            WorkflowError::Internal(format!("Dead letter item not found: {}", item_id))
        })
    }

    /// Purge expired items.
    pub fn purge_expired(&mut self) -> usize {
        let now = Utc::now();
        let before = self.items.len();
        self.items.retain(|_, item| {
            item.expires_at.map_or(true, |exp| exp > now)
        });
        before - self.items.len()
    }

    /// Get items that can be auto-retried (for service recovery).
    pub fn retryable_items(&self, failure_class: &str) -> Vec<&DeadLetterItem> {
        self.items
            .values()
            .filter(|i| i.resurrectable && i.failure_class == failure_class)
            .collect()
    }

    /// Update retention policy.
    pub fn set_policy(&mut self, policy: DeadLetterPolicy) {
        self.policy = policy;
    }

    /// Get current policy.
    pub fn get_policy(&self) -> &DeadLetterPolicy {
        &self.policy
    }
}

impl Default for DeadLetterEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dead_letter_summary() {
        let mut engine = DeadLetterEngine::new();
        engine
            .add_item("e1", "w1", "s1", "rate_limit", "429", serde_json::json!({}), 3)
            .unwrap();
        engine
            .add_item("e2", "w1", "s2", "rate_limit", "429", serde_json::json!({}), 2)
            .unwrap();
        engine
            .add_item("e3", "w1", "s3", "permanent", "invalid data", serde_json::json!({}), 1)
            .unwrap();

        let summary = engine.summary();
        assert_eq!(summary.total_items, 3);
        assert_eq!(summary.auto_retryable, 2);
        assert_eq!(summary.needs_human, 1);
    }
}
