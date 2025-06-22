//! QuDAG Exchange Core - Resource Utilization Voucher (rUv) System
//!
//! This module provides the core functionality for the QuDAG Exchange protocol,
//! including the rUv ledger, resource metering, and consensus integration.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod ledger;
pub mod resource;
pub mod ruv;
pub mod transaction;
pub mod wallet;

pub use error::{Error, Result};
pub use ledger::Ledger;
pub use resource::{ResourceContribution, ResourceMetrics, ResourceType};
pub use ruv::{Ruv, RuvAmount};
pub use transaction::{Transaction, TransactionType};
pub use wallet::Wallet;

/// Version of the QuDAG Exchange protocol
pub const PROTOCOL_VERSION: &str = "0.1.0";

/// Minimum rUv amount for transactions
pub const MIN_RUV_AMOUNT: u64 = 1;

/// Maximum rUv supply (21 billion units with 8 decimal places)
pub const MAX_RUV_SUPPLY: u128 = 21_000_000_000_00000000;