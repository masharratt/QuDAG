use criterion::black_box;
use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};
use test_utils::timing::*;

/// Test suite for constant-time operations validation
#[cfg(test)]
mod constant_time_tests {
    use super::*;

    #[test]
    fn test_mlkem_keygen_constant_time() {
        let iterations = 1000;
        let time_variance = measure_time_variance(|| {
            let (pk, sk) = MLKem::keygen();
            black_box((pk, sk));
        }, iterations);
        
        assert!(time_variance < TIMING_THRESHOLD, 
            "ML-KEM key generation showed timing variation above threshold");
    }

    #[test]
    fn test_mldsa_sign_constant_time() {
        let iterations = 1000;
        let message = b"test message";
        let (pk, sk) = MLDsa::keygen();
        
        let time_variance = measure_time_variance(|| {
            let signature = MLDsa::sign(message, &sk);
            black_box(signature);
        }, iterations);
        
        assert!(time_variance < TIMING_THRESHOLD,
            "ML-DSA signing showed timing variation above threshold");
    }

    #[test]
    fn test_hqc_encrypt_constant_time() {
        let iterations = 1000;
        let message = b"test message";
        let (pk, _) = HQC::keygen();
        
        let time_variance = measure_time_variance(|| {
            let ciphertext = HQC::encrypt(message, &pk);
            black_box(ciphertext);
        }, iterations);
        
        assert!(time_variance < TIMING_THRESHOLD,
            "HQC encryption showed timing variation above threshold");
    }
}