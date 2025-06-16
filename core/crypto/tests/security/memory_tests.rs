use qudag_crypto::{kem::MLKem, signatures::MLDsa, encryption::HQC};
use zeroize::{Zeroize, Zeroizing};
use std::{mem, sync::atomic::{AtomicU8, Ordering}, alloc::{Layout, alloc, dealloc}};
use proptest::prelude::*;
use std::time::Instant;

/// Memory security test suite for cryptographic operations
#[cfg(test)]
mod memory_security_tests {
    use super::*;

    /// Helper to verify memory patterns and zeroization
    fn verify_memory_patterns<T: AsRef<[u8]>>(data: &T, expected_zeros: usize) {
        let bytes = data.as_ref();
        
        // Check complete zeroization
        let zero_count = bytes.iter().filter(|&&b| b == 0).count();
        assert_eq!(zero_count, expected_zeros, 
            "Memory not properly zeroized - found {} zeros, expected {}", 
            zero_count, expected_zeros);

        // Check for residual patterns
        let ones_count = bytes.iter().filter(|&&b| b == 0xff).count();
        assert_eq!(ones_count, 0, "Found residual pattern of ones");

        // Check for repeating sequences
        for window in bytes.windows(4) {
            assert_ne!(window.iter().all(|&b| b == window[0]), true,
                "Found repeating byte pattern");
        }
    }

    /// Helper for aligned memory allocation
    fn allocate_aligned_buffer(size: usize, align: usize) -> (*mut u8, Layout) {
        let layout = Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { alloc(layout) };
        (ptr, layout)
    }

