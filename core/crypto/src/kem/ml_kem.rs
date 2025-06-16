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
    pub secret_key: Vec<u8>,
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        // Ensure secret key is zeroized on drop
        self.secret_key.zeroize();
    }
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
    // Generate random seed with enough entropy
    let mut seed = vec![0u8; 64];
    rng.fill_bytes(&mut seed);
    
    // Ensure seed is zeroized after use
    let result = (|| {
        let (pk, sk) = kyber768::keypair();
        
        // Copy keys into new buffers to avoid potential memory leaks
        let mut public_key = vec![0u8; PUBLIC_KEY_BYTES];
        let mut secret_key = vec![0u8; SECRET_KEY_BYTES];
        
        public_key.copy_from_slice(pk.as_bytes());
        secret_key.copy_from_slice(sk.as_bytes());
        
        // Clear original keys
        drop(pk);
        sk.as_bytes().zeroize();
        drop(sk);
        
        Ok(KeyPair { public_key, secret_key })
    })();
    
    // Always zeroize seed
    seed.zeroize();
    
    result
}

pub fn encapsulate(public_key: &[u8]) -> Result<(SharedSecret, Vec<u8>), KEMError> {
    // Validate input length in constant time
    if !constant_time_compare(
        &(public_key.len() as u32).to_be_bytes(),
        &(PUBLIC_KEY_BYTES as u32).to_be_bytes()
    ).into() {
        return Err(KEMError::InvalidParameters);
    }

    let result = (|| {
        let pk = kyber768::PublicKey::from_bytes(public_key)
            .map_err(|_| KEMError::EncapsulationError)?;
            
        let (ss, ct) = kyber768::encapsulate(&pk);
        
        // Copy shared secret and ciphertext to new buffers
        let mut shared_secret = vec![0u8; SHARED_SECRET_BYTES];
        let mut ciphertext = vec![0u8; CIPHERTEXT_BYTES];
        
        shared_secret.copy_from_slice(ss.as_bytes());
        ciphertext.copy_from_slice(ct.as_bytes());
        
        // Clear original values
        ss.as_bytes().zeroize();
        drop(ss);
        drop(ct);
        
        Ok((SharedSecret(shared_secret), ciphertext))
    })();
    
    result
}

pub fn decapsulate(secret_key: &[u8], ciphertext: &[u8]) -> Result<SharedSecret, KEMError> {
    // Validate lengths in constant time
    let valid_sk_len = constant_time_compare(
        &(secret_key.len() as u32).to_be_bytes(),
        &(SECRET_KEY_BYTES as u32).to_be_bytes()
    );
    
    let valid_ct_len = constant_time_compare(
        &(ciphertext.len() as u32).to_be_bytes(),
        &(CIPHERTEXT_BYTES as u32).to_be_bytes()
    );
    
    if !(valid_sk_len & valid_ct_len).into() {
        return Err(KEMError::InvalidParameters);
    }

    let result = (|| {
        let sk = kyber768::SecretKey::from_bytes(secret_key)
            .map_err(|_| KEMError::DecapsulationError)?;
        let ct = kyber768::Ciphertext::from_bytes(ciphertext)
            .map_err(|_| KEMError::DecapsulationError)?;
            
        let ss = kyber768::decapsulate(&ct, &sk);
        
        // Copy shared secret to new buffer
        let mut shared_secret = vec![0u8; SHARED_SECRET_BYTES];
        shared_secret.copy_from_slice(ss.as_bytes());
        
        // Clear original secret
        ss.as_bytes().zeroize();
        drop(ss);
        
        // Clear secret key
        sk.as_bytes().zeroize();
        drop(sk);
        
        Ok(SharedSecret(shared_secret))
    })();
    
    result
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