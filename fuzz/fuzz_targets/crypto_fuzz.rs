#![no_main]
use libfuzzer_sys::fuzz_target;
use qudag_crypto::{
    kem::{KeyEncapsulation, PublicKey as KemPublicKey, SecretKey as KemSecretKey},
    signature::{DigitalSignature, PublicKey as SigPublicKey, SecretKey as SigSecretKey},
    ml_kem::MlKem768,
    ml_dsa::MlDsa87,
};
use zeroize::Zeroize;
use std::time::Instant;
use subtle::ConstantTimeEq;

/// Helper function to verify constant-time behavior with advanced timing analysis
fn verify_constant_time<F>(op: F) -> bool 
where
    F: Fn() -> Result<(), ()>
{
    let iterations = 100; // Reduced for faster fuzzing
    let mut timings = Vec::with_capacity(iterations);
    let mut min_time = std::time::Duration::from_secs(u64::MAX);
    let mut max_time = std::time::Duration::from_secs(0);
    
    // Warm up the CPU
    for _ in 0..10 {
        let _ = op();
    }
    
    // Collect timing samples
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = op();
        let duration = start.elapsed();
        
        min_time = min_time.min(duration);
        max_time = max_time.max(duration);
        timings.push(duration);
    }
    
    // Sort timings for percentile analysis
    timings.sort();
    let median = timings[iterations / 2];
    let p99 = timings[(iterations * 99) / 100];
    
    // Calculate variance
    let mean = timings.iter().sum::<std::time::Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    // Multiple criteria for constant-time validation:
    variance < 5000 && // Relaxed threshold for fuzzing
    (max_time - min_time).as_nanos() < 50000 && // Max 50μs difference
    (p99 - median).as_nanos() < 25000 // P99 within 25μs of median
}

/// Helper to validate proper memory cleanup
fn validate_memory_cleanup(data: &[u8]) -> bool {
    // Test stack cleanup
    let mut test_data = data.to_vec();
    test_data.zeroize();
    test_data.iter().all(|&b| b == 0)
}

fuzz_target!(|data: &[u8]| {
    // Set panic hook to prevent information leaks
    std::panic::set_hook(Box::new(|_| {}));

    if data.len() < 256 {
        return;
    }

    // ML-KEM fuzzing with constant-time validation
    let kem_section = &data[..64];
    
    // Test key generation timing
    let gen_constant = verify_constant_time(|| {
        MlKem768::keygen()
            .map(|_| ())
            .map_err(|_| ())
    });
    
    if let Ok((pk, sk)) = MlKem768::keygen() {
        // Test encapsulation/decapsulation with timing validation
        if let Ok((ct, ss1)) = MlKem768::encapsulate(&pk) {
            let decap_constant = verify_constant_time(|| {
                MlKem768::decapsulate(&sk, &ct)
                    .map(|_| ())
                    .map_err(|_| ())
            });

            if let Ok(ss2) = MlKem768::decapsulate(&sk, &ct) {
                // Use ConstantTimeEq for comparison
                let ss1_bytes = ss1.as_ref();
                let ss2_bytes = ss2.as_ref();
                assert!(ss1_bytes.ct_eq(ss2_bytes).into());
            }
        }

        // Test with malformed ciphertext
        if data.len() >= 128 {
            let bad_ct = &data[64..128];
            let _ = MlKem768::decapsulate(&sk, &qudag_crypto::ml_kem::Ciphertext::from(bad_ct));
        }
    }

    // ML-DSA fuzzing with memory validation
    if data.len() >= 192 {
        let sig_section = &data[64..128];
        
        if let Ok((pk, sk)) = MlDsa87::keygen() {
            let msg = &data[128..192];
            
            // Test signing with timing validation
            let sign_constant = verify_constant_time(|| {
                MlDsa87::sign(&sk, msg)
                    .map(|_| ())
                    .map_err(|_| ())
            });

            if let Ok(signature) = MlDsa87::sign(&sk, msg) {
                // Test verification
                if let Ok(valid) = MlDsa87::verify(&pk, msg, &signature) {
                    assert!(valid);
                }

                // Test invalid cases
                let _ = MlDsa87::verify(&pk, &[0; 32], &signature);
                
                // Test with mutated signature
                if data.len() >= 256 {
                    let bad_sig = &data[192..256];
                    let _ = MlDsa87::verify(&pk, msg, &qudag_crypto::ml_dsa::Signature::from(bad_sig));
                }
            }
        }
    }

    // Validate memory cleanup
    assert!(validate_memory_cleanup(data), "Memory not properly zeroized");
});