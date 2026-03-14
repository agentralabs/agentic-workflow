use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub data_dir: PathBuf,
    pub log_level: String,
    pub transport: TransportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportConfig {
    Stdio,
    Sse { host: String, port: u16 },
}

impl Default for ServerConfig {
    fn default() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agentic-workflow");

        Self {
            data_dir,
            log_level: "info".to_string(),
            transport: TransportConfig::Stdio,
        }
    }
}

/// Resolve the workflow data path for a project.
pub fn resolve_data_path(project_path: Option<&str>) -> PathBuf {
    match project_path {
        Some(path) => {
            let canonical = std::path::Path::new(path);
            let hash = blake3::hash(
                canonical
                    .to_string_lossy()
                    .as_bytes(),
            );
            let dir_name = format!("project-{}", &hash.to_hex()[..16]);

            dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("agentic-workflow")
                .join(dir_name)
        }
        None => dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("agentic-workflow")
            .join("default"),
    }
}
