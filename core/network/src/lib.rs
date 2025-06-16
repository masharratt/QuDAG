#![deny(unsafe_code)]
#![warn(missing_docs)]

//! P2P networking layer with anonymous routing.
//! 
//! This module provides the networking layer for the QuDAG protocol,
//! implementing anonymous routing, P2P communication, and traffic obfuscation.

pub mod connection;
pub mod dark_resolver;
pub mod dns;
pub mod message;
pub mod routing;
pub mod shadow_address;
pub mod types;

pub use dark_resolver::{DarkResolver, DarkResolverError, DarkDomainRecord};
pub use dns::{CloudflareClient, CloudflareConfig, DnsManager, DnsRecord, RecordType, DnsError};
pub use shadow_address::{
    ShadowAddress, ShadowAddressError, ShadowAddressGenerator, ShadowAddressResolver,
    DefaultShadowAddressHandler, NetworkType, ShadowMetadata
};
pub use types::{NetworkAddress, NetworkError};