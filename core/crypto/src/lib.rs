#![deny(unsafe_code)]
#![allow(missing_docs)]

//! Quantum-resistant cryptographic primitives for QuDAG protocol.
//! 
//! This module implements the following primitives:
//! - ML-KEM: Key encapsulation mechanism
//! - ML-DSA: Digital signature algorithm
//! - HQC: Hamming Quasi-Cyclic code-based encryption
//! - BLAKE3: Cryptographic hash function
//! - Quantum Fingerprint: Data fingerprinting using ML-DSA

pub mod error;
pub mod hash;
pub mod kem;
pub mod ml_kem;
pub mod signature;
pub mod ml_dsa;
pub mod fingerprint;
pub mod hqc;
pub mod encryption;

pub use error::CryptoError;
pub use hash::HashFunction;
pub use kem::{KEMError, KeyEncapsulation, PublicKey, SecretKey, Ciphertext, SharedSecret, KeyPair};
pub use ml_kem::{MlKem768, Metrics as MlKemMetrics};
pub use signature::{DigitalSignature, SignatureError};
pub use ml_dsa::{MlDsa, MlDsaKeyPair, MlDsaPublicKey, MlDsaError};
pub use fingerprint::{Fingerprint, FingerprintError};
pub use hqc::{HqcError, SecurityParameter};