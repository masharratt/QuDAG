//! ML-KEM (Kyber) implementation for post-quantum key encapsulation

mod ml_kem;
pub use ml_kem::*;

use thiserror::Error;
use rand::RngCore;
use zeroize::ZeroizeOnDrop;

#[derive(Debug, Error)]
pub enum KEMError {
    #[error("Key generation failed")]
    KeyGenError,
    
    #[error("Encapsulation failed")]
    EncapsulationError,
    
    #[error("Decapsulation failed")]
    DecapsulationError,
    
    #[error("Invalid key")]
    InvalidKey,
    
    #[error("Invalid parameters")]
    InvalidParameters,
    
    #[error("Operation failed")]
    OperationFailed,
    
    #[error("Internal error")]
    InternalError,
}

// Ensure errors don't leak sensitive information in their Display impl
impl std::fmt::Display for KEMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KEM operation failed")
    }
}

/// ML-KEM key pair
#[derive(Debug, ZeroizeOnDrop)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}