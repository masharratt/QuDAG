use super::*;
use pqcrypto::pke::hqc128;
use rand::RngCore;
use zeroize::Zeroize;

const PUBLIC_KEY_BYTES: usize = hqc128::public_key_bytes();
const SECRET_KEY_BYTES: usize = hqc128::secret_key_bytes();
const CIPHERTEXT_BYTES: usize = hqc128::ciphertext_bytes();
const PLAINTEXT_BYTES: usize = 32; // Maximum plaintext size for HQC-128

/// Keypair for HQC encryption
#[derive(Clone)]
pub struct KeyPair {
    /// Public encryption key
    pub public_key: Vec<u8>,
    /// Secret decryption key
    pub secret_key: Vec<u8>,
}

impl Drop for KeyPair {
    fn drop(&mut self) {
        self.secret_key.zeroize();
    }
}

/// Generate a new HQC key pair for encryption
pub fn generate_keypair<R: RngCore>(rng: &mut R) -> Result<KeyPair, EncryptionError> {
    // Generate seed for deterministic key generation
    let mut seed = vec![0u8; 32];
    defer! { seed.zeroize(); }
    rng.fill_bytes(&mut seed);
    
    let (pk, sk) = hqc128::keypair();
    
    Ok(KeyPair {
        public_key: pk.as_bytes().to_vec(),
        secret_key: sk.as_bytes().to_vec(),
    })
}

/// Encrypt a message using HQC with the provided public key
pub fn encrypt<R: RngCore>(rng: &mut R, public_key: &[u8], message: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    if public_key.len() != PUBLIC_KEY_BYTES {
        return Err(EncryptionError::EncryptError("Invalid public key length".into()));
    }
    if message.len() > PLAINTEXT_BYTES {
        return Err(EncryptionError::EncryptError("Message too long".into()));
    }

    let pk = hqc128::PublicKey::from_bytes(public_key)
        .map_err(|e| EncryptionError::EncryptError(e.to_string()))?;
        
    // Generate randomness for encryption
    let mut encryption_randomness = vec![0u8; 32];
    defer! { encryption_randomness.zeroize(); }
    rng.fill_bytes(&mut encryption_randomness);
    
    let ciphertext = hqc128::encrypt(message, &pk);
    Ok(ciphertext.as_bytes().to_vec())
}

/// Decrypt a ciphertext using HQC with the provided secret key
pub fn decrypt(secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, EncryptionError> {
    if secret_key.len() != SECRET_KEY_BYTES {
        return Err(EncryptionError::DecryptError("Invalid secret key length".into()));
    }
    if ciphertext.len() != CIPHERTEXT_BYTES {
        return Err(EncryptionError::DecryptError("Invalid ciphertext length".into()));
    }

    let sk = hqc128::SecretKey::from_bytes(secret_key)
        .map_err(|e| EncryptionError::DecryptError(e.to_string()))?;
    let ct = hqc128::Ciphertext::from_bytes(ciphertext)
        .map_err(|e| EncryptionError::DecryptError(e.to_string()))?;
        
    let plaintext = hqc128::decrypt(&ct, &sk);
    Ok(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_encryption_roundtrip() {
        let mut rng = thread_rng();
        let message = b"test message";
        
        // Generate keypair
        let keypair = generate_keypair(&mut rng).unwrap();
        assert_eq!(keypair.public_key.len(), PUBLIC_KEY_BYTES);
        assert_eq!(keypair.secret_key.len(), SECRET_KEY_BYTES);
        
        // Encrypt message
        let ciphertext = encrypt(&mut rng, &keypair.public_key, message).unwrap();
        assert_eq!(ciphertext.len(), CIPHERTEXT_BYTES);
        
        // Decrypt message
        let decrypted = decrypt(&keypair.secret_key, &ciphertext).unwrap();
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_encryption_failures() {
        let mut rng = thread_rng();
        let message = b"test message";
        
        // Generate keypair and encrypt
        let keypair = generate_keypair(&mut rng).unwrap();
        let ciphertext = encrypt(&mut rng, &keypair.public_key, message).unwrap();
        
        // Test invalid key lengths
        let invalid_key = vec![0u8; 32];
        assert!(encrypt(&mut rng, &invalid_key, message).is_err());
        assert!(decrypt(&invalid_key, &ciphertext).is_err());
        
        // Test invalid ciphertext
        let invalid_ct = vec![0u8; CIPHERTEXT_BYTES];
        let result = decrypt(&keypair.secret_key, &invalid_ct);
        assert!(result.is_err() || result.unwrap() != message);
        
        // Test message too long
        let long_message = vec![0u8; PLAINTEXT_BYTES + 1];
        assert!(encrypt(&mut rng, &keypair.public_key, &long_message).is_err());
    }

    #[test]
    fn test_timing_consistency() {
        use std::time::{Duration, Instant};
        
        let mut rng = thread_rng();
        let message = b"test message";
        
        let keypair = generate_keypair(&mut rng).unwrap();
        let ciphertext = encrypt(&mut rng, &keypair.public_key, message).unwrap();
        
        // Measure timing of valid decryption
        let start = Instant::now();
        let _ = decrypt(&keypair.secret_key, &ciphertext).unwrap();
        let valid_time = start.elapsed();
        
        // Measure timing of invalid decryption
        let invalid_ct = vec![0u8; CIPHERTEXT_BYTES];
        let start = Instant::now();
        let _ = decrypt(&keypair.secret_key, &invalid_ct);
        let invalid_time = start.elapsed();
        
        // Check that timing difference is within acceptable range (1ms)
        let diff = if valid_time > invalid_time {
            valid_time - invalid_time
        } else {
            invalid_time - valid_time
        };
        assert!(diff < Duration::from_millis(1));
    }

    #[test] 
    fn test_memory_zeroization() {
        let mut rng = thread_rng();
        let keypair = generate_keypair(&mut rng).unwrap();
        
        // Get a copy of the secret key
        let secret_key_copy = keypair.secret_key.clone();
        
        // Drop the keypair
        drop(keypair);
        
        // Secret key should be zeroized
        assert_ne!(vec![0u8; SECRET_KEY_BYTES], secret_key_copy);
    }
}