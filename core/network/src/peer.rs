//! P2P network peer management implementation.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::net::SocketAddr;

/// Errors that can occur during peer operations.
#[derive(Debug, Error)]
pub enum PeerError {
    /// Connection failed
    #[error("Connection failed")]
    ConnectionFailed,
    
    /// Peer not found
    #[error("Peer not found")]
    PeerNotFound,
    
    /// Invalid peer address
    #[error("Invalid peer address")]
    InvalidAddress,
    
    /// Handshake failed
    #[error("Handshake failed")]
    HandshakeFailed,
}

/// Unique peer identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(Vec<u8>);

/// Peer connection status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerStatus {
    /// Initial connection attempt
    Connecting,
    
    /// Connected and handshake complete
    Connected,
    
    /// Connection lost
    Disconnected,
    
    /// Banned or blacklisted
    Banned,
}

/// Network peer information.
#[derive(Debug, Clone)]
pub struct Peer {
    /// Unique peer identifier
    pub id: PeerId,
    
    /// Network address
    pub address: SocketAddr,
    
    /// Connection status
    pub status: PeerStatus,
    
    /// Protocol version
    pub version: u32,
}

/// Peer management trait defining the interface for peer operations.
pub trait PeerManager {
    /// Add a new peer to the network.
    fn add_peer(&mut self, address: SocketAddr) -> Result<PeerId, PeerError>;
    
    /// Remove a peer from the network.
    fn remove_peer(&mut self, peer_id: &PeerId) -> Result<(), PeerError>;
    
    /// Get information about a specific peer.
    fn get_peer(&self, peer_id: &PeerId) -> Result<Peer, PeerError>;
    
    /// Get list of all connected peers.
    fn get_peers(&self) -> Vec<Peer>;
    
    /// Ban a peer from the network.
    fn ban_peer(&mut self, peer_id: &PeerId) -> Result<(), PeerError>;
    
    /// Check if a peer is banned.
    fn is_banned(&self, peer_id: &PeerId) -> bool;
}