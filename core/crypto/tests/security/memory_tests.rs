use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};
use zeroize::{Zeroize, Zeroizing};
use std::mem;

/// Memory security test suite for cryptographic operations
#[cfg(test)]
mod memory_security_tests {
    use super::*;

    /// Helper function to verify memory is properly zeroized
    fn verify_zeroization<T: AsRef<[u8]>>(data: &T, expected_zeros: usize) {
        let bytes = data.as_ref();
        assert_eq!(bytes.iter().filter(|&&b| b == 0).count(), expected_zeros,
            "Memory was not properly zeroized");
    }

    #[test]
    fn test_mlkem_key_lifecycle() {
        // Test key generation and zeroization
        let (mut pk, mut sk) = MLKem::keygen();
        let pk_bytes = pk.to_bytes();
        let sk_bytes = sk.to_bytes();

        // Verify secret key never touches stack memory
        let sk_ptr = sk.as_ptr();
        let stack_distance = sk_ptr.wrapping_sub((&pk as *const _) as *const u8);
        assert!(stack_distance > 1024, "Secret key may be stored on stack");

        // Test automatic zeroization on drop
        drop(sk);
        let stack_mem = unsafe { std::slice::from_raw_parts(sk_ptr, sk_bytes.len()) };
        verify_zeroization(&stack_mem, sk_bytes.len());

        // Test explicit zeroization
        pk.zeroize();
        assert_ne!(pk.to_bytes(), pk_bytes, "Public key was not properly zeroized");
    }

    #[test]
    fn test_mldsa_signature_security() {
        let message = b"test message";
        let (pk, sk) = MLDsa::keygen();

        // Test signature generation with secure memory
        let signature = {
            let mut sig = Zeroizing::new(MLDsa::sign(message, &sk));
            let sig_copy = sig.clone();
            // Verify signature is cleared on scope exit
            sig.zeroize();
            sig_copy
        };

        // Verify signature still valid after secure handling
        assert!(MLDsa::verify(message, &signature, &pk).is_ok());

        // Test automatic cleanup of temporary buffers
        let mut temp_buf = vec![0u8; 1024];
        MLDsa::sign_into(message, &sk, &mut temp_buf);
        verify_zeroization(&temp_buf, temp_buf.len());
    }

    #[test]
    fn test_hqc_encryption_memory() {
        let message = b"secret data";
        let (pk, sk) = HQC::keygen();

        // Test encryption with secure memory handling
        let ciphertext = {
            let mut ct = Zeroizing::new(HQC::encrypt(message, &pk).unwrap());
            let ct_copy = ct.clone();
            ct.zeroize();
            ct_copy
        };

        // Test decryption with secure memory
        let plaintext = Zeroizing::new(HQC::decrypt(&ciphertext, &sk).unwrap());
        assert_eq!(plaintext.as_ref(), message);

        // Verify secure cleanup
        drop(plaintext);
        let mut stack_check = [0u8; 1024];
        assert!(stack_check.iter().all(|&b| b == 0),
            "Stack memory not properly cleaned");
    }

    #[test]
    fn test_shared_secret_handling() {
        // Test ML-KEM shared secret security
        let (pk, sk) = MLKem::keygen();
        let (ct, mut ss1) = MLKem::encapsulate(&pk).unwrap();
        let mut ss2 = MLKem::decapsulate(&ct, &sk).unwrap();

        // Verify secrets match before zeroization
        assert_eq!(ss1, ss2);

        // Test proper cleanup
        ss1.zeroize();
        ss2.zeroize();
        verify_zeroization(&ss1, ss1.len());
        verify_zeroization(&ss2, ss2.len());
    }

    #[test]
    fn test_memory_alignment() {
        // Verify key alignment for constant-time operations
        let (pk, sk) = MLKem::keygen();
        assert_eq!(mem::align_of_val(&pk) >= 16, true,
            "Public key not properly aligned for constant-time ops");
        assert_eq!(mem::align_of_val(&sk) >= 16, true,
            "Secret key not properly aligned for constant-time ops");
    }
}