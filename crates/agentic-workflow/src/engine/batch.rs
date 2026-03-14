use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    BatchItem, BatchItemStatus, BatchJob, BatchProgress, BatchReport, BatchStatus,
    WorkflowError, WorkflowResult,
};

/// Batch processing engine with controlled parallelism.
pub struct BatchEngine {
    jobs: HashMap<String, BatchJob>,
}

impl BatchEngine {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
        }
    }

    /// Create a batch job from a list of items.
    pub fn create_batch(
        &mut self,
        workflow_id: &str,
        items: Vec<serde_json::Value>,
        concurrency: usize,
        checkpoint_every: usize,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let batch_items: Vec<BatchItem> = items
            .into_iter()
            .enumerate()
            .map(|(i, input)| BatchItem {
                index: i,
                input,
                status: BatchItemStatus::Pending,
                output: None,
                error: None,
                duration_ms: None,
            })
            .collect();

        let job = BatchJob {
            id: id.clone(),
            workflow_id: workflow_id.to_string(),
            items: batch_items,
            concurrency: concurrency.max(1),
            checkpoint_every: checkpoint_every.max(1),
            status: BatchStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        };

        self.jobs.insert(id.clone(), job);
        Ok(id)
    }

    /// Get batch progress.
    pub fn get_progress(&self, batch_id: &str) -> WorkflowResult<BatchProgress> {
        let job = self
            .jobs
            .get(batch_id)
            .ok_or_else(|| WorkflowError::BatchError(format!("Not found: {}", batch_id)))?;

        let total = job.items.len();
        let completed = job.items.iter().filter(|i| i.status == BatchItemStatus::Success).count();
        let failed = job.items.iter().filter(|i| i.status == BatchItemStatus::Failed).count();
        let skipped = job.items.iter().filter(|i| i.status == BatchItemStatus::Skipped).count();
        let running = job.items.iter().filter(|i| i.status == BatchItemStatus::Running).count();
        let pending = job.items.iter().filter(|i| i.status == BatchItemStatus::Pending).count();

        let percent = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        // Find last checkpoint
        let last_checkpoint = job
            .items
            .iter()
            .filter(|i| i.status == BatchItemStatus::Success)
            .map(|i| i.index)
            .max();

        Ok(BatchProgress {
            batch_id: batch_id.to_string(),
            total_items: total,
            completed,
            failed,
            skipped,
            running,
            pending,
            percent_complete: percent,
            estimated_remaining_ms: None,
            last_checkpoint_index: last_checkpoint,
        })
    }

    /// Generate batch completion report.
    pub fn get_report(&self, batch_id: &str) -> WorkflowResult<BatchReport> {
        let job = self
            .jobs
            .get(batch_id)
            .ok_or_else(|| WorkflowError::BatchError(format!("Not found: {}", batch_id)))?;

        let success_count = job.items.iter().filter(|i| i.status == BatchItemStatus::Success).count();
        let fail_count = job.items.iter().filter(|i| i.status == BatchItemStatus::Failed).count();
        let skip_count = job.items.iter().filter(|i| i.status == BatchItemStatus::Skipped).count();

        let total_duration: u64 = job
            .items
            .iter()
            .filter_map(|i| i.duration_ms)
            .sum();

        let processed = success_count + fail_count;
        let avg = if processed > 0 {
            total_duration as f64 / processed as f64
        } else {
            0.0
        };

        // Group errors by pattern
        let mut error_groups: HashMap<String, Vec<usize>> = HashMap::new();
        for item in &job.items {
            if let Some(err) = &item.error {
                error_groups
                    .entry(err.clone())
                    .or_default()
                    .push(item.index);
            }
        }

        let error_summary = error_groups
            .into_iter()
            .map(|(pattern, indices)| crate::types::batch::BatchErrorGroup {
                error_pattern: pattern,
                count: indices.len(),
                sample_indices: indices.into_iter().take(5).collect(),
            })
            .collect();

        Ok(BatchReport {
            batch_id: batch_id.to_string(),
            total_items: job.items.len(),
            success_count,
            fail_count,
            skip_count,
            total_duration_ms: total_duration,
            avg_item_duration_ms: avg,
            error_summary,
        })
    }

    /// Get a batch job.
    pub fn get_job(&self, batch_id: &str) -> WorkflowResult<&BatchJob> {
        self.jobs
            .get(batch_id)
            .ok_or_else(|| WorkflowError::BatchError(format!("Not found: {}", batch_id)))
    }
}

impl Default for BatchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_creation() {
        let mut engine = BatchEngine::new();
        let items = vec![
            serde_json::json!({"id": 1}),
            serde_json::json!({"id": 2}),
            serde_json::json!({"id": 3}),
        ];

        let bid = engine.create_batch("wf-1", items, 2, 10).unwrap();
        let progress = engine.get_progress(&bid).unwrap();
        assert_eq!(progress.total_items, 3);
        assert_eq!(progress.pending, 3);
        assert_eq!(progress.percent_complete, 0.0);
    }
}
