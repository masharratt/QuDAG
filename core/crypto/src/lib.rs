//! Quantum-resistant cryptographic primitives for QuDAG Protocol
//!
//! This module implements post-quantum cryptographic primitives:
//! - ML-KEM for key encapsulation
//! - ML-DSA for digital signatures
//! - HQC for public key encryption
//!
//! All implementations are constant-time and memory-safe.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

pub mod kem;
pub mod signatures;
pub mod encryption;

/// Error type for KEM operations
#[derive(Error, Debug)]
pub enum KEMError {
    /// Error during key generation
    #[error("Key generation error: {0}")]
    KeyGenError(String),
    /// Error during encapsulation
    #[error("Encapsulation error: {0}")]
    EncapsulationError(String),
    /// Error during decapsulation
    #[error("Decapsulation error: {0}")]
    DecapsulationError(String),
}

/// Error type for signature operations
#[derive(Error, Debug)]
pub enum SignatureError {
    /// Error during key generation
    #[error("Key generation error: {0}")]
    KeyGenError(String),
    /// Error during signing
    #[error("Signing error: {0}")]
    SignError(String),
    /// Error during verification
    #[error("Verification error: {0}")]
    VerifyError(String),
}

/// Error type for encryption operations
#[derive(Error, Debug)]
pub enum EncryptionError {
    /// Error during key generation
    #[error("Key generation error: {0}")]
    KeyGenError(String),
    /// Error during encryption
    #[error("Encryption error: {0}")]
    EncryptError(String),
    /// Error during decryption
    #[error("Decryption error: {0}")]
    DecryptError(String),
}

/// Constant-time utilities for cryptographic operations
mod utils {
    use subtle::{Choice, ConstantTimeEq};
    
    /// Performs constant-time comparison of byte slices
    pub(crate) fn constant_time_compare(a: &[u8], b: &[u8]) -> Choice {
        if a.len() != b.len() {
            return Choice::from(0u8);
        }
        a.ct_eq(b)
    }
    
    /// Performs constant-time conditional copy
    pub(crate) fn conditional_copy(condition: Choice, src: &[u8], dst: &mut [u8]) {
        assert_eq!(src.len(), dst.len());
        for (s, d) in src.iter().zip(dst.iter_mut()) {
            *d = condition.select(*s, *d);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_constant_time_utils() {
        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3, 4];
        let c = vec![1, 2, 3, 5];
        
        assert!(bool::from(utils::constant_time_compare(&a, &b)));
        assert!(!bool::from(utils::constant_time_compare(&a, &c)));
        
        let mut dst = vec![0, 0, 0, 0];
        utils::conditional_copy(Choice::from(1u8), &a, &mut dst);
        assert_eq!(dst, a);
    }
}
