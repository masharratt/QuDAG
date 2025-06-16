#![deny(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A message in the network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Unique message ID
    pub id: String,
    /// Message source (anonymized)
    pub source: Vec<u8>,
    /// Message destination(s)
    pub destination: Vec<u8>,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message priority
    pub priority: MessagePriority,
    /// Time-to-live
    pub ttl: Duration,
}

/// Message priority levels
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    /// High priority messages (consensus critical)
    High,
    /// Normal priority messages
    Normal,
    /// Low priority messages
    Low,
}

/// Message routing strategy
#[derive(Clone, Debug)]
pub enum RoutingStrategy {
    /// Direct to known peer
    Direct(Vec<u8>),
    /// Flood to all peers
    Flood,
    /// Random subset of peers
    RandomSubset(usize),
    /// Anonymous route through multiple hops
    Anonymous {
        /// Number of routing hops
        hops: usize,
    },
}

/// Connection status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    /// Connection is active
    Connected,
    /// Connection is being established
    Connecting,
    /// Connection was lost
    Disconnected,
    /// Connection failed
    Failed,
}

/// Network metrics
#[derive(Clone, Debug, Default)]
pub struct NetworkMetrics {
    /// Messages processed per second
    pub messages_per_second: f64,
    /// Current connection count
    pub connections: usize,
    /// Average message latency
    pub avg_latency: Duration,
    /// Memory usage in bytes
    pub memory_usage: usize,
}