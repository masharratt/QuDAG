use serde::{Serialize, Deserialize};
use std::time::Duration;
use std::net::{IpAddr, Ipv4Addr};
use thiserror::Error;
use blake3::Hash;

/// Network address combining IP and port
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkAddress {
    /// IP address
    pub ip: IpAddr,
    /// Port number
    pub port: u16,
}

impl NetworkAddress {
    /// Create a new network address from IPv4 address parts and port
    pub fn new(ip_parts: [u8; 4], port: u16) -> Self {
        Self {
            ip: IpAddr::V4(Ipv4Addr::new(ip_parts[0], ip_parts[1], ip_parts[2], ip_parts[3])),
            port,
        }
    }
    
    /// Create a new network address from IP and port
    pub fn from_ip_port(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }
    
    /// Get the socket address as a string
    pub fn to_socket_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

/// Network errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("Message handling failed: {0}")]
    MessageError(String),

    #[error("Routing failed: {0}")]
    RoutingError(String),

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePriority {
    /// High priority messages
    High,
    /// Normal priority messages
    Normal,
    /// Low priority messages
    Low,
}

/// Message routing strategy
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    /// Direct to peer
    Direct(Vec<u8>),
    /// Flood to all peers
    Flood,
    /// Random subset of peers
    RandomSubset(usize),
    /// Anonymous routing
    Anonymous {
        /// Number of hops
        hops: usize,
    },
}

/// Routing layer for onion routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingLayer {
    /// Next hop
    pub next_hop: Vec<u8>,
    /// Encrypted payload
    pub payload: Vec<u8>,
    /// Layer metadata
    pub metadata: LayerMetadata,
}

/// Routing layer metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetadata {
    /// Time-to-live
    pub ttl: Duration,
    /// Flags
    pub flags: u32,
    /// Layer ID
    pub id: String,
}

/// Network metrics
#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    /// Messages per second
    pub messages_per_second: f64,
    /// Current connections
    pub connections: usize,
    /// Average message latency
    pub avg_latency: Duration,
    /// Memory usage in bytes
    pub memory_usage: usize,
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Handshake message
    Handshake {
        /// Protocol version
        version: u32,
        /// Node ID
        node_id: Vec<u8>,
    },
    /// Data message
    Data {
        /// Message ID
        id: String,
        /// Payload
        payload: Vec<u8>,
        /// Priority
        priority: MessagePriority,
    },
    /// Control message
    Control {
        /// Command
        command: String,
        /// Parameters
        params: Vec<String>,
    },
}

/// Network message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessage {
    /// Message identifier
    pub id: String,
    /// Source node identifier
    pub source: Vec<u8>,
    /// Destination node identifier
    pub destination: Vec<u8>,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message priority
    pub priority: MessagePriority,
    /// Time to live
    pub ttl: Duration,
}

/// Peer identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId([u8; 32]);

impl PeerId {
    /// Generate a random peer ID
    pub fn random() -> Self {
        use rand::RngCore;
        let mut id = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut id);
        Self(id)
    }
    
    /// Create a peer ID from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    /// Get the peer ID as bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }
    
    /// Get the peer ID as a slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}