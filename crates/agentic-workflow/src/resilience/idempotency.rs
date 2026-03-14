use std::collections::HashMap;

use chrono::Utc;

use crate::types::{
    ConflictResolution, IdempotencyConfig, IdempotencyEntry, IdempotencyReport,
    IdempotencyWindow, KeyStrategy, StepIdempotencyStats,
    WorkflowError, WorkflowResult,
};

/// Idempotency engine — deduplication for step executions.
pub struct IdempotencyEngine {
    configs: HashMap<String, IdempotencyConfig>,
    cache: HashMap<String, IdempotencyEntry>,
    hit_counts: HashMap<String, u64>,
}

impl IdempotencyEngine {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            cache: HashMap::new(),
            hit_counts: HashMap::new(),
        }
    }

    /// Configure idempotency for a step.
    pub fn configure(
        &mut self,
        step_id: &str,
        key_strategy: KeyStrategy,
        window: IdempotencyWindow,
        conflict_resolution: ConflictResolution,
    ) -> WorkflowResult<()> {
        let config = IdempotencyConfig {
            step_id: step_id.to_string(),
            key_strategy,
            window,
            conflict_resolution,
        };

        self.configs.insert(step_id.to_string(), config);
        Ok(())
    }

    /// Compute idempotency key for a step execution.
    pub fn compute_key(
        &self,
        step_id: &str,
        workflow_id: &str,
        input: &serde_json::Value,
    ) -> WorkflowResult<String> {
        let config = self.configs.get(step_id);

        match config.map(|c| &c.key_strategy) {
            Some(KeyStrategy::InputHash) | None => {
                let input_str = serde_json::to_string(input)
                    .map_err(|e| WorkflowError::SerializationError(e.to_string()))?;
                let hash = blake3::hash(input_str.as_bytes());
                Ok(format!("{}:{}:{}", workflow_id, step_id, hash.to_hex()))
            }
            Some(KeyStrategy::Expression(expr)) => {
                Ok(format!("{}:{}:{}", workflow_id, step_id, expr))
            }
            Some(KeyStrategy::FieldPath(path)) => {
                let field_value = input
                    .pointer(path)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                Ok(format!("{}:{}:{}", workflow_id, step_id, field_value))
            }
        }
    }

    /// Check if a key was already processed.
    pub fn check(&self, key: &str) -> Option<&IdempotencyEntry> {
        let entry = self.cache.get(key)?;

        // Check expiration
        if let Some(expires_at) = entry.expires_at {
            if Utc::now() > expires_at {
                return None;
            }
        }

        Some(entry)
    }

    /// Store execution result for deduplication.
    pub fn store(
        &mut self,
        key: String,
        step_id: &str,
        execution_id: &str,
        input_hash: &str,
        output: serde_json::Value,
    ) -> WorkflowResult<()> {
        let config = self.configs.get(step_id);
        let now = Utc::now();

        let expires_at = match config.map(|c| &c.window) {
            Some(IdempotencyWindow::Duration { ms }) => {
                Some(now + chrono::Duration::milliseconds(*ms as i64))
            }
            Some(IdempotencyWindow::Forever) | None => None,
            Some(IdempotencyWindow::UntilNextExecution) => None,
        };

        let entry = IdempotencyEntry {
            key: key.clone(),
            step_id: step_id.to_string(),
            execution_id: execution_id.to_string(),
            input_hash: input_hash.to_string(),
            output,
            created_at: now,
            expires_at,
        };

        self.cache.insert(key, entry);
        Ok(())
    }

    /// Record a cache hit.
    pub fn record_hit(&mut self, step_id: &str) {
        *self.hit_counts.entry(step_id.to_string()).or_insert(0) += 1;
    }

    /// Purge expired entries.
    pub fn purge_expired(&mut self) -> usize {
        let now = Utc::now();
        let before = self.cache.len();
        self.cache.retain(|_, entry| {
            entry.expires_at.map_or(true, |exp| exp > now)
        });
        before - self.cache.len()
    }

    /// Get deduplication report.
    pub fn report(&self) -> IdempotencyReport {
        let mut by_step: HashMap<&str, (usize, u64)> = HashMap::new();

        for entry in self.cache.values() {
            by_step.entry(&entry.step_id).or_insert((0, 0)).0 += 1;
        }

        for (step_id, hits) in &self.hit_counts {
            by_step.entry(step_id).or_insert((0, 0)).1 = *hits;
        }

        let total_hits: u64 = self.hit_counts.values().sum();
        let total_checks = total_hits + self.cache.len() as u64;
        let hit_rate = if total_checks > 0 {
            total_hits as f64 / total_checks as f64
        } else {
            0.0
        };

        let stats: Vec<StepIdempotencyStats> = by_step
            .into_iter()
            .map(|(step_id, (entries, hits))| StepIdempotencyStats {
                step_id: step_id.to_string(),
                entries,
                hits,
                saved_executions: hits,
            })
            .collect();

        IdempotencyReport {
            total_entries: self.cache.len(),
            deduplicated_count: total_hits,
            cache_hit_rate: hit_rate,
            oldest_entry: self.cache.values().map(|e| e.created_at).min(),
            by_step: stats,
        }
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for IdempotencyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency_dedup() {
        let mut engine = IdempotencyEngine::new();
        let key = engine
            .compute_key("step-1", "wf-1", &serde_json::json!({"x": 1}))
            .unwrap();

        assert!(engine.check(&key).is_none());

        engine
            .store(key.clone(), "step-1", "exec-1", "abc", serde_json::json!({"result": 42}))
            .unwrap();

        assert!(engine.check(&key).is_some());
        assert_eq!(
            engine.check(&key).unwrap().output,
            serde_json::json!({"result": 42})
        );
    }
}
