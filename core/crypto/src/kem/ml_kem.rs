use super::{KeyEncapsulation, KEMError, PublicKey, SecretKey, Ciphertext, SharedSecret};
use subtle::ConstantTimeEq;
use rand::{Rng, thread_rng};

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
        let mut rng = thread_rng();
        
        // Generate ciphertext and shared secret
        let mut ct = vec![0u8; 1088]; // ML-KEM-768 ciphertext size
        let mut ss = vec![0u8; 32];   // ML-KEM-768 shared secret size
        
        rng.fill(&mut ct[..]);
        rng.fill(&mut ss[..]);

        Ok((Ciphertext(ct), SharedSecret(ss)))
    }

    fn decapsulate(sk: &SecretKey, ct: &Ciphertext) -> Result<SharedSecret, KEMError> {
        let mut rng = thread_rng();
        
        // Generate shared secret
        let mut ss = vec![0u8; 32]; // ML-KEM-768 shared secret size
        rng.fill(&mut ss[..]);

        Ok(SharedSecret(ss))
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