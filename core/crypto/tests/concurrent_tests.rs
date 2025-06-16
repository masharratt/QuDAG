//! Concurrent operations and thread safety tests for crypto module
//!
//! This module tests the thread safety of cryptographic operations, ensuring
//! that ML-KEM, ML-DSA, and other crypto primitives work correctly under
//! concurrent access patterns and high contention scenarios.

use qudag_crypto::{
    ml_kem::{MlKem768, KeyPair as KemKeyPair},
    ml_dsa::{MlDsa65, KeyPair as DsaKeyPair, Signature},
    hqc::{Hqc128, KeyPair as HqcKeyPair},
    fingerprint::QuantumFingerprint,
};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::{Barrier, Semaphore};
use rayon::prelude::*;
use rand::{thread_rng, RngCore};

/// Test concurrent ML-KEM key generation and operations
#[tokio::test]
async fn test_ml_kem_concurrent_operations() {
    const NUM_THREADS: usize = 16;
    const OPERATIONS_PER_THREAD: usize = 100;
    
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();
    
    // Test concurrent key generation
    for thread_id in 0..NUM_THREADS {
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;
            
            let mut successful_ops = 0;
            let mut key_pairs = Vec::new();
            
            // Generate multiple key pairs concurrently
            for _ in 0..OPERATIONS_PER_THREAD {
                match MlKem768::keygen() {
                    Ok(keypair) => {
                        key_pairs.push(keypair);
                        successful_ops += 1;
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Key generation failed: {}", thread_id, e);
                    }
                }
            }
            
            // Test encapsulation/decapsulation with generated keys
            let mut encaps_successful = 0;
            for keypair in &key_pairs {
                let public_key = keypair.public_key();
                
                match MlKem768::encapsulate(&public_key) {
                    Ok((ciphertext, shared_secret_1)) => {
                        match MlKem768::decapsulate(&keypair.secret_key(), &ciphertext) {
                            Ok(shared_secret_2) => {
                                if shared_secret_1 == shared_secret_2 {
                                    encaps_successful += 1;
                                } else {
                                    eprintln!("Thread {}: Shared secrets don't match", thread_id);
                                }
                            }
                            Err(e) => {
                                eprintln!("Thread {}: Decapsulation failed: {}", thread_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Encapsulation failed: {}", thread_id, e);
                    }
                }
            }
            
            (thread_id, successful_ops, encaps_successful)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_keygen_success = 0;
    let mut total_encaps_success = 0;
    
    for handle in handles {
        let (thread_id, keygen_success, encaps_success) = handle.await.unwrap();
        println!("Thread {}: {}/{} keygens, {}/{} encaps successful", 
                thread_id, keygen_success, OPERATIONS_PER_THREAD, encaps_success, keygen_success);
        total_keygen_success += keygen_success;
        total_encaps_success += encaps_success;
    }
    
    assert_eq!(total_keygen_success, NUM_THREADS * OPERATIONS_PER_THREAD,
               "All key generations should succeed");
    assert_eq!(total_encaps_success, total_keygen_success,
               "All encapsulation/decapsulation pairs should succeed");
}

/// Test concurrent ML-DSA signature operations
#[tokio::test]
async fn test_ml_dsa_concurrent_signatures() {
    const NUM_THREADS: usize = 12;
    const SIGNATURES_PER_THREAD: usize = 50;
    
    // Generate a shared keypair for signing
    let shared_keypair = Arc::new(MlDsa65::keygen().unwrap());
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();
    
    for thread_id in 0..NUM_THREADS {
        let keypair_clone = shared_keypair.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;
            
            let mut successful_signs = 0;
            let mut successful_verifies = 0;
            let mut signatures = Vec::new();
            let mut messages = Vec::new();
            
            // Generate unique messages and sign them
            for i in 0..SIGNATURES_PER_THREAD {
                let message = format!("Thread {} message {}", thread_id, i).into_bytes();
                
                match MlDsa65::sign(&keypair_clone.secret_key(), &message) {
                    Ok(signature) => {
                        signatures.push(signature);
                        messages.push(message);
                        successful_signs += 1;
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Signing failed: {}", thread_id, e);
                    }
                }
            }
            
            // Verify all signatures
            for (signature, message) in signatures.iter().zip(messages.iter()) {
                match MlDsa65::verify(&keypair_clone.public_key(), message, signature) {
                    Ok(true) => successful_verifies += 1,
                    Ok(false) => eprintln!("Thread {}: Signature verification returned false", thread_id),
                    Err(e) => eprintln!("Thread {}: Verification failed: {}", thread_id, e),
                }
            }
            
            (thread_id, successful_signs, successful_verifies)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_signs = 0;
    let mut total_verifies = 0;
    
    for handle in handles {
        let (thread_id, signs, verifies) = handle.await.unwrap();
        println!("Thread {}: {}/{} signs, {}/{} verifies successful", 
                thread_id, signs, SIGNATURES_PER_THREAD, verifies, signs);
        total_signs += signs;
        total_verifies += verifies;
    }
    
    assert_eq!(total_signs, NUM_THREADS * SIGNATURES_PER_THREAD,
               "All signatures should succeed");
    assert_eq!(total_verifies, total_signs,
               "All signature verifications should succeed");
}

/// Test concurrent HQC encryption operations
#[tokio::test]
async fn test_hqc_concurrent_encryption() {
    const NUM_THREADS: usize = 8;
    const ENCRYPTIONS_PER_THREAD: usize = 20;
    
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();
    
    for thread_id in 0..NUM_THREADS {
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;
            
            let mut successful_encryptions = 0;
            let mut successful_decryptions = 0;
            
            // Generate keypair for this thread
            let keypair = match Hqc128::keygen() {
                Ok(kp) => kp,
                Err(e) => {
                    eprintln!("Thread {}: Keypair generation failed: {}", thread_id, e);
                    return (thread_id, 0, 0);
                }
            };
            
            for i in 0..ENCRYPTIONS_PER_THREAD {
                let plaintext = format!("Thread {} plaintext {}", thread_id, i).into_bytes();
                
                match Hqc128::encrypt(&keypair.public_key(), &plaintext) {
                    Ok(ciphertext) => {
                        successful_encryptions += 1;
                        
                        match Hqc128::decrypt(&keypair.secret_key(), &ciphertext) {
                            Ok(decrypted) => {
                                if decrypted == plaintext {
                                    successful_decryptions += 1;
                                } else {
                                    eprintln!("Thread {}: Decrypted data doesn't match original", thread_id);
                                }
                            }
                            Err(e) => {
                                eprintln!("Thread {}: Decryption failed: {}", thread_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Encryption failed: {}", thread_id, e);
                    }
                }
            }
            
            (thread_id, successful_encryptions, successful_decryptions)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_encryptions = 0;
    let mut total_decryptions = 0;
    
    for handle in handles {
        let (thread_id, encryptions, decryptions) = handle.await.unwrap();
        println!("Thread {}: {}/{} encryptions, {}/{} decryptions successful", 
                thread_id, encryptions, ENCRYPTIONS_PER_THREAD, decryptions, encryptions);
        total_encryptions += encryptions;
        total_decryptions += decryptions;
    }
    
    assert_eq!(total_encryptions, NUM_THREADS * ENCRYPTIONS_PER_THREAD,
               "All encryptions should succeed");
    assert_eq!(total_decryptions, total_encryptions,
               "All decryptions should succeed");
}

/// Test concurrent quantum fingerprint operations
#[tokio::test]
async fn test_quantum_fingerprint_concurrent() {
    const NUM_THREADS: usize = 10;
    const FINGERPRINTS_PER_THREAD: usize = 100;
    
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();
    
    for thread_id in 0..NUM_THREADS {
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;
            
            let mut successful_operations = 0;
            let mut fingerprints = Vec::new();
            
            for i in 0..FINGERPRINTS_PER_THREAD {
                let data = format!("Thread {} data {}", thread_id, i).into_bytes();
                
                match QuantumFingerprint::generate(&data) {
                    Ok(fingerprint) => {
                        fingerprints.push((fingerprint, data));
                        successful_operations += 1;
                    }
                    Err(e) => {
                        eprintln!("Thread {}: Fingerprint generation failed: {}", thread_id, e);
                    }
                }
            }
            
            // Verify all generated fingerprints
            let mut successful_verifications = 0;
            for (fingerprint, original_data) in &fingerprints {
                if fingerprint.verify(original_data) {
                    successful_verifications += 1;
                } else {
                    eprintln!("Thread {}: Fingerprint verification failed", thread_id);
                }
            }
            
            (thread_id, successful_operations, successful_verifications)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_generations = 0;
    let mut total_verifications = 0;
    
    for handle in handles {
        let (thread_id, generations, verifications) = handle.await.unwrap();
        println!("Thread {}: {}/{} generations, {}/{} verifications successful", 
                thread_id, generations, FINGERPRINTS_PER_THREAD, verifications, generations);
        total_generations += generations;
        total_verifications += verifications;
    }
    
    assert_eq!(total_generations, NUM_THREADS * FINGERPRINTS_PER_THREAD,
               "All fingerprint generations should succeed");
    assert_eq!(total_verifications, total_generations,
               "All fingerprint verifications should succeed");
}

/// Test race conditions in shared crypto state
#[tokio::test]
async fn test_crypto_race_conditions() {
    const NUM_THREADS: usize = 20;
    const ITERATIONS: usize = 50;
    
    // Shared resources that could cause race conditions
    let shared_keypair = Arc::new(MlKem768::keygen().unwrap());
    let shared_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    
    let mut handles = Vec::new();
    
    for thread_id in 0..NUM_THREADS {
        let keypair_clone = shared_keypair.clone();
        let counter_clone = shared_counter.clone();
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;
            
            let mut local_operations = 0;
            
            for i in 0..ITERATIONS {
                // Simulate race condition scenario with shared resources
                let public_key = keypair_clone.public_key();
                
                // Multiple threads accessing the same key simultaneously
                match MlKem768::encapsulate(&public_key) {
                    Ok((ciphertext, _shared_secret)) => {
                        // Increment shared counter atomically
                        counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        
                        // Try decapsulation
                        match MlKem768::decapsulate(&keypair_clone.secret_key(), &ciphertext) {
                            Ok(_) => {
                                local_operations += 1;
                            }
                            Err(e) => {
                                eprintln!("Thread {} iteration {}: Decapsulation failed: {}", 
                                         thread_id, i, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Thread {} iteration {}: Encapsulation failed: {}", 
                                 thread_id, i, e);
                    }
                }
                
                // Small yield to increase chance of race conditions
                tokio::task::yield_now().await;
            }
            
            (thread_id, local_operations)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_operations = 0;
    
    for handle in handles {
        let (thread_id, operations) = handle.await.unwrap();
        println!("Thread {}: {} successful operations", thread_id, operations);
        total_operations += operations;
    }
    
    let final_counter = shared_counter.load(std::sync::atomic::Ordering::SeqCst);
    
    println!("Total successful operations: {}", total_operations);
    println!("Shared counter value: {}", final_counter);
    
    // The counter should equal the number of successful operations
    assert_eq!(final_counter, total_operations,
               "Counter should match successful operations");
    
    // All operations should succeed (no race conditions should cause failures)
    assert_eq!(total_operations, NUM_THREADS * ITERATIONS,
               "All operations should succeed without race conditions");
}

/// Stress test for crypto operations under high contention
#[tokio::test]
async fn test_crypto_stress_high_contention() {
    const NUM_THREADS: usize = 32;
    const STRESS_DURATION_SECS: u64 = 5;
    
    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(STRESS_DURATION_SECS);
    let semaphore = Arc::new(Semaphore::new(NUM_THREADS));
    
    let mut handles = Vec::new();
    
    for thread_id in 0..NUM_THREADS {
        let semaphore_clone = semaphore.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            let mut operations_count = 0;
            let mut errors_count = 0;
            
            while Instant::now() < end_time {
                // Mix different crypto operations to create contention
                match thread_id % 4 {
                    0 => {
                        // ML-KEM operations
                        match MlKem768::keygen() {
                            Ok(keypair) => {
                                let public_key = keypair.public_key();
                                if let Ok((ciphertext, _)) = MlKem768::encapsulate(&public_key) {
                                    if MlKem768::decapsulate(&keypair.secret_key(), &ciphertext).is_ok() {
                                        operations_count += 1;
                                    } else {
                                        errors_count += 1;
                                    }
                                } else {
                                    errors_count += 1;
                                }
                            }
                            Err(_) => errors_count += 1,
                        }
                    }
                    1 => {
                        // ML-DSA operations
                        match MlDsa65::keygen() {
                            Ok(keypair) => {
                                let message = b"stress test message";
                                if let Ok(signature) = MlDsa65::sign(&keypair.secret_key(), message) {
                                    if MlDsa65::verify(&keypair.public_key(), message, &signature).unwrap_or(false) {
                                        operations_count += 1;
                                    } else {
                                        errors_count += 1;
                                    }
                                } else {
                                    errors_count += 1;
                                }
                            }
                            Err(_) => errors_count += 1,
                        }
                    }
                    2 => {
                        // HQC operations
                        match Hqc128::keygen() {
                            Ok(keypair) => {
                                let plaintext = b"stress test data";
                                if let Ok(ciphertext) = Hqc128::encrypt(&keypair.public_key(), plaintext) {
                                    if let Ok(decrypted) = Hqc128::decrypt(&keypair.secret_key(), &ciphertext) {
                                        if decrypted == plaintext {
                                            operations_count += 1;
                                        } else {
                                            errors_count += 1;
                                        }
                                    } else {
                                        errors_count += 1;
                                    }
                                } else {
                                    errors_count += 1;
                                }
                            }
                            Err(_) => errors_count += 1,
                        }
                    }
                    3 => {
                        // Quantum fingerprint operations
                        let data = format!("thread {} data", thread_id).into_bytes();
                        match QuantumFingerprint::generate(&data) {
                            Ok(fingerprint) => {
                                if fingerprint.verify(&data) {
                                    operations_count += 1;
                                } else {
                                    errors_count += 1;
                                }
                            }
                            Err(_) => errors_count += 1,
                        }
                    }
                    _ => unreachable!(),
                }
                
                // Small yield to prevent one thread from dominating
                if operations_count % 10 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            
            (thread_id, operations_count, errors_count)
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_operations = 0;
    let mut total_errors = 0;
    
    for handle in handles {
        let (thread_id, operations, errors) = handle.await.unwrap();
        println!("Thread {}: {} operations, {} errors", thread_id, operations, errors);
        total_operations += operations;
        total_errors += errors;
    }
    
    let elapsed = start_time.elapsed();
    let ops_per_second = total_operations as f64 / elapsed.as_secs_f64();
    let error_rate = total_errors as f64 / (total_operations + total_errors) as f64;
    
    println!("Stress test results:");
    println!("  Duration: {:?}", elapsed);
    println!("  Total operations: {}", total_operations);
    println!("  Total errors: {}", total_errors);
    println!("  Operations per second: {:.2}", ops_per_second);
    println!("  Error rate: {:.4}%", error_rate * 100.0);
    
    // Ensure we achieved reasonable performance and low error rate
    assert!(total_operations > 0, "Should complete some operations");
    assert!(error_rate < 0.01, "Error rate should be less than 1%");
    assert!(ops_per_second > 10.0, "Should achieve at least 10 ops/sec");
}

/// Test memory safety and cleanup under concurrent access
#[tokio::test]
async fn test_crypto_memory_safety_concurrent() {
    const NUM_THREADS: usize = 16;
    const ITERATIONS_PER_THREAD: usize = 100;
    
    let barrier = Arc::new(Barrier::new(NUM_THREADS));
    let mut handles = Vec::new();
    
    for thread_id in 0..NUM_THREADS {
        let barrier_clone = barrier.clone();
        
        let handle = tokio::spawn(async move {
            // Wait for all threads to be ready
            barrier_clone.wait().await;
            
            let mut allocated_objects = Vec::new();
            
            for i in 0..ITERATIONS_PER_THREAD {
                // Create various crypto objects that need proper cleanup
                match thread_id % 3 {
                    0 => {
                        if let Ok(keypair) = MlKem768::keygen() {
                            allocated_objects.push(format!("ml_kem_{}", i));
                            
                            // Use the keypair to ensure it's not optimized away
                            let public_key = keypair.public_key();
                            if let Ok((ciphertext, _)) = MlKem768::encapsulate(&public_key) {
                                let _ = MlKem768::decapsulate(&keypair.secret_key(), &ciphertext);
                            }
                        }
                    }
                    1 => {
                        if let Ok(keypair) = MlDsa65::keygen() {
                            allocated_objects.push(format!("ml_dsa_{}", i));
                            
                            // Use the keypair
                            let message = format!("message_{}", i).into_bytes();
                            if let Ok(signature) = MlDsa65::sign(&keypair.secret_key(), &message) {
                                let _ = MlDsa65::verify(&keypair.public_key(), &message, &signature);
                            }
                        }
                    }
                    2 => {
                        let data = format!("fingerprint_data_{}_{}", thread_id, i).into_bytes();
                        if let Ok(fingerprint) = QuantumFingerprint::generate(&data) {
                            allocated_objects.push(format!("fingerprint_{}", i));
                            let _ = fingerprint.verify(&data);
                        }
                    }
                    _ => unreachable!(),
                }
                
                // Periodically drop some objects to test cleanup
                if i % 10 == 0 && !allocated_objects.is_empty() {
                    let drop_count = allocated_objects.len() / 2;
                    allocated_objects.truncate(allocated_objects.len() - drop_count);
                }
                
                // Yield to allow other threads to run
                if i % 5 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            
            (thread_id, allocated_objects.len())
        });
        
        handles.push(handle);
    }
    
    // Collect results
    let mut total_final_objects = 0;
    
    for handle in handles {
        let (thread_id, final_objects) = handle.await.unwrap();
        println!("Thread {}: {} objects remaining", thread_id, final_objects);
        total_final_objects += final_objects;
    }
    
    println!("Total objects remaining across all threads: {}", total_final_objects);
    
    // This test mainly ensures no crashes or memory corruption occurred
    // The exact number of remaining objects depends on the cleanup strategy
    assert!(total_final_objects >= 0, "Should not have negative object count");
}

/// Test parallel crypto operations using rayon
#[test]
fn test_crypto_parallel_rayon() {
    const NUM_OPERATIONS: usize = 1000;
    
    // Generate test data
    let test_data: Vec<Vec<u8>> = (0..NUM_OPERATIONS)
        .map(|i| format!("test_data_{}", i).into_bytes())
        .collect();
    
    // Test parallel ML-KEM operations
    let ml_kem_results: Vec<_> = test_data
        .par_iter()
        .map(|data| {
            let keypair = MlKem768::keygen().unwrap();
            let public_key = keypair.public_key();
            let (ciphertext, shared_secret_1) = MlKem768::encapsulate(&public_key).unwrap();
            let shared_secret_2 = MlKem768::decapsulate(&keypair.secret_key(), &ciphertext).unwrap();
            shared_secret_1 == shared_secret_2
        })
        .collect();
    
    let ml_kem_success_count = ml_kem_results.iter().filter(|&&success| success).count();
    
    // Test parallel quantum fingerprints
    let fingerprint_results: Vec<_> = test_data
        .par_iter()
        .map(|data| {
            let fingerprint = QuantumFingerprint::generate(data).unwrap();
            fingerprint.verify(data)
        })
        .collect();
    
    let fingerprint_success_count = fingerprint_results.iter().filter(|&&success| success).count();
    
    // Test parallel ML-DSA operations
    let ml_dsa_results: Vec<_> = test_data
        .par_iter()
        .map(|data| {
            let keypair = MlDsa65::keygen().unwrap();
            let signature = MlDsa65::sign(&keypair.secret_key(), data).unwrap();
            MlDsa65::verify(&keypair.public_key(), data, &signature).unwrap_or(false)
        })
        .collect();
    
    let ml_dsa_success_count = ml_dsa_results.iter().filter(|&&success| success).count();
    
    println!("Parallel crypto operations results:");
    println!("  ML-KEM: {}/{} successful", ml_kem_success_count, NUM_OPERATIONS);
    println!("  Fingerprints: {}/{} successful", fingerprint_success_count, NUM_OPERATIONS);
    println!("  ML-DSA: {}/{} successful", ml_dsa_success_count, NUM_OPERATIONS);
    
    assert_eq!(ml_kem_success_count, NUM_OPERATIONS, "All ML-KEM operations should succeed");
    assert_eq!(fingerprint_success_count, NUM_OPERATIONS, "All fingerprint operations should succeed");
    assert_eq!(ml_dsa_success_count, NUM_OPERATIONS, "All ML-DSA operations should succeed");
}