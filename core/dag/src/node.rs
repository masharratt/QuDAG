use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use blake3::Hash;

/// Represents the state of a node in the DAG
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// Node has been created but not yet verified
    Pending,
    /// Node has been verified and is part of the DAG
    Verified,
    /// Node has achieved finality through consensus
    Final,
    /// Node has been rejected by consensus
    Rejected,
}

/// A node in the DAG containing a transaction or consensus message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier hash for this node
    hash: Hash,
    /// Payload contained in this node
    payload: Vec<u8>,
    /// Current state of this node
    state: NodeState,
    /// Timestamp when node was created
    timestamp: SystemTime,
    /// Parent node hashes
    parents: Vec<Hash>,
}

impl Node {
    /// Creates a new node with the given payload and parents
    pub fn new(payload: Vec<u8>, parents: Vec<Hash>) -> Self {
        let timestamp = SystemTime::now();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&payload);
        for parent in &parents {
            hasher.update(parent.as_bytes());
        }
        let hash = hasher.finalize();

        Self {
            hash,
            payload,
            state: NodeState::Pending,
            timestamp,
            parents,
        }
    }

    /// Returns the node's unique hash
    pub fn hash(&self) -> &Hash {
        &self.hash
    }

    /// Returns reference to node's payload
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns current state of the node
    pub fn state(&self) -> NodeState {
        self.state  
    }

    /// Returns reference to parent hashes
    pub fn parents(&self) -> &[Hash] {
        &self.parents
    }

    /// Updates node state if transition is valid
    pub fn update_state(&mut self, new_state: NodeState) -> crate::Result<()> {
        match (self.state, new_state) {
            // Valid transitions
            (NodeState::Pending, NodeState::Verified) |
            (NodeState::Verified, NodeState::Final) |
            (NodeState::Pending, NodeState::Rejected) |
            (NodeState::Verified, NodeState::Rejected) => {
                self.state = new_state;
                Ok(())
            }
            // Invalid transitions
            _ => Err(crate::DagError::InvalidStateTransition(
                format!("{:?} -> {:?}", self.state, new_state)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let payload = vec![1, 2, 3];
        let parents = vec![blake3::hash(b"parent1"), blake3::hash(b"parent2")];
        let node = Node::new(payload.clone(), parents.clone());

        assert_eq!(node.state(), NodeState::Pending);
        assert_eq!(node.payload(), &payload);
        assert_eq!(node.parents(), &parents);
    }

    #[test]
    fn test_valid_state_transitions() {
        let mut node = Node::new(vec![1, 2, 3], vec![]);
        
        // Pending -> Verified
        assert!(node.update_state(NodeState::Verified).is_ok());
        assert_eq!(node.state(), NodeState::Verified);

        // Verified -> Final
        assert!(node.update_state(NodeState::Final).is_ok());
        assert_eq!(node.state(), NodeState::Final);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut node = Node::new(vec![1, 2, 3], vec![]);

        // Can't go from Pending to Final
        assert!(node.update_state(NodeState::Final).is_err());
        
        // Update to Verified
        assert!(node.update_state(NodeState::Verified).is_ok());
        
        // Can't go back to Pending
        assert!(node.update_state(NodeState::Pending).is_err());
    }
}