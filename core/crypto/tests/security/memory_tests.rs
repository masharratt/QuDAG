use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};
use zeroize::Zeroize;

#[cfg(test)]
mod memory_security_tests {
    use super::*;

    #[test]
    fn test_mlkem_key_zeroization() {
        let (mut pk, mut sk) = MLKem::keygen();
        
        // Store copies for verification
        let pk_bytes = pk.to_bytes();
        let sk_bytes = sk.to_bytes();
        
        // Trigger zeroize
        pk.zeroize();
        sk.zeroize();
        
        // Verify memory has been cleared
        assert_ne!(pk.to_bytes(), pk_bytes, "Public key was not properly zeroized");
        assert_ne!(sk.to_bytes(), sk_bytes, "Secret key was not properly zeroized");
    }

    #[test]
    fn test_mldsa_key_zeroization() {
        let (mut pk, mut sk) = MLDsa::keygen();
        
        let pk_bytes = pk.to_bytes();
        let sk_bytes = sk.to_bytes();
        
        pk.zeroize();
        sk.zeroize();
        
        assert_ne!(pk.to_bytes(), pk_bytes, "ML-DSA public key was not properly zeroized");
        assert_ne!(sk.to_bytes(), sk_bytes, "ML-DSA secret key was not properly zeroized");
    }

    #[test]
    fn test_hqc_key_zeroization() {
        let (mut pk, mut sk) = HQC::keygen();
        
        let pk_bytes = pk.to_bytes();
        let sk_bytes = sk.to_bytes();
        
        pk.zeroize();
        sk.zeroize();
        
        assert_ne!(pk.to_bytes(), pk_bytes, "HQC public key was not properly zeroized");
        assert_ne!(sk.to_bytes(), sk_bytes, "HQC secret key was not properly zeroized");
    }
}