    /// Helper to measure operation timing
    fn measure_constant_time<F>(op: F, iterations: usize) -> bool 
    where
        F: Fn() -> ()
    {
        let mut timings = Vec::with_capacity(iterations);
        
        for _ in 0..iterations {
            let start = Instant::now();
            op();
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
        variance < 1000
    }

    #[test]
    fn test_mlkem_key_lifecycle() {
        // Allocate aligned buffers
        let (pk_ptr, pk_layout) = allocate_aligned_buffer(MLKem::PUBLIC_KEY_SIZE, 32);
        let (sk_ptr, sk_layout) = allocate_aligned_buffer(MLKem::SECRET_KEY_SIZE, 32);

        // Test key generation with aligned memory
        let (mut pk, mut sk) = unsafe {
            let pk_slice = std::slice::from_raw_parts_mut(pk_ptr, MLKem::PUBLIC_KEY_SIZE);
            let sk_slice = std::slice::from_raw_parts_mut(sk_ptr, MLKem::SECRET_KEY_SIZE);
            MLKem::keygen_into(pk_slice, sk_slice).unwrap()
        };

        // Verify key alignment
        assert_eq!(pk.as_ptr() as usize % 32, 0, "Public key not 32-byte aligned");
        assert_eq!(sk.as_ptr() as usize % 32, 0, "Secret key not 32-byte aligned");

        // Verify keys are on different pages
        let pk_page = pk.as_ptr() as usize / 4096;
        let sk_page = sk.as_ptr() as usize / 4096;
        assert_ne!(pk_page, sk_page, "Keys must be on different memory pages");

        // Test constant-time operations
        let is_constant = measure_constant_time(|| {
            let _ = MLKem::decapsulate(&[0u8; 32], &sk);
        }, 100);
        assert!(is_constant, "Key operations not constant-time");

        // Test zeroization
        sk.zeroize();
        verify_memory_patterns(&sk, sk.len());

        // Clean up aligned buffers
        unsafe {
            dealloc(pk_ptr, pk_layout);
            dealloc(sk_ptr, sk_layout);
        }
    }

    #[test]
    fn test_mldsa_signature_security() {
        // Test with various message sizes
        proptest!(|(message in prop::collection::vec(any::<u8>(), 1..1024))| {
            let (pk, mut sk) = MLDsa::keygen();

            // Test signature generation with secure memory
            let signature = {
                let mut sig = Zeroizing::new(MLDsa::sign(&message, &sk));
                
                // Add memory fence to ensure operation ordering
                std::sync::atomic::fence(Ordering::SeqCst);
                
                let sig_copy = sig.clone();
                sig.zeroize();
                verify_memory_patterns(&sig, sig.len());
                sig_copy
            };

            // Verify signature remains valid
            assert!(MLDsa::verify(&message, &signature, &pk).is_ok());

            // Test cleanup of temporary buffers
            let mut temp_buf = vec![0u8; 1024];
            MLDsa::sign_into(&message, &sk, &mut temp_buf);
            verify_memory_patterns(&temp_buf, temp_buf.len());

            // Ensure secret key cleanup
            sk.zeroize();
            verify_memory_patterns(&sk, sk.len());
        });
    }

    #[test]
    fn test_hqc_encryption_memory() {
        // Test with various message sizes
        proptest!(|(message in prop::collection::vec(any::<u8>(), 1..1024))| {
            let (pk, mut sk) = HQC::keygen();

            // Test encryption with secure memory
            let ciphertext = {
                let mut ct = Zeroizing::new(HQC::encrypt(&message, &pk).unwrap());
                
                // Memory fence to ensure cleanup ordering
                std::sync::atomic::fence(Ordering::SeqCst);
                
                let ct_copy = ct.clone();
                ct.zeroize();
                verify_memory_patterns(&ct, ct.len());
                ct_copy
            };

            // Test decryption with secure memory
            let plaintext = {
                let mut pt = Zeroizing::new(HQC::decrypt(&ciphertext, &sk).unwrap());
                assert_eq!(pt.as_ref(), &message);
                
                // Memory fence before cleanup
                std::sync::atomic::fence(Ordering::SeqCst);
                
                pt.zeroize();
                verify_memory_patterns(&pt, pt.len());
                pt
            };

            // Verify secret key cleanup
            sk.zeroize();
            verify_memory_patterns(&sk, sk.len());
        });
    }

    #[test]
    fn test_shared_secret_handling() {
        // Test with multiple key pairs
        for _ in 0..10 {
            let (pk, sk) = MLKem::keygen();
            
            // Test encapsulation
            let (ct, mut ss1) = MLKem::encapsulate(&pk).unwrap();
            
            // Test constant-time decapsulation
            let is_constant = measure_constant_time(|| {
                let _ = MLKem::decapsulate(&ct, &sk);
            }, 100);
            assert!(is_constant, "Decapsulation not constant-time");

            let mut ss2 = MLKem::decapsulate(&ct, &sk).unwrap();

            // Verify secrets match
            assert_eq!(ss1, ss2);

            // Test cleanup with memory fences
            std::sync::atomic::fence(Ordering::SeqCst);
            ss1.zeroize();
            verify_memory_patterns(&ss1, ss1.len());

            std::sync::atomic::fence(Ordering::SeqCst);
            ss2.zeroize();
            verify_memory_patterns(&ss2, ss2.len());
        }
    }

    #[test]
    fn test_memory_alignment() {
        // Test alignment for different key sizes
        proptest!(|(size in 16usize..4096)| {
            let (ptr, layout) = allocate_aligned_buffer(size, 32);
            
            // Verify alignment
            assert_eq!(ptr as usize % 32, 0, 
                "Buffer not 32-byte aligned");

            // Test constant-time operations
            let slice = unsafe { std::slice::from_raw_parts_mut(ptr, size) };
            let is_constant = measure_constant_time(|| {
                for i in 0..size {
                    // Use atomic operations to prevent optimization
                    let _ = AtomicU8::new(slice[i]).load(Ordering::SeqCst);
                }
            }, 100);
            assert!(is_constant, "Memory access not constant-time");

            // Clean up
            unsafe { dealloc(ptr, layout); }
        });
    }
}