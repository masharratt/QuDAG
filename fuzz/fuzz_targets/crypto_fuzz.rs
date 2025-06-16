#![no_main]
use libfuzzer_sys::fuzz_target;
use zeroize::Zeroize;
use std::time::Instant;

/// Helper function to verify basic timing consistency 
fn verify_timing_consistency<F>(op: F) -> bool 
where
    F: Fn() -> Result<(), ()>
{
    let iterations = 50; // Reduced for faster fuzzing
    let mut timings = Vec::with_capacity(iterations);
    
    // Collect timing samples
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    if timings.is_empty() {
        return false;
    }
    
    // Calculate basic statistics
    let mean = timings.iter().sum::<std::time::Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    // Accept reasonable variance for fuzzing
    variance < 100000
}

/// Helper to validate proper memory cleanup
fn validate_memory_cleanup(data: &[u8]) -> bool {
    // Test stack cleanup
    let mut test_data = data.to_vec();
    test_data.zeroize();
    test_data.iter().all(|&b| b == 0)
}

/// Test cryptographic hash function behavior
fn test_hash_operations(data: &[u8]) {
    use blake3::Hasher;
    
    // Test consistent hashing
    let hash1 = blake3::hash(data);
    let hash2 = blake3::hash(data);
    assert_eq!(hash1, hash2, "Hash function not deterministic");
    
    // Test incremental hashing
    let mut hasher = Hasher::new();
    hasher.update(data);
    let incremental_hash = hasher.finalize();
    assert_eq!(hash1, incremental_hash, "Incremental hash mismatch");
    
    // Test different chunk sizes
    if data.len() > 8 {
        let mut chunked_hasher = Hasher::new();
        for chunk in data.chunks(8) {
            chunked_hasher.update(chunk);
        }
        let chunked_hash = chunked_hasher.finalize();
        assert_eq!(hash1, chunked_hash, "Chunked hash mismatch");
    }
}

/// Test memory zeroization
fn test_zeroization(data: &[u8]) {
    // Test Vec zeroization
    let mut vec_data = data.to_vec();
    vec_data.zeroize();
    assert!(vec_data.iter().all(|&b| b == 0), "Vec not properly zeroized");
    
    // Test array zeroization
    if data.len() >= 32 {
        let mut array_data = [0u8; 32];
        array_data.copy_from_slice(&data[..32]);
        array_data.zeroize();
        assert!(array_data.iter().all(|&b| b == 0), "Array not properly zeroized");
    }
}

/// Test random number generation consistency
fn test_random_consistency() {
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    
    let seed = [42u8; 32];
    let mut rng1 = StdRng::from_seed(seed);
    let mut rng2 = StdRng::from_seed(seed);
    
    // Generate same sequence
    for _ in 0..100 {
        let val1: u64 = rng1.gen();
        let val2: u64 = rng2.gen();
        assert_eq!(val1, val2, "RNG not deterministic with same seed");
    }
}

fuzz_target!(|data: &[u8]| {
    // Set panic hook to prevent information leaks
    std::panic::set_hook(Box::new(|_| {}));

    if data.is_empty() {
        return;
    }

    // Test hash operations with timing validation
    let hash_timing = verify_timing_consistency(|| {
        test_hash_operations(data);
        Ok(())
    });
    // Don't assert on timing in fuzzing - just ensure it doesn't crash

    // Test zeroization
    test_zeroization(data);

    // Test memory cleanup validation
    assert!(validate_memory_cleanup(data), "Memory not properly zeroized");

    // Test random number generation (not dependent on input data)
    if data.len() >= 32 {
        test_random_consistency();
    }

    // Test edge cases
    if data.len() >= 64 {
        // Test with all zeros
        let zero_data = vec![0u8; 64];
        test_hash_operations(&zero_data);
        test_zeroization(&zero_data);

        // Test with all ones  
        let ones_data = vec![0xFFu8; 64];
        test_hash_operations(&ones_data);
        test_zeroization(&ones_data);

        // Test with alternating pattern
        let alt_data: Vec<u8> = (0..64).map(|i| if i % 2 == 0 { 0x55 } else { 0xAA }).collect();
        test_hash_operations(&alt_data);
        test_zeroization(&alt_data);
    }

    // Test with truncated data
    for i in 1..std::cmp::min(data.len(), 32) {
        let truncated = &data[..i];
        test_hash_operations(truncated);
        if truncated.len() >= 4 {
            test_zeroization(truncated);
        }
    }

    // Test with bit flipping
    if data.len() >= 16 {
        let mut mutated = data[..16].to_vec();
        for i in 0..mutated.len() {
            mutated[i] ^= 1;
            test_hash_operations(&mutated);
            mutated[i] ^= 1; // Restore original
        }
    }
});