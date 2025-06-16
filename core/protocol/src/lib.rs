#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Main protocol implementation and coordination for QuDAG.

pub mod allocator;
pub mod config;
pub mod instrumentation;
pub mod message;
pub mod metrics;
pub mod node;
pub mod state;
pub mod synchronization;
pub mod types;
pub mod validation;

pub use allocator::{get_memory_usage, get_total_allocated, get_total_deallocated};
pub use config::Config as ProtocolConfig;
pub use instrumentation::{MemoryTracker, MemoryMetrics};
pub use message::{Message, MessageType, MessageError};
pub use node::{Node, NodeConfig};
pub use state::{StateError};
pub use types::{ProtocolError, ProtocolEvent, ProtocolState};