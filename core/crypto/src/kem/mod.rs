//! ML-KEM (Kyber) implementation for post-quantum key encapsulation

mod ml_kem;
pub use ml_kem::*;

use thiserror::Error;
use rand::RngCore;
use zeroize::ZeroizeOnDrop;

#[derive(Debug, Error)]
pub enum KEMError {
    #[error("Key generation failed: {0}")]
    KeyGenError(String),
    #[error("Encapsulation failed: {0}")]
    EncapsulationError(String),
    #[error("Decapsulation failed: {0}")]
    DecapsulationError(String),
}

/// ML-KEM key pair
#[derive(Debug, ZeroizeOnDrop)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}