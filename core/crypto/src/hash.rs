//! Cryptographic hash functions implementation.

use thiserror::Error;

/// Errors that can occur during hash operations.
#[derive(Debug, Error)]
pub enum HashError {
    /// Input data is too large
    #[error("Input data is too large")]
    InputTooLarge,
    
    /// Hash computation failed
    #[error("Hash computation failed")]
    ComputationFailed,
}

/// Hash function output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest(Vec<u8>);

/// Cryptographic hash function trait.
pub trait HashFunction {
    /// Create a new hash instance.
    fn new() -> Self;
    
    /// Update the hash state with input data.
    fn update(&mut self, data: &[u8]) -> Result<(), HashError>;
    
    /// Finalize the hash computation and return the digest.
    fn finalize(self) -> Result<Digest, HashError>;
    
    /// Compute hash of input data in one step.
    fn hash(data: &[u8]) -> Result<Digest, HashError>;
}