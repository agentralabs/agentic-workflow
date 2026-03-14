use thiserror::Error;

/// Core error types for AgenticWorkflow.
#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Step not found: {0}")]
    StepNotFound(String),

    #[error("DAG cycle detected: {0}")]
    CycleDetected(String),

    #[error("Unsatisfied dependency: step {step} depends on {dependency}")]
    UnsatisfiedDependency { step: String, dependency: String },

    #[error("Invalid transition: {from} → {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Execution not found: {0}")]
    ExecutionNotFound(String),

    #[error("Execution already running: {0}")]
    ExecutionAlreadyRunning(String),

    #[error("Execution not paused: {0}")]
    ExecutionNotPaused(String),

    #[error("Schedule error: {0}")]
    ScheduleError(String),

    #[error("Trigger error: {0}")]
    TriggerError(String),

    #[error("Approval required: gate {0}")]
    ApprovalRequired(String),

    #[error("Approval denied: gate {0}")]
    ApprovalDenied(String),

    #[error("Retry budget exhausted: {0}")]
    RetryBudgetExhausted(String),

    #[error("Circuit breaker open: service {0}")]
    CircuitBreakerOpen(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Variable type mismatch: expected {expected}, got {actual}")]
    VariableTypeMismatch { expected: String, actual: String },

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Batch error: {0}")]
    BatchError(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Idempotency conflict: key {0} already processed")]
    IdempotencyConflict(String),

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
