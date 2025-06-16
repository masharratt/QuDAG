#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Command-line interface for the QuDAG protocol.
//! 
//! This module provides a comprehensive CLI for managing QuDAG nodes,
//! including node operations, peer management, network diagnostics,
//! and DAG visualization capabilities.

pub mod commands;
pub mod config;
pub mod output;

pub use commands::*;

/// CLI-specific error types
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Node error: {0}")]
    Node(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Visualization error: {0}")]
    Visualization(String),
    #[error("Configuration error: {0}")]
    Config(String),
}