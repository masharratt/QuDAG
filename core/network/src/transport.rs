//! Network transport layer implementation.

use thiserror::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};

/// Errors that can occur during transport operations.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Connection failed
    #[error("Connection failed")]
    ConnectionFailed,
    
    /// Read error
    #[error("Read error")]
    ReadError,
    
    /// Write error
    #[error("Write error")]
    WriteError,
    
    /// TLS error
    #[error("TLS error")]
    TlsError,
}

/// Transport encryption configuration.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Enable TLS encryption
    pub use_tls: bool,
    
    /// Certificate path
    pub cert_path: Option<String>,
    
    /// Private key path
    pub key_path: Option<String>,
}

/// Async transport stream trait.
pub trait AsyncTransport: AsyncRead + AsyncWrite + Send + Unpin {}

/// Network transport trait defining the interface for transport operations.
pub trait Transport {
    /// Initialize transport with configuration.
    fn init(&mut self, config: TransportConfig) -> Result<(), TransportError>;
    
    /// Create a new connection to a remote peer.
    fn connect(&mut self, addr: SocketAddr) -> Result<Box<dyn AsyncTransport>, TransportError>;
    
    /// Accept an incoming connection.
    fn accept(&mut self) -> Result<Box<dyn AsyncTransport>, TransportError>;
    
    /// Close a connection.
    fn close(&mut self, stream: Box<dyn AsyncTransport>) -> Result<(), TransportError>;
    
    /// Get active connections.
    fn get_connections(&self) -> Vec<SocketAddr>;
}