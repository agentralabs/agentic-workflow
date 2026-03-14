use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    ApprovalDecision, ApprovalGate, ApprovalReceipt, PendingApproval,
    WorkflowError, WorkflowResult,
};

/// Approval gate engine — rich approval workflow with escalation.
pub struct ApprovalEngine {
    gates: HashMap<String, ApprovalGate>,
    pending: HashMap<String, PendingApproval>,
    receipts: Vec<ApprovalReceipt>,
}

impl ApprovalEngine {
    pub fn new() -> Self {
        Self {
            gates: HashMap::new(),
            pending: HashMap::new(),
            receipts: Vec::new(),
        }
    }

    /// Define an approval gate.
    pub fn define_gate(&mut self, gate: ApprovalGate) -> WorkflowResult<()> {
        self.gates.insert(gate.id.clone(), gate);
        Ok(())
    }

    /// Request approval — creates a pending approval.
    pub fn request_approval(
        &mut self,
        gate_id: &str,
        execution_id: &str,
        step_id: &str,
        context: serde_json::Value,
    ) -> WorkflowResult<String> {
        let gate = self
            .gates
            .get(gate_id)
            .ok_or_else(|| WorkflowError::ApprovalRequired(gate_id.to_string()))?;

        let now = Utc::now();
        let current_approver = gate
            .approver_chain
            .first()
            .map(|a| a.identity.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let expires_at = gate
            .timeout
            .as_ref()
            .map(|t| now + chrono::Duration::milliseconds(t.timeout_ms as i64));

        let pending_id = Uuid::new_v4().to_string();
        let pending = PendingApproval {
            gate_id: gate_id.to_string(),
            execution_id: execution_id.to_string(),
            step_id: step_id.to_string(),
            requested_at: now,
            expires_at,
            current_approver,
            context,
        };

        self.pending.insert(pending_id.clone(), pending);
        Ok(pending_id)
    }

    /// Decide on a pending approval.
    pub fn decide(
        &mut self,
        pending_id: &str,
        decision: ApprovalDecision,
        decided_by: &str,
        reason: Option<String>,
    ) -> WorkflowResult<ApprovalReceipt> {
        let pending = self
            .pending
            .remove(pending_id)
            .ok_or_else(|| WorkflowError::Internal(format!("No pending approval: {}", pending_id)))?;

        let receipt = ApprovalReceipt {
            gate_id: pending.gate_id,
            execution_id: pending.execution_id,
            decision,
            decided_by: decided_by.to_string(),
            decided_at: Utc::now(),
            reason,
            checksum: blake3::hash(pending_id.as_bytes()).to_hex().to_string(),
        };

        self.receipts.push(receipt.clone());
        Ok(receipt)
    }

    /// List pending approvals.
    pub fn list_pending(&self) -> Vec<(&str, &PendingApproval)> {
        self.pending.iter().map(|(k, v)| (k.as_str(), v)).collect()
    }

    /// Get approval audit trail.
    pub fn get_receipts(&self, gate_id: Option<&str>) -> Vec<&ApprovalReceipt> {
        match gate_id {
            Some(gid) => self.receipts.iter().filter(|r| r.gate_id == gid).collect(),
            None => self.receipts.iter().collect(),
        }
    }

    /// Escalate a pending approval to the next approver in the chain.
    pub fn escalate(&mut self, pending_id: &str) -> WorkflowResult<()> {
        let pending = self
            .pending
            .get_mut(pending_id)
            .ok_or_else(|| WorkflowError::Internal(format!("No pending approval: {}", pending_id)))?;

        let gate = self
            .gates
            .get(&pending.gate_id)
            .ok_or_else(|| WorkflowError::Internal("Gate not found".to_string()))?;

        // Find next approver
        let current_idx = gate
            .approver_chain
            .iter()
            .position(|a| a.identity == pending.current_approver);

        if let Some(idx) = current_idx {
            if idx + 1 < gate.approver_chain.len() {
                pending.current_approver = gate.approver_chain[idx + 1].identity.clone();
                return Ok(());
            }
        }

        Err(WorkflowError::Internal(
            "No more approvers in chain".to_string(),
        ))
    }

    /// Get a gate definition.
    pub fn get_gate(&self, gate_id: &str) -> Option<&ApprovalGate> {
        self.gates.get(gate_id)
    }
}

impl Default for ApprovalEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::approval::Approver;

    #[test]
    fn test_approval_flow() {
        let mut engine = ApprovalEngine::new();

        let gate = ApprovalGate {
            id: "deploy-gate".into(),
            step_id: "deploy-prod".into(),
            workflow_id: "ci-cd".into(),
            approver_chain: vec![
                Approver { identity: "alice".into(), role: Some("lead".into()), priority: 1 },
                Approver { identity: "bob".into(), role: Some("manager".into()), priority: 2 },
            ],
            condition: None,
            timeout: None,
            delegation: None,
        };

        engine.define_gate(gate).unwrap();

        let pid = engine
            .request_approval("deploy-gate", "exec-1", "deploy-prod", serde_json::json!({}))
            .unwrap();

        assert_eq!(engine.list_pending().len(), 1);

        let receipt = engine
            .decide(&pid, ApprovalDecision::Approved, "alice", Some("LGTM".into()))
            .unwrap();

        assert!(matches!(receipt.decision, ApprovalDecision::Approved));
        assert_eq!(engine.list_pending().len(), 0);
    }
}
