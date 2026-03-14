use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    BackpressureConfig, BackpressureStrategy, ProcessingWindow, StreamCheckpoint,
    StreamFork, StreamProcessor, StreamSource, StreamStatus,
    WorkflowError, WorkflowResult,
};

/// Unified stream processing engine.
pub struct StreamEngine {
    processors: HashMap<String, StreamProcessor>,
    checkpoints: HashMap<String, StreamCheckpoint>,
    forks: HashMap<String, Vec<StreamFork>>,
}

impl StreamEngine {
    pub fn new() -> Self {
        Self {
            processors: HashMap::new(),
            checkpoints: HashMap::new(),
            forks: HashMap::new(),
        }
    }

    /// Create a stream processor.
    pub fn create_processor(
        &mut self,
        name: &str,
        workflow_id: &str,
        source: StreamSource,
        window: Option<ProcessingWindow>,
        max_queue_size: usize,
    ) -> WorkflowResult<String> {
        let id = Uuid::new_v4().to_string();
        let processor = StreamProcessor {
            id: id.clone(),
            name: name.to_string(),
            workflow_id: workflow_id.to_string(),
            source,
            window,
            backpressure: BackpressureConfig {
                max_queue_size,
                strategy: BackpressureStrategy::SlowDown,
            },
            status: StreamStatus::Created,
            created_at: Utc::now(),
        };

        self.processors.insert(id.clone(), processor);
        Ok(id)
    }

    /// Start consuming from a stream.
    pub fn start(&mut self, stream_id: &str) -> WorkflowResult<()> {
        let proc = self
            .processors
            .get_mut(stream_id)
            .ok_or_else(|| WorkflowError::StreamError(format!("Not found: {}", stream_id)))?;

        proc.status = StreamStatus::Running;
        Ok(())
    }

    /// Pause stream consumption.
    pub fn pause(&mut self, stream_id: &str) -> WorkflowResult<()> {
        let proc = self
            .processors
            .get_mut(stream_id)
            .ok_or_else(|| WorkflowError::StreamError(format!("Not found: {}", stream_id)))?;

        proc.status = StreamStatus::Paused;
        Ok(())
    }

    /// Stop a stream processor.
    pub fn stop(&mut self, stream_id: &str) -> WorkflowResult<()> {
        let proc = self
            .processors
            .get_mut(stream_id)
            .ok_or_else(|| WorkflowError::StreamError(format!("Not found: {}", stream_id)))?;

        proc.status = StreamStatus::Stopped;
        Ok(())
    }

    /// Force checkpoint at current position.
    pub fn checkpoint(&mut self, stream_id: &str, offset: u64, items_processed: u64) -> WorkflowResult<()> {
        if !self.processors.contains_key(stream_id) {
            return Err(WorkflowError::StreamError(format!("Not found: {}", stream_id)));
        }

        let cp = StreamCheckpoint {
            stream_id: stream_id.to_string(),
            offset,
            items_processed,
            checkpoint_at: Utc::now(),
        };

        self.checkpoints.insert(stream_id.to_string(), cp);
        Ok(())
    }

    /// Add a fork to split stream by condition.
    pub fn add_fork(
        &mut self,
        stream_id: &str,
        name: &str,
        condition: &str,
        target_workflow_id: &str,
    ) -> WorkflowResult<String> {
        if !self.processors.contains_key(stream_id) {
            return Err(WorkflowError::StreamError(format!("Not found: {}", stream_id)));
        }

        let fork_id = Uuid::new_v4().to_string();
        let fork = StreamFork {
            id: fork_id.clone(),
            stream_id: stream_id.to_string(),
            condition: condition.to_string(),
            target_workflow_id: target_workflow_id.to_string(),
            name: name.to_string(),
        };

        self.forks
            .entry(stream_id.to_string())
            .or_default()
            .push(fork);

        Ok(fork_id)
    }

    /// Get processor status.
    pub fn get_processor(&self, stream_id: &str) -> WorkflowResult<&StreamProcessor> {
        self.processors
            .get(stream_id)
            .ok_or_else(|| WorkflowError::StreamError(format!("Not found: {}", stream_id)))
    }

    /// List all processors.
    pub fn list_processors(&self) -> Vec<&StreamProcessor> {
        self.processors.values().collect()
    }
}

impl Default for StreamEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_lifecycle() {
        let mut engine = StreamEngine::new();
        let sid = engine
            .create_processor(
                "file-watcher",
                "wf-1",
                StreamSource::FileWatch {
                    path: "/tmp/data".to_string(),
                    pattern: Some("*.csv".to_string()),
                },
                None,
                100,
            )
            .unwrap();

        assert_eq!(engine.get_processor(&sid).unwrap().status, StreamStatus::Created);
        engine.start(&sid).unwrap();
        assert_eq!(engine.get_processor(&sid).unwrap().status, StreamStatus::Running);
        engine.pause(&sid).unwrap();
        assert_eq!(engine.get_processor(&sid).unwrap().status, StreamStatus::Paused);
        engine.stop(&sid).unwrap();
        assert_eq!(engine.get_processor(&sid).unwrap().status, StreamStatus::Stopped);
    }
}
