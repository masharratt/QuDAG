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
pub mod persistence;
pub mod state;
pub mod synchronization;
pub mod types;
pub mod validation;
pub mod versioning;

pub use allocator::{get_memory_usage, get_total_allocated, get_total_deallocated};
pub use compatibility::{CompatibilityAdapter, CompatibilityError, MessageTransformer};
pub use config::Config as ProtocolConfig;
pub use handshake::{
    HandshakeConfig, HandshakeCoordinator, HandshakeError, HandshakeKeys, HandshakeSession,
};
pub use instrumentation::{MemoryMetrics, MemoryTracker};
pub use message::{Message, MessageError, MessageFactory, MessageType, ProtocolVersion};
pub use node::{Node, NodeConfig, NodeStateProvider};
// pub use crate::rpc_server::{RpcServer, RpcCommand};
pub use persistence::{
    MemoryBackend, PersistenceError, PersistenceManager, PersistedDagState, PersistedPeer,
    PersistedState, SqliteBackend, StatePersistence, StateProvider, CURRENT_STATE_VERSION,
};
#[cfg(feature = "rocksdb")]
pub use persistence::RocksDbBackend;
pub use state::{ProtocolState, ProtocolStateMachine, StateError, StateMachineConfig};
pub use types::{ProtocolError, ProtocolEvent};
pub use versioning::{
    VersionError, VersionInfo, VersionManager, VersionPreferences, VersionRegistry,
};

// Re-export coordinator for test compatibility
pub use coordinator::Coordinator;
