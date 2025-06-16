#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Quantum-resistant cryptographic primitives for QuDAG protocol.
//! 
//! This module implements the following primitives:
//! - ML-KEM: Key encapsulation mechanism
//! - ML-DSA: Digital signature algorithm
//! - BLAKE3: Cryptographic hash function
//! - Quantum Fingerprint: Data fingerprinting using ML-DSA

pub mod error;
pub mod hash;
pub mod kem;
pub mod ml_kem;
pub mod signature;
pub mod ml_dsa;
pub mod fingerprint;

pub use error::CryptoError;
pub use hash::HashFunction;
pub use kem::{KEMError, KeyEncapsulation, PublicKey, SecretKey, Ciphertext, SharedSecret};
pub use ml_kem::{MlKem768, Metrics as MlKemMetrics};
pub use signature::{DigitalSignature, SignatureError};