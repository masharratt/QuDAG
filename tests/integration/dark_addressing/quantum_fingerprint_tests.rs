use qudag_crypto::fingerprint::{Fingerprint, FingerprintError};
use qudag_crypto::ml_dsa::{MlDsaPublicKey, MlDsaKeyPair};
use rand::rngs::OsRng;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_quantum_fingerprint_generation_and_verification() {
    let mut rng = OsRng;
    
    // Test data
    let test_data = b"This is test data for quantum fingerprint";
    
    // Generate fingerprint
    let (fingerprint, public_key) = Fingerprint::generate(test_data, &mut rng).unwrap();
    
    // Verify fingerprint with correct public key
    assert!(fingerprint.verify(&public_key).is_ok());
    
    // Verify fingerprint data is deterministic for same input
    let (fingerprint2, _) = Fingerprint::generate(test_data, &mut rng).unwrap();
    assert_eq!(fingerprint.data(), fingerprint2.data());
    
    // But signatures should be different
    assert_ne!(fingerprint.signature(), fingerprint2.signature());
}

#[tokio::test]
async fn test_quantum_fingerprint_invalid_verification() {
    let mut rng = OsRng;
    
    // Generate two different fingerprints
    let (fingerprint1, public_key1) = Fingerprint::generate(b"Data 1", &mut rng).unwrap();
    let (fingerprint2, public_key2) = Fingerprint::generate(b"Data 2", &mut rng).unwrap();
    
    // Cross-verification should fail
    assert!(fingerprint1.verify(&public_key2).is_err());
    assert!(fingerprint2.verify(&public_key1).is_err());
}

#[tokio::test]
async fn test_quantum_fingerprint_with_large_data() {
    let mut rng = OsRng;
    
    // Generate large test data
    let large_data: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
    
    // Generate fingerprint for large data
    let (fingerprint, public_key) = Fingerprint::generate(&large_data, &mut rng).unwrap();
    
    // Verify it works with large data
    assert!(fingerprint.verify(&public_key).is_ok());
    
    // Fingerprint size should be constant regardless of input size
    assert_eq!(fingerprint.data().len(), 64);
}

#[tokio::test]
async fn test_quantum_fingerprint_empty_data() {
    let mut rng = OsRng;
    
    // Generate fingerprint for empty data
    let (fingerprint, public_key) = Fingerprint::generate(b"", &mut rng).unwrap();
    
    // Should still generate valid fingerprint
    assert!(fingerprint.verify(&public_key).is_ok());
    assert_eq!(fingerprint.data().len(), 64);
}

