//! P2P network peer discovery implementation.

use thiserror::Error;
use crate::peer::{Peer, PeerId};
use std::net::SocketAddr;

/// Errors that can occur during peer discovery.
#[derive(Debug, Error)]
pub enum DiscoveryError {
    /// Discovery service failed
    #[error("Discovery service failed")]
    ServiceFailed,
    
    /// Invalid peer information
    #[error("Invalid peer information")]
    InvalidPeerInfo,
    
    /// DHT operation failed
    #[error("DHT operation failed")]
    DhtFailed,
}

/// Peer discovery method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// DHT-based discovery
    Dht,
    
    /// Static peer list
    Static,
    
    /// DNS-based discovery
    Dns,
    
    /// Bootstrap node discovery
    Bootstrap,
}

/// Peer discovery configuration.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Discovery methods to use
    pub methods: Vec<DiscoveryMethod>,
    
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<SocketAddr>,
    
    /// Discovery interval in seconds
    pub interval: u64,
    
    /// Maximum peers to discover
    pub max_peers: usize,
}

/// Peer discovery trait defining the interface for peer discovery.
pub trait PeerDiscovery {
    /// Initialize peer discovery with configuration.
    fn init(config: DiscoveryConfig) -> Result<(), DiscoveryError>;
    
    /// Start peer discovery service.
    fn start_discovery(&mut self) -> Result<(), DiscoveryError>;
    
    /// Stop peer discovery service.
    fn stop_discovery(&mut self) -> Result<(), DiscoveryError>;
    
    /// Discover new peers.
    fn discover_peers(&mut self) -> Result<Vec<Peer>, DiscoveryError>;
    
    /// Announce peer to the network.
    fn announce(&mut self, peer_id: &PeerId) -> Result<(), DiscoveryError>;
    
    /// Get known peers from discovery service.
    fn get_known_peers(&self) -> Vec<Peer>;
}