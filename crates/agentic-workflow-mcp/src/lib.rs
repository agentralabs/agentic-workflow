//! AgenticWorkflow MCP Server
//!
//! Provides 124 MCP tools across 24 inventions for universal workflow orchestration.

pub mod config;
pub mod protocol;
pub mod tools;
pub mod transport;
pub mod types;
pub mod prompts;
pub mod resources;
pub mod session;

pub use config::ServerConfig;
pub use protocol::ProtocolHandler;
pub use transport::StdioTransport;
