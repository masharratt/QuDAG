#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Main protocol implementation and coordination for QuDAG.

pub mod allocator;
pub mod compatibility;
pub mod config;
pub mod coordinator;
pub mod handshake;
pub mod instrumentation;
pub mod message;
pub mod metrics;
pub mod node;
pub mod rpc_server;
pub mod state;
pub mod synchronization;
pub mod types;
pub mod validation;
pub mod versioning;

pub use allocator::{get_memory_usage, get_total_allocated, get_total_deallocated};
pub use compatibility::{CompatibilityAdapter, CompatibilityError, MessageTransformer};
pub use config::Config as ProtocolConfig;
pub use handshake::{HandshakeCoordinator, HandshakeConfig, HandshakeSession, HandshakeError, HandshakeKeys};
pub use instrumentation::{MemoryTracker, MemoryMetrics};
pub use message::{Message, MessageType, MessageError, ProtocolVersion, MessageFactory};
pub use node::{Node, NodeConfig};
// pub use crate::rpc_server::{RpcServer, RpcCommand};
pub use state::{StateError, ProtocolStateMachine, ProtocolState, StateMachineConfig};
pub use types::{ProtocolError, ProtocolEvent};
pub use versioning::{VersionManager, VersionRegistry, VersionError, VersionInfo, VersionPreferences};

// Re-export coordinator for test compatibility
pub use coordinator::Coordinator;