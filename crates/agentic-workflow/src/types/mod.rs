pub mod approval;
pub mod audit;
pub mod batch;
pub mod circuit;
pub mod dead_letter;
pub mod error;
pub mod execution;
pub mod fanout;
pub mod fsm;
pub mod idempotency;
pub mod prediction;
pub mod retry;
pub mod rollback;
pub mod schedule;
pub mod stream;
pub mod template;
pub mod trigger;
pub mod variable;
pub mod workflow;

// Re-export core types at the module level
pub use error::{WorkflowError, WorkflowResult};
pub use workflow::{CompletionPolicy, Edge, EdgeType, StepNode, StepType, Workflow};
pub use execution::{
    ExecutionContext, ExecutionEvent, ExecutionEventType, ExecutionFingerprint,
    ExecutionProgress, ExecutionStatus, StepLifecycle, StepState, TriggerInfo,
};
pub use trigger::{FileEvent, Trigger, TriggerActivation, TriggerCondition, TriggerType};
pub use schedule::{AdaptiveSchedule, ConflictPolicy, Schedule, ScheduleExpression};
pub use approval::{
    ApprovalDecision, ApprovalGate, ApprovalReceipt, Approver, PendingApproval,
};
pub use audit::{AuditEvent, AuditEventType, AuditImpact, AuditOutcome, AuditQuery, AuditRetention};
pub use variable::{ScopeType, ScopedVariable, TypeCheckError, TypeCheckResult, VariableScope, VariableType};
pub use retry::{
    FailureClass, RetryBudget, RetryEscalation, RetryPattern, RetryPolicy,
    RetryProfile, RetryStats, RetryStrategy,
};
pub use batch::{BatchErrorGroup, BatchItem, BatchItemStatus, BatchJob, BatchProgress, BatchReport, BatchStatus};
pub use stream::{
    BackpressureConfig, BackpressureStrategy, ProcessingWindow, StreamCheckpoint,
    StreamFork, StreamProcessor, StreamSource, StreamStatus,
};
pub use fanout::{FanOutBranch, FanOutBranchStatus, FanOutDestination, FanOutStatus, FanOutStep, ResultAggregator};
pub use fsm::{State, StateMachine, Transition, TransitionRecord};
pub use template::{
    Clarification, CompositionOperator, DataBridge, MetaWorkflow, NaturalLanguageRequest,
    SharedWorkflow, TemplateParameter, WorkflowTemplate,
};
pub use idempotency::{
    ConflictResolution, IdempotencyConfig, IdempotencyEntry, IdempotencyReport,
    IdempotencyWindow, KeyStrategy, StepIdempotencyStats,
};
pub use rollback::{RollbackAction, RollbackReceipt, RollbackScope, RollbackStepResult, RollbackType};
pub use circuit::{CircuitBreaker, CircuitState, PreflightResult, QueuedWorkflow, ServiceHealth};
pub use dead_letter::{DeadLetterItem, DeadLetterPolicy, DeadLetterSummary, FailureGroup};
pub use prediction::{
    CostPrediction, DurationPrediction, HealthIssue, IssueSeverity, ResourcePrediction,
    RiskFactor, StepDurationPrediction, SuccessPrediction, WorkflowHealth,
};
