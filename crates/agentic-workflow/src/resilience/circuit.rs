use std::collections::HashMap;

use chrono::Utc;

use crate::types::{
    CircuitBreaker, CircuitState, PreflightResult, QueuedWorkflow,
    ServiceHealth, WorkflowError, WorkflowResult,
};

/// Workflow-aware circuit breaker engine.
pub struct CircuitBreakerEngine {
    breakers: HashMap<String, CircuitBreaker>,
    queued: Vec<QueuedWorkflow>,
}

impl CircuitBreakerEngine {
    pub fn new() -> Self {
        Self {
            breakers: HashMap::new(),
            queued: Vec::new(),
        }
    }

    /// Get or create a circuit breaker for a service.
    pub fn get_or_create(
        &mut self,
        service_id: &str,
        failure_threshold: u32,
        success_threshold: u32,
        cooldown_ms: u64,
    ) -> &CircuitBreaker {
        self.breakers
            .entry(service_id.to_string())
            .or_insert_with(|| CircuitBreaker {
                service_id: service_id.to_string(),
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                failure_threshold,
                success_threshold,
                last_failure_at: None,
                last_success_at: None,
                half_open_at: None,
                cooldown_ms,
            })
    }

    /// Record a failure for a service.
    pub fn record_failure(&mut self, service_id: &str) -> WorkflowResult<()> {
        let breaker = self
            .breakers
            .get_mut(service_id)
            .ok_or_else(|| WorkflowError::CircuitBreakerOpen(service_id.to_string()))?;

        breaker.failure_count += 1;
        breaker.last_failure_at = Some(Utc::now());

        if breaker.failure_count >= breaker.failure_threshold {
            breaker.state = CircuitState::Open;
            eprintln!(
                "Circuit breaker OPEN for service {}: {} failures",
                service_id, breaker.failure_count
            );
        }

        Ok(())
    }

    /// Record a success for a service.
    pub fn record_success(&mut self, service_id: &str) -> WorkflowResult<()> {
        let breaker = self
            .breakers
            .get_mut(service_id)
            .ok_or_else(|| WorkflowError::Internal(format!("Breaker not found: {}", service_id)))?;

        breaker.success_count += 1;
        breaker.last_success_at = Some(Utc::now());

        if breaker.state == CircuitState::HalfOpen
            && breaker.success_count >= breaker.success_threshold
        {
            breaker.state = CircuitState::Closed;
            breaker.failure_count = 0;
            breaker.success_count = 0;
            eprintln!("Circuit breaker CLOSED for service {}", service_id);
        }

        Ok(())
    }

    /// Check if a service is available.
    pub fn is_available(&self, service_id: &str) -> bool {
        match self.breakers.get(service_id) {
            None => true,
            Some(b) => b.state != CircuitState::Open,
        }
    }

    /// Force reset a circuit breaker.
    pub fn reset(&mut self, service_id: &str) -> WorkflowResult<()> {
        let breaker = self
            .breakers
            .get_mut(service_id)
            .ok_or_else(|| WorkflowError::Internal(format!("Breaker not found: {}", service_id)))?;

        breaker.state = CircuitState::Closed;
        breaker.failure_count = 0;
        breaker.success_count = 0;
        Ok(())
    }

    /// Preflight check — verify all services needed by a workflow.
    pub fn preflight_check(
        &self,
        workflow_id: &str,
        service_ids: &[String],
    ) -> PreflightResult {
        let now = Utc::now();
        let mut service_states = Vec::new();
        let mut all_healthy = true;

        for sid in service_ids {
            let (healthy, circuit_state) = match self.breakers.get(sid) {
                None => (true, CircuitState::Closed),
                Some(b) => (b.state != CircuitState::Open, b.state.clone()),
            };

            if !healthy {
                all_healthy = false;
            }

            service_states.push(ServiceHealth {
                service_id: sid.clone(),
                healthy,
                circuit_state,
                last_check: now,
                message: None,
            });
        }

        PreflightResult {
            workflow_id: workflow_id.to_string(),
            all_services_healthy: all_healthy,
            service_states,
            checked_at: now,
        }
    }

    /// Get all circuit breaker statuses.
    pub fn all_statuses(&self) -> Vec<&CircuitBreaker> {
        self.breakers.values().collect()
    }

    /// Get queued workflows.
    pub fn queued_workflows(&self) -> &[QueuedWorkflow] {
        &self.queued
    }

    /// Queue a workflow waiting for service recovery.
    pub fn queue_workflow(
        &mut self,
        workflow_id: &str,
        execution_id: &str,
        service_id: &str,
        priority: u32,
    ) {
        self.queued.push(QueuedWorkflow {
            workflow_id: workflow_id.to_string(),
            execution_id: execution_id.to_string(),
            waiting_for_service: service_id.to_string(),
            queued_at: Utc::now(),
            priority,
        });
    }
}

impl Default for CircuitBreakerEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_opens() {
        let mut engine = CircuitBreakerEngine::new();
        engine.get_or_create("api-service", 3, 2, 5000);

        assert!(engine.is_available("api-service"));
        engine.record_failure("api-service").unwrap();
        engine.record_failure("api-service").unwrap();
        assert!(engine.is_available("api-service"));
        engine.record_failure("api-service").unwrap();
        assert!(!engine.is_available("api-service")); // OPEN after 3 failures
    }
}
