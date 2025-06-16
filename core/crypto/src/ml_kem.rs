use crate::kem::{KEMError, KeyEncapsulation};
use rand_core::{CryptoRng, RngCore};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// ML-KEM-768 implementation (NIST security level 3)
pub struct MlKem768;

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct PublicKey([u8; MlKem768::PUBLIC_KEY_SIZE]);

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct SecretKey([u8; MlKem768::SECRET_KEY_SIZE]);

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct Ciphertext([u8; MlKem768::CIPHERTEXT_SIZE]);

#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct SharedSecret([u8; MlKem768::SHARED_SECRET_SIZE]);

impl MlKem768 {
    // Constants for ML-KEM-768
    const SHARED_SECRET_SIZE: usize = 32;
    const PUBLIC_KEY_SIZE: usize = 1184;
    const SECRET_KEY_SIZE: usize = 2400;
    const CIPHERTEXT_SIZE: usize = 1088;
}

impl KeyEncapsulation for MlKem768 {
    type PublicKey = PublicKey;
    type SecretKey = SecretKey;
    type Ciphertext = Ciphertext;
    type SharedSecret = SharedSecret;

    const PUBLIC_KEY_SIZE: usize = Self::PUBLIC_KEY_SIZE;
    const SECRET_KEY_SIZE: usize = Self::SECRET_KEY_SIZE;
    const CIPHERTEXT_SIZE: usize = Self::CIPHERTEXT_SIZE;
    const SHARED_SECRET_SIZE: usize = Self::SHARED_SECRET_SIZE;

    fn keygen() -> Result<(Self::PublicKey, Self::SecretKey), KEMError> {
        let mut rng = rand::thread_rng();
        let keypair = crate::kem::ml_kem::generate_keypair(&mut rng)?;

        // Convert to fixed-size arrays
        let mut pk = [0u8; Self::PUBLIC_KEY_SIZE];
        let mut sk = [0u8; Self::SECRET_KEY_SIZE];
        
        pk.copy_from_slice(&keypair.public_key);
        sk.copy_from_slice(&keypair.secret_key);

        Ok((PublicKey(pk), SecretKey(sk)))
    }

    fn encapsulate(pk: &Self::PublicKey) -> Result<(Self::Ciphertext, Self::SharedSecret), KEMError> {
        let (shared_secret, ciphertext) = crate::kem::ml_kem::encapsulate(pk.as_ref())?;
        
        let mut ct = [0u8; Self::CIPHERTEXT_SIZE];
        let mut ss = [0u8; Self::SHARED_SECRET_SIZE];
        
        ct.copy_from_slice(&ciphertext);
        ss.copy_from_slice(shared_secret.as_bytes());

        Ok((Ciphertext(ct), SharedSecret(ss)))
    }

    fn decapsulate(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::SharedSecret, KEMError> {
        let shared_secret = crate::kem::ml_kem::decapsulate(sk.as_ref(), ct.as_ref())?;
        
        let mut ss = [0u8; Self::SHARED_SECRET_SIZE];
        ss.copy_from_slice(shared_secret.as_bytes());

        Ok(SharedSecret(ss))
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SecretKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for Ciphertext {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SharedSecret {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}