use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use crate::types::{
    State, StateMachine, Transition, TransitionRecord,
    WorkflowError, WorkflowResult,
};

/// Finite state machine engine.
pub struct FsmEngine {
    machines: HashMap<String, StateMachine>,
    history: HashMap<String, Vec<TransitionRecord>>,
}

impl FsmEngine {
    pub fn new() -> Self {
        Self {
            machines: HashMap::new(),
            history: HashMap::new(),
        }
    }

    /// Create a new state machine.
    pub fn create_fsm(
        &mut self,
        name: &str,
        states: Vec<State>,
        transitions: Vec<Transition>,
        initial_state: &str,
    ) -> WorkflowResult<String> {
        // Validate initial state exists
        if !states.iter().any(|s| s.name == initial_state) {
            return Err(WorkflowError::InvalidTransition {
                from: "".to_string(),
                to: initial_state.to_string(),
            });
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let fsm = StateMachine {
            id: id.clone(),
            name: name.to_string(),
            states,
            transitions,
            initial_state: initial_state.to_string(),
            current_state: initial_state.to_string(),
            context: HashMap::new(),
            created_at: now,
            updated_at: now,
        };

        self.machines.insert(id.clone(), fsm);
        self.history.insert(id.clone(), Vec::new());
        Ok(id)
    }

    /// Attempt a state transition.
    pub fn transition(
        &mut self,
        fsm_id: &str,
        event: &str,
    ) -> WorkflowResult<String> {
        let fsm = self
            .machines
            .get_mut(fsm_id)
            .ok_or_else(|| WorkflowError::Internal(format!("FSM not found: {}", fsm_id)))?;

        let current = fsm.current_state.clone();

        // Find matching transition
        let transition = fsm
            .transitions
            .iter()
            .find(|t| t.from == current && t.event == event)
            .ok_or_else(|| WorkflowError::InvalidTransition {
                from: current.clone(),
                to: format!("(event: {})", event),
            })?
            .clone();

        let to_state = transition.to.clone();

        // Record the transition
        let record = TransitionRecord {
            fsm_id: fsm_id.to_string(),
            from_state: current.clone(),
            to_state: to_state.clone(),
            event: event.to_string(),
            timestamp: Utc::now(),
            context_snapshot: fsm.context.clone(),
        };

        // Apply the transition
        fsm.current_state = to_state.clone();
        fsm.updated_at = Utc::now();

        self.history
            .entry(fsm_id.to_string())
            .or_default()
            .push(record);

        Ok(to_state)
    }

    /// Get current state.
    pub fn current_state(&self, fsm_id: &str) -> WorkflowResult<&str> {
        let fsm = self
            .machines
            .get(fsm_id)
            .ok_or_else(|| WorkflowError::Internal(format!("FSM not found: {}", fsm_id)))?;

        Ok(&fsm.current_state)
    }

    /// Get valid next transitions.
    pub fn valid_next(&self, fsm_id: &str) -> WorkflowResult<Vec<&Transition>> {
        let fsm = self
            .machines
            .get(fsm_id)
            .ok_or_else(|| WorkflowError::Internal(format!("FSM not found: {}", fsm_id)))?;

        Ok(fsm.valid_transitions())
    }

    /// Get transition history.
    pub fn get_history(&self, fsm_id: &str) -> WorkflowResult<&[TransitionRecord]> {
        self.history
            .get(fsm_id)
            .map(|v| v.as_slice())
            .ok_or_else(|| WorkflowError::Internal(format!("FSM not found: {}", fsm_id)))
    }

    /// Get a state machine.
    pub fn get_fsm(&self, fsm_id: &str) -> WorkflowResult<&StateMachine> {
        self.machines
            .get(fsm_id)
            .ok_or_else(|| WorkflowError::Internal(format!("FSM not found: {}", fsm_id)))
    }

    /// Generate a Mermaid state diagram.
    pub fn diagram(&self, fsm_id: &str) -> WorkflowResult<String> {
        let fsm = self.get_fsm(fsm_id)?;
        let mut lines = vec!["stateDiagram-v2".to_string()];

        lines.push(format!("    [*] --> {}", fsm.initial_state));

        for t in &fsm.transitions {
            lines.push(format!("    {} --> {} : {}", t.from, t.to, t.event));
        }

        for state in &fsm.states {
            if state.is_terminal {
                lines.push(format!("    {} --> [*]", state.name));
            }
        }

        Ok(lines.join("\n"))
    }
}

impl Default for FsmEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order_states() -> (Vec<State>, Vec<Transition>) {
        let states = vec![
            State { name: "Created".into(), description: None, entry_action: None, exit_action: None, is_terminal: false },
            State { name: "Paid".into(), description: None, entry_action: None, exit_action: None, is_terminal: false },
            State { name: "Shipped".into(), description: None, entry_action: None, exit_action: None, is_terminal: false },
            State { name: "Delivered".into(), description: None, entry_action: None, exit_action: None, is_terminal: true },
        ];

        let transitions = vec![
            Transition { from: "Created".into(), to: "Paid".into(), event: "pay".into(), guard: None, action: None },
            Transition { from: "Paid".into(), to: "Shipped".into(), event: "ship".into(), guard: None, action: None },
            Transition { from: "Shipped".into(), to: "Delivered".into(), event: "deliver".into(), guard: None, action: None },
        ];

        (states, transitions)
    }

    #[test]
    fn test_fsm_transitions() {
        let mut engine = FsmEngine::new();
        let (states, transitions) = order_states();
        let fid = engine.create_fsm("order", states, transitions, "Created").unwrap();

        assert_eq!(engine.current_state(&fid).unwrap(), "Created");
        engine.transition(&fid, "pay").unwrap();
        assert_eq!(engine.current_state(&fid).unwrap(), "Paid");
        engine.transition(&fid, "ship").unwrap();
        assert_eq!(engine.current_state(&fid).unwrap(), "Shipped");

        // Invalid transition
        assert!(engine.transition(&fid, "pay").is_err());
    }
}
