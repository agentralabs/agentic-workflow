use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::format::{AwfReader, AwfWriter};
use crate::types::{Workflow, WorkflowError, WorkflowResult};

/// Persistent workflow store backed by .awf files.
pub struct WorkflowStore {
    path: PathBuf,
    workflows: HashMap<String, Workflow>,
    dirty: bool,
    auto_save: bool,
}

impl WorkflowStore {
    /// Open or create a workflow store at the given path.
    pub fn open(path: impl AsRef<Path>) -> WorkflowResult<Self> {
        let path = path.as_ref().to_path_buf();
        let mut store = Self {
            path: path.clone(),
            workflows: HashMap::new(),
            dirty: false,
            auto_save: true,
        };

        if path.exists() {
            store.load()?;
            eprintln!("WorkflowStore: loaded {} workflows from {}", store.workflows.len(), path.display());
        } else {
            eprintln!("WorkflowStore: created new store at {}", path.display());
        }

        Ok(store)
    }

    /// Open an in-memory store (no persistence).
    pub fn open_memory() -> Self {
        Self {
            path: PathBuf::new(),
            workflows: HashMap::new(),
            dirty: false,
            auto_save: false,
        }
    }

    /// Load workflows from .awf file.
    fn load(&mut self) -> WorkflowResult<()> {
        let file = std::fs::File::open(&self.path)?;
        let mut reader = AwfReader::new(file);
        reader.read_header()?;

        // Read all workflow sections
        loop {
            match reader.read_workflow() {
                Ok(wf) => {
                    self.workflows.insert(wf.id.clone(), wf);
                }
                Err(WorkflowError::IoError(_)) => break, // EOF
                Err(e) => {
                    eprintln!("WorkflowStore: error reading workflow: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Save all workflows to .awf file.
    pub fn save(&mut self) -> WorkflowResult<()> {
        if self.path.as_os_str().is_empty() {
            return Ok(()); // Memory-only store
        }

        let file = std::fs::File::create(&self.path)?;
        let mut writer = AwfWriter::new(file);
        writer.write_header()?;

        for wf in self.workflows.values() {
            writer.write_workflow(wf)?;
        }

        writer.finish()?;
        self.dirty = false;
        eprintln!("WorkflowStore: saved {} workflows to {}", self.workflows.len(), self.path.display());
        Ok(())
    }

    /// Auto-save if dirty and auto_save is enabled.
    fn maybe_auto_save(&mut self) {
        if self.dirty && self.auto_save {
            if let Err(e) = self.save() {
                eprintln!("WorkflowStore: auto-save failed: {}", e);
            }
        }
    }

    /// Add a workflow.
    pub fn insert(&mut self, workflow: Workflow) -> WorkflowResult<()> {
        self.workflows.insert(workflow.id.clone(), workflow);
        self.dirty = true;
        self.maybe_auto_save();
        Ok(())
    }

    /// Get a workflow by ID.
    pub fn get(&self, id: &str) -> WorkflowResult<&Workflow> {
        self.workflows
            .get(id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound(id.to_string()))
    }

    /// Remove a workflow.
    pub fn remove(&mut self, id: &str) -> WorkflowResult<Workflow> {
        let wf = self.workflows
            .remove(id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound(id.to_string()))?;
        self.dirty = true;
        self.maybe_auto_save();
        Ok(wf)
    }

    /// List all workflows.
    pub fn list(&self) -> Vec<&Workflow> {
        self.workflows.values().collect()
    }

    /// Count of stored workflows.
    pub fn count(&self) -> usize {
        self.workflows.len()
    }

    /// Check if store has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Set auto-save behavior.
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save = enabled;
    }

    /// Get the store file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for WorkflowStore {
    fn drop(&mut self) {
        if self.dirty && self.auto_save {
            if let Err(e) = self.save() {
                eprintln!("WorkflowStore: drop save failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StepNode;
    use crate::types::StepType;

    #[test]
    fn test_store_memory_roundtrip() {
        let mut store = WorkflowStore::open_memory();
        let wf = Workflow::new("test-wf", "A test");
        let wfid = wf.id.clone();
        store.insert(wf).unwrap();

        assert_eq!(store.count(), 1);
        assert_eq!(store.get(&wfid).unwrap().name, "test-wf");
        store.remove(&wfid).unwrap();
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn test_store_file_persistence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.awf");

        // Write
        {
            let mut store = WorkflowStore::open(&path).unwrap();
            store.set_auto_save(false);
            let mut wf = Workflow::new("persist", "Persistent workflow");
            wf.add_step(StepNode::new("S1", StepType::Noop));
            store.insert(wf).unwrap();
            store.save().unwrap();
        }

        // Read back
        {
            let store = WorkflowStore::open(&path).unwrap();
            assert_eq!(store.count(), 1);
            let wf = store.list()[0];
            assert_eq!(wf.name, "persist");
            assert_eq!(wf.steps.len(), 1);
        }
    }

    #[test]
    fn test_store_auto_save_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("autosave.awf");

        {
            let mut store = WorkflowStore::open(&path).unwrap();
            store.insert(Workflow::new("auto", "Auto-saved")).unwrap();
            // Drop triggers save
        }

        let store = WorkflowStore::open(&path).unwrap();
        assert_eq!(store.count(), 1);
    }
}
