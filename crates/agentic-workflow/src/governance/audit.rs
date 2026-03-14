use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    AuditEvent, AuditEventType, AuditImpact, AuditOutcome, AuditQuery,
    AuditRetention, WorkflowResult,
};

/// Structured, queryable audit trail engine.
pub struct AuditEngine {
    events: Vec<AuditEvent>,
    retention: AuditRetention,
}

impl AuditEngine {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            retention: AuditRetention {
                retain_days: 90,
                compliance_preset: None,
                archive_after_days: Some(365),
            },
        }
    }

    /// Record an audit event.
    pub fn record(
        &mut self,
        execution_id: &str,
        workflow_id: &str,
        step_id: Option<&str>,
        event_type: AuditEventType,
        actor: &str,
        resource: Option<&str>,
        input: Option<serde_json::Value>,
        output: Option<serde_json::Value>,
        outcome: AuditOutcome,
    ) -> String {
        let event_id = Uuid::new_v4().to_string();
        let event = AuditEvent {
            event_id: event_id.clone(),
            execution_id: execution_id.to_string(),
            workflow_id: workflow_id.to_string(),
            step_id: step_id.map(|s| s.to_string()),
            event_type,
            actor: actor.to_string(),
            timestamp: Utc::now(),
            resource: resource.map(|s| s.to_string()),
            input,
            output,
            outcome,
            metadata: HashMap::new(),
        };

        self.events.push(event);
        event_id
    }

    /// Query the audit trail.
    pub fn query(&self, q: &AuditQuery) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| {
                if let Some(wid) = &q.workflow_id {
                    if &e.workflow_id != wid {
                        return false;
                    }
                }
                if let Some(eid) = &q.execution_id {
                    if &e.execution_id != eid {
                        return false;
                    }
                }
                if let Some(actor) = &q.actor {
                    if &e.actor != actor {
                        return false;
                    }
                }
                if let Some(resource) = &q.resource {
                    if e.resource.as_deref() != Some(resource) {
                        return false;
                    }
                }
                if let Some(from) = &q.from {
                    if e.timestamp < *from {
                        return false;
                    }
                }
                if let Some(to) = &q.to {
                    if e.timestamp > *to {
                        return false;
                    }
                }
                true
            })
            .take(q.limit.unwrap_or(1000))
            .collect()
    }

    /// Get chronological timeline.
    pub fn timeline(
        &self,
        execution_id: Option<&str>,
        limit: usize,
    ) -> Vec<&AuditEvent> {
        let mut events: Vec<&AuditEvent> = match execution_id {
            Some(eid) => self
                .events
                .iter()
                .filter(|e| e.execution_id == eid)
                .collect(),
            None => self.events.iter().collect(),
        };

        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        events.truncate(limit);
        events
    }

    /// Find all workflows that touched a resource.
    pub fn impact_analysis(&self, resource: &str) -> AuditImpact {
        let matching: Vec<&AuditEvent> = self
            .events
            .iter()
            .filter(|e| e.resource.as_deref() == Some(resource))
            .collect();

        let workflow_ids: Vec<String> = matching
            .iter()
            .map(|e| e.workflow_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let execution_ids: Vec<String> = matching
            .iter()
            .map(|e| e.execution_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let first_touch = matching.iter().map(|e| e.timestamp).min().unwrap_or_else(Utc::now);
        let last_touch = matching.iter().map(|e| e.timestamp).max().unwrap_or_else(Utc::now);

        AuditImpact {
            resource: resource.to_string(),
            workflow_ids,
            execution_ids,
            event_count: matching.len(),
            first_touch,
            last_touch,
        }
    }

    /// Export audit trail as JSON.
    pub fn export(&self, query: &AuditQuery) -> WorkflowResult<String> {
        let events = self.query(query);
        serde_json::to_string_pretty(&events)
            .map_err(|e| crate::types::WorkflowError::SerializationError(e.to_string()))
    }

    /// Set retention policy.
    pub fn set_retention(&mut self, retention: AuditRetention) {
        self.retention = retention;
    }

    /// Get retention policy.
    pub fn get_retention(&self) -> &AuditRetention {
        &self.retention
    }

    /// Total event count.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

impl Default for AuditEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_record_and_query() {
        let mut engine = AuditEngine::new();

        engine.record(
            "exec-1", "wf-1", Some("step-1"),
            AuditEventType::StepExecuted,
            "system", Some("billing-db"),
            None, None, AuditOutcome::Success,
        );

        engine.record(
            "exec-2", "wf-2", None,
            AuditEventType::WorkflowStarted,
            "user-a", None,
            None, None, AuditOutcome::Success,
        );

        let q = AuditQuery {
            workflow_id: Some("wf-1".to_string()),
            execution_id: None,
            event_types: None,
            actor: None,
            resource: None,
            from: None,
            to: None,
            limit: None,
        };

        let results = engine.query(&q);
        assert_eq!(results.len(), 1);

        let impact = engine.impact_analysis("billing-db");
        assert_eq!(impact.event_count, 1);
    }
}
