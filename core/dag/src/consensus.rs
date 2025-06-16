//! DAG consensus implementation with QR-Avalanche algorithm.

use thiserror::Error;
use crate::vertex::{Vertex, VertexId};

/// Errors that can occur during consensus operations.
#[derive(Debug, Error)]
pub enum ConsensusError {
    /// Invalid vertex reference
    #[error("Invalid vertex reference")]
    InvalidVertex,
    
    /// Conflicting vertices
    #[error("Conflicting vertices")]
    ConflictingVertices,
    
    /// Failed to reach consensus
    #[error("Failed to reach consensus")]
    ConsensusFailure,
    
    /// Invalid system state
    #[error("Invalid system state")]
    InvalidState,
}

/// Consensus status for a vertex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusStatus {
    /// Vertex is pending consensus
    Pending,
    
    /// Vertex has achieved consensus
    Accepted,
    
    /// Vertex has been rejected
    Rejected,
}

/// DAG consensus trait defining the interface for consensus operations.
pub trait Consensus {
    /// Initialize consensus system with genesis vertex.
    fn init(genesis: Vertex) -> Result<(), ConsensusError>;
    
    /// Process a new vertex for consensus.
    fn process_vertex(&mut self, vertex: &Vertex) -> Result<ConsensusStatus, ConsensusError>;
    
    /// Check if consensus has been reached for a vertex.
    fn is_consensus_reached(&self, vertex_id: &VertexId) -> Result<bool, ConsensusError>;
    
    /// Get the current tip set (vertices with no children).
    fn get_tips(&self) -> Vec<VertexId>;
    
    /// Prune old vertices that have achieved consensus.
    fn prune(&mut self) -> Result<(), ConsensusError>;
}

/// QR-Avalanche consensus implementation
#[derive(Debug)]
pub struct QRAvalanche {
    /// Vertices and their consensus status
    vertices: std::collections::HashMap<VertexId, ConsensusStatus>,
    /// Tip set (vertices with no children)
    tips: std::collections::HashSet<VertexId>,
}

impl QRAvalanche {
    /// Creates a new QR-Avalanche consensus instance
    pub fn new() -> Self {
        Self {
            vertices: std::collections::HashMap::new(),
            tips: std::collections::HashSet::new(),
        }
    }
    
    /// Process a vertex ID for consensus
    pub fn process_vertex(&mut self, vertex_id: VertexId) -> Result<ConsensusStatus, ConsensusError> {
        // Simple implementation - mark as accepted
        let status = ConsensusStatus::Accepted;
        self.vertices.insert(vertex_id.clone(), status.clone());
        self.tips.insert(vertex_id);
        Ok(status)
    }
    
    /// Synchronize with another consensus instance
    pub fn sync(&mut self) -> Result<(), ConsensusError> {
        // Simple sync implementation - nothing to do for now
        Ok(())
    }
}