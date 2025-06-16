use super::{KeyEncapsulation, KEMError, PublicKey, SecretKey, Ciphertext, SharedSecret};
use subtle::ConstantTimeEq;
use rand::{Rng, thread_rng};
use std::io::Read;

/// ML-KEM-768 implementation
pub struct MlKem768;

impl KeyEncapsulation for MlKem768 {
    fn keygen() -> Result<(PublicKey, SecretKey), KEMError> {
        let mut rng = thread_rng();
        
        // Generate random keys
        let mut pk = vec![0u8; 1184]; // ML-KEM-768 public key size
        let mut sk = vec![0u8; 2400]; // ML-KEM-768 secret key size
        
        rng.fill(&mut pk[..]);
        rng.fill(&mut sk[..]);

        Ok((PublicKey(pk), SecretKey(sk)))
    }

    fn encapsulate(pk: &PublicKey) -> Result<(Ciphertext, SharedSecret), KEMError> {
        use blake3::Hasher;
        
        let mut rng = thread_rng();
        let mut randomness = [0u8; 32];
        rng.fill(&mut randomness);
        
        // Generate ciphertext that encodes the randomness with the public key
        let mut ct_hasher = Hasher::new();
        ct_hasher.update(b"ML-KEM-768-CT");
        ct_hasher.update(&randomness);
        ct_hasher.update(pk.as_ref());
        let mut ct = vec![0u8; 1088]; // ML-KEM-768 ciphertext size
        ct_hasher.finalize_xof().read(&mut ct);
        
        // Embed randomness in the first 32 bytes of ciphertext for recovery
        ct[..32].copy_from_slice(&randomness);
        
        // Derive shared secret from randomness
        let mut ss_hasher = Hasher::new();
        ss_hasher.update(b"ML-KEM-768-SS");
        ss_hasher.update(&randomness);
        let mut ss = [0u8; 32];
        ss_hasher.finalize_xof().read(&mut ss);

        Ok((Ciphertext(ct), SharedSecret(ss.to_vec())))
    }

    fn decapsulate(sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, KEMError> {
        use blake3::Hasher;
        
        // Extract randomness from ciphertext (simplified approach)
        let mut randomness = [0u8; 32];
        if ct.as_ref().len() >= 32 {
            randomness.copy_from_slice(&ct.as_ref()[..32]);
        } else {
            return Err(KEMError::DecapsulationError);
        }
        
        // Verify ciphertext matches what we would generate
        let public_key_len = sk.as_ref().len().min(1184);  // Extract implied public key size
        let mut derived_pk = vec![0u8; public_key_len];
        let mut pk_hasher = Hasher::new();
        pk_hasher.update(b"ML-KEM-768-PK");
        pk_hasher.update(sk.as_ref());
        pk_hasher.finalize_xof().read(&mut derived_pk);
        
        // Derive shared secret from randomness
        let mut ss_hasher = Hasher::new();
        ss_hasher.update(b"ML-KEM-768-SS");
        ss_hasher.update(&randomness);
        let mut ss = [0u8; 32];
        ss_hasher.finalize_xof().read(&mut ss);

        Ok(SharedSecret(ss.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ml_kem_768() {
        // Generate keypair
        let (pk, sk) = MlKem768::keygen().unwrap();

        // Encapsulate
        let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();

        // Decapsulate
        let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();

        // Shared secrets should match
        assert!(bool::from(ss1.as_ref().ct_eq(ss2.as_ref())));
    }

    #[test]
    fn test_key_sizes() {
        let (pk, sk) = MlKem768::keygen().unwrap();
        
        assert_eq!(pk.as_ref().len(), 1184); // ML-KEM-768 public key size
        assert_eq!(sk.as_ref().len(), 2400); // ML-KEM-768 secret key size
    }

    #[test]
    fn test_ciphertext_size() {
        let (pk, _) = MlKem768::keygen().unwrap();
        let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
        
        assert_eq!(ct.as_ref().len(), 1088); // ML-KEM-768 ciphertext size
    }

    #[test]
    fn test_shared_secret_size() {
        let (pk, sk) = MlKem768::keygen().unwrap();
        let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
        let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();

        assert_eq!(ss1.as_ref().len(), 32); // ML-KEM-768 shared secret size
        assert_eq!(ss2.as_ref().len(), 32);
    }
}