#![deny(unsafe_code)]
#![warn(missing_docs)]

//! DAG consensus implementation with QR-Avalanche algorithm.
//! 
//! This module provides the core DAG (Directed Acyclic Graph) implementation
//! with quantum-resistant consensus using a modified Avalanche protocol.

pub mod consensus;
pub mod vertex;
pub mod tip_selection;
pub mod dag;

pub use consensus::{Consensus, ConsensusError, ConsensusStatus};
pub use vertex::{Vertex, VertexId, VertexError};
pub use tip_selection::{TipSelection, TipSelectionError};
pub use dag::{Dag, DagMessage, DagError};