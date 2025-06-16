#![deny(unsafe_code)]
#![allow(missing_docs)]

//! P2P networking layer with anonymous routing.
//! 
//! This module provides the networking layer for the QuDAG protocol,
//! implementing anonymous routing, P2P communication, and traffic obfuscation.

pub mod connection;
pub mod dark_resolver;
pub mod dns;
pub mod message;
pub mod onion;
pub mod routing;
pub mod router;
pub mod shadow_address;
pub mod transport;
pub mod types;

pub use dark_resolver::{DarkResolver, DarkResolverError, DarkDomainRecord};
pub use dns::{CloudflareClient, CloudflareConfig, DnsManager, DnsRecord, RecordType, DnsError};
pub use shadow_address::{
    ShadowAddress, ShadowAddressError, ShadowAddressGenerator, ShadowAddressResolver,
    DefaultShadowAddressHandler, NetworkType, ShadowMetadata
};
pub use types::{
    NetworkAddress, NetworkError, NetworkMessage, PeerId, MessagePriority, RoutingStrategy,
    ConnectionStatus, QueueMetrics, LatencyMetrics, ThroughputMetrics
};
pub use message::MessageEnvelope;
pub use onion::{
    OnionLayer, OnionRouter, MLKEMOnionRouter, OnionError,
    MixNode, MixConfig, MixMessage, MixMessageType, MixNodeStats,
    MetadataProtector, MetadataConfig, ProtectedMetadata,
    TrafficAnalysisResistance, TrafficAnalysisConfig
};
pub use router::{Router, HopInfo};
pub use transport::{Transport, TransportConfig, TransportError, AsyncTransport};

/// Network manager for test compatibility
pub struct NetworkManager {
    // Placeholder implementation
}

impl NetworkManager {
    /// Create new network manager
    pub fn new() -> Self {
        Self {}
    }
}
pub use connection::{ConnectionManager, SecureConnection, SecureConfig, TransportKeys};