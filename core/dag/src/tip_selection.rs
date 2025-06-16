//! DAG tip selection implementation.

use thiserror::Error;
use crate::vertex::{Vertex, VertexId};

/// Errors that can occur during tip selection.
#[derive(Debug, Error)]
pub enum TipSelectionError {
    /// No valid tips available
    #[error("No valid tips available")]
    NoValidTips,
    
    /// Invalid tip reference
    #[error("Invalid tip reference")]
    InvalidTip,
    
    /// Selection failure
    #[error("Selection failure")]
    SelectionFailed,
}

/// Tip selection algorithm configuration.
#[derive(Debug, Clone)]
pub struct TipSelectionConfig {
    /// Number of tips to select
    pub tip_count: usize,
    
    /// Maximum tip age (in seconds)
    pub max_age: u64,
    
    /// Minimum confidence score
    pub min_confidence: f64,
}

/// DAG tip selection trait defining the interface for tip selection algorithms.
pub trait TipSelection {
    /// Initialize tip selection with configuration.
    fn init(config: TipSelectionConfig) -> Result<(), TipSelectionError>;
    
    /// Select tips for a new vertex.
    fn select_tips(&self) -> Result<Vec<VertexId>, TipSelectionError>;
    
    /// Check if a vertex is eligible as a tip.
    fn is_valid_tip(&self, vertex: &Vertex) -> bool;
    
    /// Calculate confidence score for a tip.
    fn calculate_confidence(&self, tip: &VertexId) -> f64;
    
    /// Update tip pool with new vertex.
    fn update_tips(&mut self, vertex: &Vertex) -> Result<(), TipSelectionError>;
}