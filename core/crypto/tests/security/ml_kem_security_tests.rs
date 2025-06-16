use qudag_crypto::kem::{KEMError, KeyEncapsulation};
use qudag_crypto::ml_kem::MlKem768;
use rand::RngCore;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;

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
    
    // Test encapsulation timing consistency
    let (pk, _) = MlKem768::keygen().unwrap();
    timings.clear();
    
    for _ in 0..100 {
        let start = Instant::now();
        let _ = MlKem768::encapsulate(&pk).unwrap();
        timings.push(start.elapsed());
    }
    
    let mean = timings.iter().sum::<Duration>() / timings.len() as u32;
    let variance: f64 = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean.as_nanos() as f64;
            diff * diff
        })
        .sum::<f64>() / timings.len() as f64;
    let std_dev = (variance as f64).sqrt();
    
    assert!(std_dev / mean.as_nanos() as f64 < 0.1, "Encapsulation timing variation too high");
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
    let err1 = MlKem768::decapsulate(&sk, &MlKem768::Ciphertext::from_bytes(&short_ct).unwrap()).unwrap_err();
    
    // Test with extended ciphertext
    let mut long_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE + 1];
    rand::thread_rng().fill_bytes(&mut long_ct);
    let err2 = MlKem768::decapsulate(&sk, &MlKem768::Ciphertext::from_bytes(&long_ct[..MlKem768::CIPHERTEXT_SIZE]).unwrap()).unwrap_err();
    
    // Test with invalid key
    let mut invalid_sk = sk.as_ref().to_vec();
    invalid_sk[0] ^= 0xFF;
    let err3 = MlKem768::decapsulate(
        &MlKem768::SecretKey::from_bytes(&invalid_sk).unwrap(),
        &MlKem768::Ciphertext::from_bytes(&long_ct[..MlKem768::CIPHERTEXT_SIZE]).unwrap()
    ).unwrap_err();
    
    // Verify all error messages reveal the same information
    let err1_str = format!("{:?}", err1);
    let err2_str = format!("{:?}", err2);
    let err3_str = format!("{:?}", err3);
    
    assert_eq!(err1_str, err2_str, "Error messages should not leak length information");
    assert_eq!(err2_str, err3_str, "Error messages should not leak key validity information");
}

#[test]
fn test_key_cache_overflow() {
    // Generate more keys than cache size
    let mut keys = Vec::new();
    for _ in 0..MlKem768::CACHE_SIZE + 10 {
        keys.push(MlKem768::keygen().unwrap());
    }
    
    // Use each key once to fill cache
    for (pk, sk) in &keys {
        let (ct, _) = MlKem768::encapsulate(pk).unwrap();
        let _ = MlKem768::decapsulate(sk, &ct).unwrap();
    }
    
    let metrics = MlKem768::get_metrics();
    assert!(metrics.key_cache_misses >= MlKem768::CACHE_SIZE as u64);
    
    // Use first key again - should be evicted
    let (pk, sk) = &keys[0];
    let (ct, _) = MlKem768::encapsulate(pk).unwrap();
    let _ = MlKem768::decapsulate(sk, &ct).unwrap();
    
    let new_metrics = MlKem768::get_metrics();
    assert!(new_metrics.key_cache_misses > metrics.key_cache_misses);
}

#[test]
fn test_shared_secret_uniqueness() {
    let (pk, sk) = MlKem768::keygen().unwrap();
    let mut secrets = Vec::new();
    
    // Generate multiple shared secrets
    for _ in 0..100 {
        let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
        let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
        
        // Verify each pair matches
        assert_eq!(ss1, ss2);
        
        // Store for uniqueness check
        secrets.push(ss1);
    }
    
    // Verify all secrets are unique
    for i in 0..secrets.len() {
        for j in (i + 1)..secrets.len() {
            assert_ne!(secrets[i], secrets[j], "Found duplicate shared secret");
        }
    }
}

#[test]
fn test_mlkem_constant_time() {
    let (pk, sk) = MlKem768::keygen().unwrap();
    let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
    
    // Test decapsulation timing consistency
    let mut timings_valid = Vec::new();
    let mut timings_invalid = Vec::new();
    
    let mut invalid_ct = ct.as_ref().to_vec();
    invalid_ct[0] ^= 0xFF; // Flip bits in first byte
    let invalid_ct = MlKem768::Ciphertext::from_bytes(&invalid_ct).unwrap();
    
    for _ in 0..100 {
        let start = Instant::now();
        let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
        timings_valid.push(start.elapsed().as_nanos());
        
        let start = Instant::now();
        let _ = MlKem768::decapsulate(&sk, &invalid_ct);
        timings_invalid.push(start.elapsed().as_nanos());
    }
    
    // Calculate statistics
    let mean_valid = timings_valid.iter().sum::<u128>() as f64 / timings_valid.len() as f64;
    let mean_invalid = timings_invalid.iter().sum::<u128>() as f64 / timings_invalid.len() as f64;
    
    let time_diff = (mean_valid - mean_invalid).abs();
    let avg_time = (mean_valid + mean_invalid) / 2.0;
    
    // Verify timing difference is within 5%
    assert!(
        time_diff / avg_time < 0.05,
        "Timing difference too large: {:.2}% ({} vs {})",
        (time_diff / avg_time) * 100.0,
        mean_valid,
        mean_invalid
    );
    
    // Test constant-time comparison operations
    let (pk2, _) = MlKem768::keygen().unwrap();
    
    let start = Instant::now();
    let _ = pk.as_ref().ct_eq(pk.as_ref());
    let equal_time = start.elapsed();
    
    let start = Instant::now();
    let _ = pk.as_ref().ct_eq(pk2.as_ref());
    let not_equal_time = start.elapsed();
    
    let time_diff = (equal_time.as_nanos() as f64 - not_equal_time.as_nanos() as f64).abs();
    let avg_time = (equal_time.as_nanos() + not_equal_time.as_nanos()) as f64 / 2.0;
    
    assert!(
        time_diff / avg_time < 0.05,
        "Comparison timing difference too large: {:.2}%",
        (time_diff / avg_time) * 100.0
    );
}