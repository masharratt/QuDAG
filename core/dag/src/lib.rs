#![deny(unsafe_code)]
#![warn(missing_docs)]

//! QuDAG core DAG implementation with QR-Avalanche consensus
//! 
//! This module implements a quantum-resistant DAG-based consensus mechanism
//! with sub-second finality guarantees.

mod node;
mod edge;
mod graph;
mod consensus;
mod error;

pub use error::DagError;
pub use node::Node;
pub use edge::Edge;
pub use graph::Graph;
pub use consensus::QrAvalanche;

/// Result type for DAG operations
pub type Result<T> = std::result::Result<T, DagError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_basic_initialization() {
        let graph = Graph::new();
        assert!(graph.is_empty());
    }
}