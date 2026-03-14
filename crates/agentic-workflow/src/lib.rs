//! AgenticWorkflow — Universal Orchestration Engine
//!
//! Coordinates workflows, pipelines, state machines, batch processing,
//! and every coordination pattern through a single engine.

pub mod types;
pub mod engine;
pub mod resilience;
pub mod governance;
pub mod template;
pub mod intelligence;
pub mod format;

// Re-export core types for convenience
pub use types::*;
