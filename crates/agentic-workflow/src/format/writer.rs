use std::io::Write;

use crate::types::{Workflow, WorkflowError, WorkflowResult};

/// Magic bytes for .awf file format.
pub const AWF_MAGIC: &[u8; 4] = b"AWFL";
/// Current format version.
pub const AWF_VERSION: u32 = 1;

/// Section types in the .awf file.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SectionType {
    WorkflowRegistry = 1,
    TemplateLibrary = 2,
    ExecutionHistory = 3,
    ScheduleTable = 4,
    StateMachineTable = 5,
    TriggerIndex = 6,
    AuditLog = 7,
    IdempotencyCache = 8,
    VariableStore = 9,
}

/// Writes .awf binary files.
pub struct AwfWriter<W: Write> {
    writer: W,
    workflow_count: u32,
    execution_count: u32,
}

impl<W: Write> AwfWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            workflow_count: 0,
            execution_count: 0,
        }
    }

    /// Write the file header.
    pub fn write_header(&mut self) -> WorkflowResult<()> {
        // Magic
        self.writer.write_all(AWF_MAGIC)?;
        // Version (little-endian)
        self.writer.write_all(&AWF_VERSION.to_le_bytes())?;
        // Workflow count (placeholder — patched at end)
        self.writer.write_all(&0u32.to_le_bytes())?;
        // Execution count (placeholder — patched at end)
        self.writer.write_all(&0u32.to_le_bytes())?;

        Ok(())
    }

    /// Write a section header.
    pub fn write_section_header(
        &mut self,
        section_type: SectionType,
        data_len: u32,
    ) -> WorkflowResult<()> {
        self.writer.write_all(&[section_type as u8])?;
        self.writer.write_all(&data_len.to_le_bytes())?;
        Ok(())
    }

    /// Write a workflow to the registry section.
    pub fn write_workflow(&mut self, workflow: &Workflow) -> WorkflowResult<()> {
        let json = serde_json::to_vec(workflow)
            .map_err(|e| WorkflowError::SerializationError(e.to_string()))?;

        let checksum = blake3::hash(&json);

        self.write_section_header(SectionType::WorkflowRegistry, json.len() as u32)?;
        self.writer.write_all(&json)?;
        self.writer.write_all(checksum.as_bytes())?;

        self.workflow_count += 1;
        Ok(())
    }

    /// Write raw JSON data as a section.
    pub fn write_json_section(
        &mut self,
        section_type: SectionType,
        data: &serde_json::Value,
    ) -> WorkflowResult<()> {
        let json = serde_json::to_vec(data)
            .map_err(|e| WorkflowError::SerializationError(e.to_string()))?;

        let checksum = blake3::hash(&json);

        self.write_section_header(section_type, json.len() as u32)?;
        self.writer.write_all(&json)?;
        self.writer.write_all(checksum.as_bytes())?;

        Ok(())
    }

    /// Finish writing and flush.
    pub fn finish(mut self) -> WorkflowResult<W> {
        self.writer.flush()?;
        Ok(self.writer)
    }

    /// Get counts.
    pub fn workflow_count(&self) -> u32 {
        self.workflow_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_header() {
        let mut buf = Vec::new();
        let mut writer = AwfWriter::new(&mut buf);
        writer.write_header().unwrap();
        assert_eq!(&buf[0..4], AWF_MAGIC);
    }

    #[test]
    fn test_write_workflow() {
        let mut buf = Vec::new();
        {
            let mut writer = AwfWriter::new(&mut buf);
            writer.write_header().unwrap();

            let wf = Workflow::new("test", "A test workflow");
            writer.write_workflow(&wf).unwrap();
            assert_eq!(writer.workflow_count(), 1);
            writer.finish().unwrap();
        }

        assert!(buf.len() > 16);
    }
}
