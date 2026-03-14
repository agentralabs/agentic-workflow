use std::io::Read;

use crate::types::{Workflow, WorkflowError, WorkflowResult};

use super::writer::{AWF_MAGIC, AWF_VERSION, SectionType};

/// Reads .awf binary files.
pub struct AwfReader<R: Read> {
    reader: R,
    version: u32,
    workflow_count: u32,
    execution_count: u32,
}

impl<R: Read> AwfReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            version: 0,
            workflow_count: 0,
            execution_count: 0,
        }
    }

    /// Read and validate the file header.
    pub fn read_header(&mut self) -> WorkflowResult<()> {
        let mut magic = [0u8; 4];
        self.reader.read_exact(&mut magic)?;

        if &magic != AWF_MAGIC {
            return Err(WorkflowError::FormatError(
                "Invalid .awf file: bad magic bytes".to_string(),
            ));
        }

        let mut version_bytes = [0u8; 4];
        self.reader.read_exact(&mut version_bytes)?;
        self.version = u32::from_le_bytes(version_bytes);

        if self.version > AWF_VERSION {
            return Err(WorkflowError::FormatError(format!(
                "Unsupported .awf version: {} (max: {})",
                self.version, AWF_VERSION
            )));
        }

        let mut wf_count = [0u8; 4];
        self.reader.read_exact(&mut wf_count)?;
        self.workflow_count = u32::from_le_bytes(wf_count);

        let mut exec_count = [0u8; 4];
        self.reader.read_exact(&mut exec_count)?;
        self.execution_count = u32::from_le_bytes(exec_count);

        Ok(())
    }

    /// Read a section header.
    pub fn read_section_header(&mut self) -> WorkflowResult<(u8, u32)> {
        let mut section_type = [0u8; 1];
        self.reader.read_exact(&mut section_type)?;

        let mut data_len = [0u8; 4];
        self.reader.read_exact(&mut data_len)?;

        Ok((section_type[0], u32::from_le_bytes(data_len)))
    }

    /// Read section data and verify BLAKE3 checksum.
    pub fn read_section_data(&mut self, data_len: u32) -> WorkflowResult<Vec<u8>> {
        let mut data = vec![0u8; data_len as usize];
        self.reader.read_exact(&mut data)?;

        let mut stored_checksum = [0u8; 32];
        self.reader.read_exact(&mut stored_checksum)?;

        let computed_checksum = blake3::hash(&data);
        if computed_checksum.as_bytes() != &stored_checksum {
            return Err(WorkflowError::FormatError(
                "Section checksum mismatch — data corrupted".to_string(),
            ));
        }

        Ok(data)
    }

    /// Read a workflow from section data.
    pub fn read_workflow(&mut self) -> WorkflowResult<Workflow> {
        let (section_type, data_len) = self.read_section_header()?;

        if section_type != SectionType::WorkflowRegistry as u8 {
            return Err(WorkflowError::FormatError(format!(
                "Expected WorkflowRegistry section, got type {}",
                section_type
            )));
        }

        let data = self.read_section_data(data_len)?;
        let workflow: Workflow = serde_json::from_slice(&data)
            .map_err(|e| WorkflowError::SerializationError(e.to_string()))?;

        Ok(workflow)
    }

    /// Read a JSON section.
    pub fn read_json_section(&mut self) -> WorkflowResult<(u8, serde_json::Value)> {
        let (section_type, data_len) = self.read_section_header()?;
        let data = self.read_section_data(data_len)?;

        let value: serde_json::Value = serde_json::from_slice(&data)
            .map_err(|e| WorkflowError::SerializationError(e.to_string()))?;

        Ok((section_type, value))
    }

    /// Get file version.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Get workflow count from header.
    pub fn workflow_count(&self) -> u32 {
        self.workflow_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::AwfWriter;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip() {
        // Write
        let mut buf = Vec::new();
        {
            let mut writer = AwfWriter::new(&mut buf);
            writer.write_header().unwrap();

            let wf = Workflow::new("roundtrip", "Test roundtrip");
            writer.write_workflow(&wf).unwrap();
            writer.finish().unwrap();
        }

        // Read
        let cursor = Cursor::new(buf);
        let mut reader = AwfReader::new(cursor);
        reader.read_header().unwrap();

        let wf = reader.read_workflow().unwrap();
        assert_eq!(wf.name, "roundtrip");
    }
}
