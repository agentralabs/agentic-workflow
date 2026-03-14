use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Finite state machine definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    pub id: String,
    pub name: String,
    pub states: Vec<State>,
    pub transitions: Vec<Transition>,
    pub initial_state: String,
    pub current_state: String,
    pub context: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StateMachine {
    /// Get valid next transitions from the current state.
    pub fn valid_transitions(&self) -> Vec<&Transition> {
        self.transitions
            .iter()
            .filter(|t| t.from == self.current_state)
            .collect()
    }

    /// Check if a transition is valid.
    pub fn can_transition(&self, to: &str) -> bool {
        self.valid_transitions().iter().any(|t| t.to == to)
    }
}

/// A state in the FSM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub name: String,
    pub description: Option<String>,
    pub entry_action: Option<StateAction>,
    pub exit_action: Option<StateAction>,
    pub is_terminal: bool,
}

/// Action to perform on state entry or exit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateAction {
    pub action_type: String,
    pub config: serde_json::Value,
}

/// A transition between states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub event: String,
    pub guard: Option<TransitionGuard>,
    pub action: Option<StateAction>,
}

/// Guard condition that must be true for transition to proceed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionGuard {
    pub expression: String,
    pub description: Option<String>,
}

/// Record of a state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    pub fsm_id: String,
    pub from_state: String,
    pub to_state: String,
    pub event: String,
    pub timestamp: DateTime<Utc>,
    pub context_snapshot: HashMap<String, serde_json::Value>,
}
