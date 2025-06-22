//! QuDAG Exchange Core Library
//! 
//! This crate provides the core functionality for the QuDAG Exchange system:
//! - rUv (Resource Utilization Voucher) token ledger
//! - Resource metering and cost calculations
//! - Transaction processing with quantum-resistant signatures
//! - Consensus integration with QR-Avalanche DAG
//! - Secure key management through QuDAG Vault
//!
//! The library is designed to be no_std compatible for WASM deployment.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, collections::BTreeMap};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec, collections::BTreeMap};

// Public modules
pub mod error;
pub mod ledger;
pub mod account;
pub mod transaction;
pub mod metering;
pub mod consensus;
pub mod state;
pub mod types;

// Re-exports
pub use error::{Error, Result};
pub use ledger::Ledger;
pub use account::{Account, AccountId, Balance};
pub use transaction::{Transaction, TransactionId, TransactionStatus};
pub use metering::{ResourceMeter, OperationCost};
pub use consensus::ConsensusAdapter;
pub use state::LedgerState;
pub use types::rUv;

/// Core version string
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the version of the QuDAG Exchange Core library
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
        assert!(version().contains('.'));
    }
}