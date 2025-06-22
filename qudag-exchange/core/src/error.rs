//! Error types for QuDAG Exchange

use thiserror::Error;

/// Main error type for QuDAG Exchange operations
#[derive(Error, Debug)]
pub enum Error {
    /// Insufficient rUv balance for operation
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance {
        /// Required amount
        required: u128,
        /// Available amount
        available: u128,
    },

    /// Invalid transaction
    #[error("Invalid transaction: {reason}")]
    InvalidTransaction {
        /// Reason for invalidity
        reason: String,
    },

    /// Resource metering error
    #[error("Resource metering error: {0}")]
    ResourceMetering(String),

    /// Wallet error
    #[error("Wallet error: {0}")]
    Wallet(String),

    /// Ledger error
    #[error("Ledger error: {0}")]
    Ledger(String),

    /// Consensus error
    #[error("Consensus error: {0}")]
    Consensus(String),

    /// Cryptographic error
    #[error("Cryptographic error: {0}")]
    Crypto(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type alias for QuDAG Exchange operations
pub type Result<T> = std::result::Result<T, Error>;