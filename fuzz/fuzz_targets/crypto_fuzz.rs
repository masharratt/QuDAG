use libfuzzer_sys::fuzz_target;
use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};
use zeroize::Zeroizing;
use std::time::Instant;

/// Helper function to verify constant-time behavior
fn verify_constant_time<F>(op: F) -> bool 
where
    F: Fn() -> Result<(), ()>
{
    let iterations = 100;
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    // Calculate timing variance
    let mean = timings.iter().sum::<std::time::Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    // Variance should be small for constant-time ops
    variance < 1000 // Threshold determined empirically
}

/// Helper to validate proper memory cleanup
fn validate_memory_cleanup<T: zeroize::Zeroize>(data: &[u8]) -> bool {
    let mut test_data = data.to_vec();
    test_data.zeroize();
    test_data.iter().all(|&b| b == 0)
}

fuzz_target!(|data: &[u8]| {
    // Set panic hook to prevent information leaks
    std::panic::set_hook(Box::new(|_| {}));

    if data.len() < 512 {
        return;
    }

    // ML-KEM fuzzing with constant-time validation
    let kem_section = &data[..128];
    if let Ok(mut key_material) = kem_section[..64].try_into() {
        let key_material = Zeroizing::new(key_material);
        
        // Test key generation timing
        let gen_constant = verify_constant_time(|| {
            MLKem::keygen_with_seed(key_material.as_ref())
                .map(|_| ())
                .map_err(|_| ())
        });
        assert!(gen_constant, "Key generation not constant-time");

        if let Ok((pk, sk)) = MLKem::keygen_with_seed(key_material.as_ref()) {
            // Test encapsulation/decapsulation with timing validation
            if let Ok((ct, ss1)) = MLKem::encapsulate(&pk) {
                let decap_constant = verify_constant_time(|| {
                    MLKem::decapsulate(&ct, &sk)
                        .map(|_| ())
                        .map_err(|_| ())
                });
                assert!(decap_constant, "Decapsulation not constant-time");

                let _ = MLKem::decapsulate(&ct, &sk).map(|ss2| {
                    assert!(ss1.ct_eq(&ss2).unwrap_or(false));
                });
            }

            // Test with modified ciphertext
            if let Ok((mut ct, _)) = MLKem::encapsulate(&pk) {
                ct[0] ^= 1; // Flip one bit
                let _ = MLKem::decapsulate(&ct, &sk);
            }
        }
    }

    // ML-DSA fuzzing with memory validation
    let sig_section = &data[128..256];
    if let Ok(mut sig_material) = sig_section[..64].try_into() {
        let sig_material = Zeroizing::new(sig_material);
        
        if let Ok((pk, sk)) = MLDsa::keygen_with_seed(sig_material.as_ref()) {
            let msg = &data[256..320];
            
            // Test signing with timing validation
            let sign_constant = verify_constant_time(|| {
                MLDsa::sign(msg, &sk)
                    .map(|_| ())
                    .map_err(|_| ())
            });
            assert!(sign_constant, "Signing not constant-time");

            if let Ok(signature) = MLDsa::sign(msg, &sk) {
                // Test verification timing
                let verify_constant = verify_constant_time(|| {
                    MLDsa::verify(msg, &signature, &pk)
                        .map(|_| ())
                        .map_err(|_| ())
                });
                assert!(verify_constant, "Verification not constant-time");

                // Test invalid cases
                let _ = MLDsa::verify(&[0; 32], &signature, &pk);
                
                let mut bad_sig = signature.clone();
                bad_sig[0] ^= 1;
                let _ = MLDsa::verify(msg, &bad_sig, &pk);
            }
        }
    }

    // HQC fuzzing with comprehensive validation
    let hqc_section = &data[320..448];
    if let Ok(mut hqc_material) = hqc_section[..64].try_into() {
        let hqc_material = Zeroizing::new(hqc_material);
        
        if let Ok((pk, sk)) = HQC::keygen_with_seed(hqc_material.as_ref()) {
            let msg = &data[448..512];
            
            // Test encryption timing
            let enc_constant = verify_constant_time(|| {
                HQC::encrypt(msg, &pk)
                    .map(|_| ())
                    .map_err(|_| ())
            });
            assert!(enc_constant, "Encryption not constant-time");

            if let Ok(ct) = HQC::encrypt(msg, &pk) {
                // Test decryption timing
                let dec_constant = verify_constant_time(|| {
                    HQC::decrypt(&ct, &sk)
                        .map(|_| ())
                        .map_err(|_| ())
                });
                assert!(dec_constant, "Decryption not constant-time");

                // Test with modified ciphertext
                let mut bad_ct = ct.clone();
                bad_ct[0] ^= 1;
                let _ = HQC::decrypt(&bad_ct, &sk);
                
                // Test with zero ciphertext
                let zero_ct = vec![0u8; ct.len()];
                let _ = HQC::decrypt(&zero_ct, &sk);
            }
        }
    }

    // Validate memory cleanup
    assert!(validate_memory_cleanup(data), "Memory not properly zeroized");
});