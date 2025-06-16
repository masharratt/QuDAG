//! DAG vertex implementation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during vertex operations.
#[derive(Debug, Error)]
pub enum VertexError {
    /// Invalid parent reference
    #[error("Invalid parent reference")]
    InvalidParent,
    
    /// Invalid payload format
    #[error("Invalid payload format")]
    InvalidPayload,
    
    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,
    
    /// Vertex creation failed
    #[error("Vertex creation failed")]
    CreationFailed,
}

/// Unique vertex identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(Vec<u8>);

/// DAG vertex containing a message payload and references to parent vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    /// Unique vertex identifier
    pub id: VertexId,
    
    /// References to parent vertices
    pub parents: Vec<VertexId>,
    
    /// Message payload
    pub payload: Vec<u8>,
    
    /// Vertex timestamp
    pub timestamp: u64,
    
    /// Cryptographic signature
    pub signature: Vec<u8>,
}

/// Vertex trait defining the interface for creating and validating vertices.
pub trait VertexOps {
    /// Create a new vertex with the given payload and parent references.
    fn create(payload: Vec<u8>, parents: Vec<VertexId>) -> Result<Vertex, VertexError>;
    
    /// Validate a vertex's structure and signature.
    fn validate(&self) -> Result<bool, VertexError>;
    
    /// Get the vertex's score based on the DAG topology.
    fn score(&self) -> f64;
    
    /// Check if vertex is a tip (has no children).
    fn is_tip(&self) -> bool;
}