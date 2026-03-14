use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Circuit breaker state for a service used by a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub service_id: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub half_open_at: Option<DateTime<Utc>>,
    pub cooldown_ms: u64,
}

/// Circuit breaker states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Preflight check result for all services in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightResult {
    pub workflow_id: String,
    pub all_services_healthy: bool,
    pub service_states: Vec<ServiceHealth>,
    pub checked_at: DateTime<Utc>,
}

/// Health status of a single service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_id: String,
    pub healthy: bool,
    pub circuit_state: CircuitState,
    pub last_check: DateTime<Utc>,
    pub message: Option<String>,
}

/// A workflow queued waiting for service recovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedWorkflow {
    pub workflow_id: String,
    pub execution_id: String,
    pub waiting_for_service: String,
    pub queued_at: DateTime<Utc>,
    pub priority: u32,
}
