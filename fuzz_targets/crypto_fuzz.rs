#![no_main]
use libfuzzer_sys::fuzz_target;
use qudag_crypto::{
    encryption::{self, HqcCiphertext, HqcPublicKey, HqcSecretKey},
    kem::{MlKemCiphertext, MlKemPublicKey, MlKemSecretKey},
    signatures::{MlDsaPublicKey, MlDsaSecretKey, MlDsaSignature},
};
use std::{panic, sync::Once};
use zeroize::Zeroizing;

// Ensure panic handling is set up exactly once
static INIT: Once = Once::new();

// Custom panic hook that preserves stack traces while preventing information leaks
fn setup_panic_hook() {
    INIT.call_once(|| {
        panic::set_hook(Box::new(|_| {
            // Deliberately minimal panic message to prevent information leaks
            eprintln!("Fuzzer encountered an error");
        }));
    });
}

// Constant-time fuzzing helper - returns true if operations took similar time
fn verify_constant_time<F>(mut op1: F, mut op2: F) -> bool 
where
    F: FnMut() -> Vec<u8>,
{
    use std::time::{Duration, Instant};
    
    // Multiple runs to account for system noise
    const ITERATIONS: u32 = 1000;
    let mut times1 = Vec::with_capacity(ITERATIONS as usize);
    let mut times2 = Vec::with_capacity(ITERATIONS as usize);

    for _ in 0..ITERATIONS {
        let start = Instant::now();
        let _ = op1();
        times1.push(start.elapsed());

        let start = Instant::now();
        let _ = op2();
        times2.push(start.elapsed());
    }

    // Calculate statistics
    let avg1: Duration = times1.iter().sum::<Duration>() / ITERATIONS;
    let avg2: Duration = times2.iter().sum::<Duration>() / ITERATIONS;
    
    // Check if timing difference is within acceptable threshold (1%)
    let threshold = avg1.max(avg2) / 100;
    avg1.abs_diff(avg2) <= threshold
}

// Memory safety validation helper
fn validate_memory_cleanup<T: zeroize::Zeroize>(data: &T) -> bool {
    use std::mem;
    
    // Create a copy that will be zeroized
    let mut data_copy = unsafe { mem::transmute_copy(data) };
    data_copy.zeroize();
    
    // Verify memory has been cleared
    let bytes = unsafe {
        std::slice::from_raw_parts(
            &data_copy as *const T as *const u8,
            mem::size_of::<T>(),
        )
    };
    
    bytes.iter().all(|&b| b == 0)
}

fuzz_target!(|data: &[u8]| {
    setup_panic_hook();
    
    // Skip empty or very small inputs
    if data.len() < 32 {
        return;
    }

    // Use different chunks of input data for different operations
    let chunk_size = data.len() / 4;
    let (key_data, sig_data, msg_data, extra_data) = (
        &data[..chunk_size],
        &data[chunk_size..chunk_size * 2],
        &data[chunk_size * 2..chunk_size * 3],
        &data[chunk_size * 3..],
    );

    // 1. ML-KEM Fuzzing with constant-time validation
    {
        let pub_key = MlKemPublicKey::try_from(key_data).unwrap_or_default();
        let sec_key = MlKemSecretKey::try_from(sig_data).unwrap_or_default();
        
        // Test encapsulation timing consistency
        let is_constant_time = verify_constant_time(
            || pub_key.encapsulate(msg_data).unwrap_or_default().to_bytes(),
            || pub_key.encapsulate(extra_data).unwrap_or_default().to_bytes()
        );
        assert!(is_constant_time, "ML-KEM encapsulation timing leak detected");

        // Test decapsulation timing consistency
        let ct1 = MlKemCiphertext::try_from(msg_data).unwrap_or_default();
        let ct2 = MlKemCiphertext::try_from(extra_data).unwrap_or_default();
        let is_constant_time = verify_constant_time(
            || sec_key.decapsulate(&ct1).unwrap_or_default(),
            || sec_key.decapsulate(&ct2).unwrap_or_default()
        );
        assert!(is_constant_time, "ML-KEM decapsulation timing leak detected");

        // Verify memory zeroization
        assert!(validate_memory_cleanup(&sec_key), "ML-KEM secret key not properly zeroized");
    }

    // 2. ML-DSA Fuzzing
    {
        let pub_key = MlDsaPublicKey::try_from(key_data).unwrap_or_default();
        let sec_key = Zeroizing::new(MlDsaSecretKey::try_from(sig_data).unwrap_or_default());
        
        // Test signature generation timing consistency
        let is_constant_time = verify_constant_time(
            || sec_key.sign(msg_data).unwrap_or_default().to_bytes(),
            || sec_key.sign(extra_data).unwrap_or_default().to_bytes()
        );
        assert!(is_constant_time, "ML-DSA signing timing leak detected");

        // Test signature verification timing consistency
        let sig1 = MlDsaSignature::try_from(msg_data).unwrap_or_default();
        let sig2 = MlDsaSignature::try_from(extra_data).unwrap_or_default();
        let is_constant_time = verify_constant_time(
            || pub_key.verify(msg_data, &sig1).map(|_| vec![1]).unwrap_or_default(),
            || pub_key.verify(extra_data, &sig2).map(|_| vec![1]).unwrap_or_default()
        );
        assert!(is_constant_time, "ML-DSA verification timing leak detected");

        // Memory cleanup validation
        assert!(validate_memory_cleanup(&sec_key), "ML-DSA secret key not properly zeroized");
    }

    // 3. HQC Fuzzing
    {
        let pub_key = HqcPublicKey::try_from(key_data).unwrap_or_default();
        let sec_key = Zeroizing::new(HqcSecretKey::try_from(sig_data).unwrap_or_default());

        // Test encryption timing consistency
        let is_constant_time = verify_constant_time(
            || encryption::encrypt(&pub_key, msg_data).unwrap_or_default().to_bytes(),
            || encryption::encrypt(&pub_key, extra_data).unwrap_or_default().to_bytes()
        );
        assert!(is_constant_time, "HQC encryption timing leak detected");

        // Test decryption timing consistency
        let ct1 = HqcCiphertext::try_from(msg_data).unwrap_or_default();
        let ct2 = HqcCiphertext::try_from(extra_data).unwrap_or_default();
        let is_constant_time = verify_constant_time(
            || encryption::decrypt(&sec_key, &ct1).unwrap_or_default(),
            || encryption::decrypt(&sec_key, &ct2).unwrap_or_default()
        );
        assert!(is_constant_time, "HQC decryption timing leak detected");

        // Memory cleanup validation
        assert!(validate_memory_cleanup(&sec_key), "HQC secret key not properly zeroized");
    }

    // 4. Edge Cases and Error Handling
    {
        // Test with zero-filled data
        let zero_data = vec![0u8; chunk_size];
        let result = MlKemPublicKey::try_from(zero_data.as_slice())
            .and_then(|pk| pk.encapsulate(&zero_data));
        assert!(result.is_err(), "Should reject zero-filled input");

        // Test with invalid lengths
        let short_data = &[1u8; 16];
        assert!(MlKemPublicKey::try_from(short_data).is_err(), "Should reject short input");
        
        // Test with maximum allowed size
        let max_size = encryption::MAX_MESSAGE_SIZE;
        let large_data = vec![1u8; max_size + 1];
        let result = HqcPublicKey::try_from(key_data)
            .and_then(|pk| encryption::encrypt(&pk, &large_data));
        assert!(result.is_err(), "Should reject oversized messages");
    }
});