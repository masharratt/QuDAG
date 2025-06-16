use qudag_crypto::kem::{KemError, KeyEncapsulation};
use qudag_crypto::ml_kem::MlKem768;
use rand::RngCore;
use std::time::{Duration, Instant};

#[test]
fn test_mlkem_timing_consistency() {
    // Test that key generation timing is consistent
    let mut timings = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let _ = MlKem768::keygen().unwrap();
        timings.push(start.elapsed());
    }
    
    // Calculate mean and standard deviation
    let mean = timings.iter().sum::<Duration>() / timings.len() as u32;
    let variance: f64 = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean.as_nanos() as f64;
            diff * diff
        })
        .sum::<f64>() / timings.len() as f64;
    let std_dev = (variance as f64).sqrt();
    
    // Verify timing consistency is within reasonable bounds
    assert!(std_dev / mean.as_nanos() as f64 < 0.1, "Timing variation too high");
}

#[test]
fn test_mlkem_memory_cleanup() {
    // Generate keys
    let (pk, sk) = MlKem768::keygen().unwrap();
    let pk_ptr = pk.as_bytes().as_ptr();
    let sk_ptr = sk.as_bytes().as_ptr();
    
    // Capture key material
    let pk_data = pk.as_bytes().to_vec();
    let sk_data = sk.as_bytes().to_vec();
    
    // Drop keys
    drop(pk);
    drop(sk);
    
    // Verify memory is cleared
    unsafe {
        let mut pk_cleared = true;
        let mut sk_cleared = true;
        
        for i in 0..MlKem768::PUBLIC_KEY_SIZE {
            if *pk_ptr.add(i) != 0 {
                pk_cleared = false;
                break;
            }
        }
        
        for i in 0..MlKem768::SECRET_KEY_SIZE {
            if *sk_ptr.add(i) != 0 {
                sk_cleared = false;
                break;
            }
        }
        
        assert!(pk_cleared, "Public key memory not cleared");
        assert!(sk_cleared, "Secret key memory not cleared");
    }
}

#[test]
fn test_mlkem_error_masking() {
    // Test with various invalid inputs to verify error messages don't leak info
    let (pk, sk) = MlKem768::keygen().unwrap();
    
    // Test with truncated ciphertext
    let mut short_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE - 1];
    rand::thread_rng().fill_bytes(&mut short_ct);
    let err1 = MlKem768::decapsulate(&sk, &short_ct).unwrap_err();
    
    // Test with extended ciphertext
    let mut long_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE + 1];
    rand::thread_rng().fill_bytes(&mut long_ct);
    let err2 = MlKem768::decapsulate(&sk, &long_ct).unwrap_err();
    
    // Verify error messages are identical to avoid leaking information
    assert_eq!(
        format!("{:?}", err1),
        format!("{:?}", err2),
        "Error messages should not leak length information"
    );
}

#[test]
fn test_mlkem_constant_time() {
    let (pk, sk) = MlKem768::keygen().unwrap();
    let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
    
    // Measure timing with valid inputs
    let start = Instant::now();
    let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
    let valid_time = start.elapsed();
    
    // Measure timing with invalid ciphertext
    let mut invalid_ct = ct.as_bytes().to_vec();
    invalid_ct[0] ^= 0xFF; // Flip bits in first byte
    
    let start = Instant::now();
    let _ = MlKem768::decapsulate(&sk, &invalid_ct);
    let invalid_time = start.elapsed();
    
    // Verify timing difference is minimal
    let time_diff = if valid_time > invalid_time {
        valid_time - invalid_time
    } else {
        invalid_time - valid_time
    };
    
    assert!(
        time_diff < Duration::from_micros(100),
        "Timing difference too large: {:?}",
        time_diff
    );
}