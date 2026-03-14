#[cfg(feature = "format")]
use lz4_flex::{compress_prepend_size, decompress_size_prepended};

use crate::types::{WorkflowError, WorkflowResult};

/// Compress data using LZ4.
#[cfg(feature = "format")]
pub fn compress(data: &[u8]) -> Vec<u8> {
    compress_prepend_size(data)
}

/// Decompress LZ4 data.
#[cfg(feature = "format")]
pub fn decompress(data: &[u8]) -> WorkflowResult<Vec<u8>> {
    decompress_size_prepended(data)
        .map_err(|e| WorkflowError::FormatError(format!("Decompression failed: {}", e)))
}

/// No-op compress when format feature is disabled.
#[cfg(not(feature = "format"))]
pub fn compress(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

/// No-op decompress when format feature is disabled.
#[cfg(not(feature = "format"))]
pub fn decompress(data: &[u8]) -> WorkflowResult<Vec<u8>> {
    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"Hello, AgenticWorkflow! This is a test of compression.";
        let compressed = compress(original);
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }
}
