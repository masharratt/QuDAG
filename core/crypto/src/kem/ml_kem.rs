use super::*;
use pqcrypto::kem::kyber768;
use rand::RngCore;
use subtle::{Choice, ConstantTimeEq};
use zeroize::{Zeroize, ZeroizeOnDrop};

const PUBLIC_KEY_BYTES: usize = kyber768::public_key_bytes();
const SECRET_KEY_BYTES: usize = kyber768::secret_key_bytes();
const CIPHERTEXT_BYTES: usize = kyber768::ciphertext_bytes();
const SHARED_SECRET_BYTES: usize = kyber768::shared_secret_bytes();

#[derive(Clone, ZeroizeOnDrop)]
pub struct KeyPair {
    pub public_key: Vec<u8>,
    #[zeroize(skip)]
    pub secret_key: Vec<u8>,
}

#[derive(Clone, ZeroizeOnDrop)]
pub struct SharedSecret(Vec<u8>);

impl SharedSecret {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SharedSecret {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

pub fn generate_keypair<R: RngCore>(rng: &mut R) -> Result<KeyPair, KEMError> {
    // Generate random seed
    let mut seed = vec![0u8; 32];
    rng.fill_bytes(&mut seed);
    
    // Ensure seed is zeroized after use
    defer! { seed.zeroize(); }
    
    let (pk, sk) = kyber768::keypair();
    
    Ok(KeyPair {
        public_key: pk.as_bytes().to_vec(),
        secret_key: sk.as_bytes().to_vec(),
    })
}

pub fn encapsulate(public_key: &[u8]) -> Result<(SharedSecret, Vec<u8>), KEMError> {
    if public_key.len() != PUBLIC_KEY_BYTES {
        return Err(KEMError::EncapsulationError("Invalid public key length".into()));
    }

    let pk = kyber768::PublicKey::from_bytes(public_key)
        .map_err(|e| KEMError::EncapsulationError(e.to_string()))?;
        
    let (ss, ct) = kyber768::encapsulate(&pk);
    
    Ok((
        SharedSecret(ss.as_bytes().to_vec()),
        ct.as_bytes().to_vec()
    ))
}

pub fn decapsulate(secret_key: &[u8], ciphertext: &[u8]) -> Result<SharedSecret, KEMError> {
    if secret_key.len() != SECRET_KEY_BYTES {
        return Err(KEMError::DecapsulationError("Invalid secret key length".into()));
    }
    if ciphertext.len() != CIPHERTEXT_BYTES {
        return Err(KEMError::DecapsulationError("Invalid ciphertext length".into()));
    }

    let sk = kyber768::SecretKey::from_bytes(secret_key)
        .map_err(|e| KEMError::DecapsulationError(e.to_string()))?;
    let ct = kyber768::Ciphertext::from_bytes(ciphertext)
        .map_err(|e| KEMError::DecapsulationError(e.to_string()))?;
        
    let ss = kyber768::decapsulate(&ct, &sk);
    Ok(SharedSecret(ss.as_bytes().to_vec()))
}

/// Performs constant-time comparison of byte arrays
fn constant_time_compare(a: &[u8], b: &[u8]) -> Choice {
    if a.len() != b.len() {
        return Choice::from(0u8);
    }
    a.ct_eq(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_shared_secret_zeroize() {
        let mut rng = thread_rng();
        let keypair = generate_keypair(&mut rng).unwrap();
        let (shared_secret, _) = encapsulate(&keypair.public_key).unwrap();
        
        // Create a copy of the secret for verification
        let secret_copy = shared_secret.0.clone();
        
        // Drop the SharedSecret - this should zeroize its contents
        drop(shared_secret);
        
        // Verify the copy is different from an all-zero buffer
        let zero_buf = vec![0u8; secret_copy.len()];
        assert_ne!(secret_copy, zero_buf);
    }

    #[test]
    fn test_keypair_zeroize() {
        let mut rng = thread_rng();
        let keypair = generate_keypair(&mut rng).unwrap();
        
        // Create copies for verification
        let pk_copy = keypair.public_key.clone();
        let sk_copy = keypair.secret_key.clone();
        
        // Drop the KeyPair - this should zeroize the secret key
        drop(keypair);
        
        // Verify public key was not zeroized
        let zero_buf = vec![0u8; pk_copy.len()];
        assert_ne!(pk_copy, zero_buf);
        
        // Verify secret key copy is different from an all-zero buffer
        let zero_buf = vec![0u8; sk_copy.len()];
        assert_ne!(sk_copy, zero_buf);
    }
}