#[tokio::test]
async fn test_concurrent_quantum_fingerprint_operations() {
    let fingerprints = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];
    
    // Spawn multiple tasks to generate fingerprints concurrently
    for i in 0..10 {
        let fingerprints_clone = fingerprints.clone();
        
        let handle = tokio::spawn(async move {
            let mut rng = OsRng;
            let data = format!("Concurrent test data {}", i);
            
            let (fingerprint, public_key) = Fingerprint::generate(data.as_bytes(), &mut rng).unwrap();
            
            // Verify immediately
            assert!(fingerprint.verify(&public_key).is_ok());
            
            // Store for later verification
            fingerprints_clone.lock().await.push((fingerprint, public_key));
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all fingerprints again
    let fingerprints = fingerprints.lock().await;
    assert_eq!(fingerprints.len(), 10);
    
    for (fingerprint, public_key) in fingerprints.iter() {
        assert!(fingerprint.verify(public_key).is_ok());
    }
}

#[tokio::test]
async fn test_quantum_fingerprint_collision_resistance() {
    let mut rng = OsRng;
    let mut fingerprints = Vec::new();
    
    // Generate fingerprints for similar data
    for i in 0..100 {
        let data = format!("Test data with slight variation: {}", i);
        let (fingerprint, _) = Fingerprint::generate(data.as_bytes(), &mut rng).unwrap();
        fingerprints.push(fingerprint.data().to_vec());
    }
    
    // Check that all fingerprints are unique
    for i in 0..fingerprints.len() {
        for j in i+1..fingerprints.len() {
            assert_ne!(
                fingerprints[i], 
                fingerprints[j],
                "Collision detected between fingerprints {} and {}", 
                i, j
            );
        }
    }
}

#[tokio::test]
async fn test_quantum_fingerprint_key_rotation() {
    let mut rng = OsRng;
    let test_data = b"Key rotation test data";
    
    // Generate multiple fingerprints with different keys
    let mut fingerprints_and_keys = Vec::new();
    
    for _ in 0..5 {
        let (fingerprint, public_key) = Fingerprint::generate(test_data, &mut rng).unwrap();
        fingerprints_and_keys.push((fingerprint, public_key));
    }
    
    // Verify each fingerprint with its corresponding key
    for (fingerprint, public_key) in &fingerprints_and_keys {
        assert!(fingerprint.verify(public_key).is_ok());
    }
    
    // Verify fingerprints cannot be verified with wrong keys
    for i in 0..fingerprints_and_keys.len() {
        for j in 0..fingerprints_and_keys.len() {
            if i != j {
                let (fingerprint, _) = &fingerprints_and_keys[i];
                let (_, wrong_key) = &fingerprints_and_keys[j];
                assert!(fingerprint.verify(wrong_key).is_err());
            }
        }
    }
}

#[tokio::test]
async fn test_quantum_fingerprint_deterministic_data() {
    let mut rng = OsRng;
    
    // Same input should produce same fingerprint data
    let test_data = b"Deterministic test";
    let mut data_results = Vec::new();
    
    for _ in 0..10 {
        let (fingerprint, _) = Fingerprint::generate(test_data, &mut rng).unwrap();
        data_results.push(fingerprint.data().to_vec());
    }
    
    // All fingerprint data should be identical
    for i in 1..data_results.len() {
        assert_eq!(data_results[0], data_results[i]);
    }
}

#[tokio::test]
async fn test_quantum_fingerprint_bit_flipping() {
    let mut rng = OsRng;
    
    // Generate fingerprint
    let original_data = b"Bit flipping test";
    let (fingerprint, public_key) = Fingerprint::generate(original_data, &mut rng).unwrap();
    
    // Flip one bit in the data
    let mut modified_data = original_data.to_vec();
    modified_data[0] ^= 1;
    
    let (modified_fingerprint, _) = Fingerprint::generate(&modified_data, &mut rng).unwrap();
    
    // Fingerprints should be completely different
    assert_ne!(fingerprint.data(), modified_fingerprint.data());
    
    // Verify avalanche effect - many bits should change
    let original_bits = fingerprint.data();
    let modified_bits = modified_fingerprint.data();
    
    let mut bit_differences = 0;
    for i in 0..original_bits.len() {
        bit_differences += (original_bits[i] ^ modified_bits[i]).count_ones() as usize;
    }
    
    // Expect roughly 50% of bits to be different (avalanche effect)
    let total_bits = original_bits.len() * 8;
    assert!(bit_differences > total_bits / 3); // At least 1/3 of bits should differ
}

#[tokio::test]
async fn test_quantum_fingerprint_serialization() {
    let mut rng = OsRng;
    
    // Generate fingerprint
    let test_data = b"Serialization test";
    let (fingerprint, public_key) = Fingerprint::generate(test_data, &mut rng).unwrap();
    
    // Store fingerprint data and signature
    let fingerprint_data = fingerprint.data().to_vec();
    let fingerprint_signature = fingerprint.signature().to_vec();
    
    // Simulate deserialization by creating a new fingerprint with stored data
    // Note: In real implementation, we'd need proper serialization methods
    
    // Verify the stored data can still be verified
    assert!(public_key.verify(&fingerprint_data, &fingerprint_signature).is_ok());
}

#[tokio::test] 
async fn test_quantum_fingerprint_timing_consistency() {
    use std::time::Instant;
    let mut rng = OsRng;
    
    // Test that fingerprint generation time is consistent
    let mut generation_times = Vec::new();
    
    for i in 0..20 {
        let data = vec![0u8; 1000 * (i + 1)]; // Varying data sizes
        
        let start = Instant::now();
        let _ = Fingerprint::generate(&data, &mut rng).unwrap();
        let duration = start.elapsed();
        
        generation_times.push(duration.as_micros());
    }
    
    // Check that times don't vary too much (constant-time property)
    let avg_time: u128 = generation_times.iter().sum::<u128>() / generation_times.len() as u128;
    
    for time in &generation_times {
        let deviation = if *time > avg_time { 
            *time - avg_time 
        } else { 
            avg_time - *time 
        };
        
        // Allow up to 50% deviation
        assert!(
            deviation < avg_time / 2,
            "Timing deviation too high: {} vs avg {}",
            time,
            avg_time
        );
    }
}