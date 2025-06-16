//! Protocol state implementation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during state operations.
#[derive(Debug, Error)]
pub enum StateError {
    /// Invalid state transition
    #[error("Invalid state transition")]
    InvalidTransition,
    
    /// State synchronization failed
    #[error("State synchronization failed")]
    SyncFailed,
    
    /// Invalid state data
    #[error("Invalid state data")]
    InvalidData,
}

/// Protocol state type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    /// Initial state
    Initial,
    
    /// Handshake completed
    Ready,
    
    /// Active communication
    Active,
    
    /// Error state
    Error,
}

/// Protocol state information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// State type
    pub state_type: StateType,
    
    /// State data
    pub data: Vec<u8>,
    
    /// State version
    pub version: u32,
    
    /// State timestamp
    pub timestamp: u64,
}

/// State management trait defining the interface for state operations.
pub trait StateManager {
    /// Initialize protocol state.
    fn init() -> Result<State, StateError>;
    
    /// Transition to a new state.
    fn transition(&mut self, new_state: StateType) -> Result<(), StateError>;
    
    /// Update state data.
    fn update_data(&mut self, data: Vec<u8>) -> Result<(), StateError>;
    
    /// Get current state.
    fn get_state(&self) -> State;
    
    /// Validate state transition.
    fn validate_transition(&self, new_state: StateType) -> bool;
}