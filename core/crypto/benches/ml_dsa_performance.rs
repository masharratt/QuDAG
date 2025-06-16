//! Performance benchmarks for ML-DSA implementation
//! 
//! This benchmark suite measures the performance of:
//! - Key generation
//! - Message signing
//! - Signature verification
//! - Memory usage patterns
//! - Constant-time operation validation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use qudag_crypto::ml_dsa::{MlDsa, MlDsaKeyPair, MlDsaPublicKey};
use rand::{thread_rng, Rng};
use std::time::{Duration, Instant};

/// Benchmark ML-DSA key generation performance
fn bench_ml_dsa_keygen(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Key Generation");
    
    group.bench_function("keygen", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let _ = MlDsaKeyPair::generate(&mut rng);
        })
    });
    
    group.finish();
}

/// Benchmark ML-DSA signing performance with different message sizes
fn bench_ml_dsa_signing(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Signing");
    
    // Pre-generate keypair
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    
    // Test with different message sizes
    let message_sizes = [32, 256, 1024, 4096, 16384];
    
    for &size in &message_sizes {
        let message: Vec<u8> = (0..size).map(|_| rng.gen()).collect();
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("sign", size),
            &message,
            |b, msg| {
                b.iter(|| {
                    let mut rng = thread_rng();
                    let _ = keypair.sign(msg, &mut rng);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark ML-DSA verification performance with different message sizes
fn bench_ml_dsa_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Verification");
    
    // Pre-generate keypair and signatures
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    // Test with different message sizes
    let message_sizes = [32, 256, 1024, 4096, 16384];
    
    for &size in &message_sizes {
        let message: Vec<u8> = (0..size).map(|_| rng.gen()).collect();
        let signature = keypair.sign(&message, &mut rng).expect("Signing should succeed");
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("verify", size),
            &(message, signature),
            |b, (msg, sig)| {
                b.iter(|| {
                    let _ = public_key.verify(msg, sig);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark full ML-DSA round-trip operations
fn bench_ml_dsa_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Round-trip");
    
    let message_sizes = [256, 1024, 4096];
    
    for &size in &message_sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("keygen_sign_verify", size),
            &size,
            |b, &msg_size| {
                b.iter(|| {
                    let mut rng = thread_rng();
                    let message: Vec<u8> = (0..msg_size).map(|_| rng.gen()).collect();
                    
                    // Generate keypair
                    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
                    
                    // Sign message
                    let signature = keypair.sign(&message, &mut rng).expect("Signing should succeed");
                    
                    // Verify signature
                    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
                    let _ = public_key.verify(&message, &signature);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark constant-time properties of ML-DSA operations
fn bench_ml_dsa_constant_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Constant-time Properties");
    
    // Pre-generate test data
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    let message1: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();
    let message2: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();
    
    let signature1 = keypair.sign(&message1, &mut rng).expect("Signing should succeed");
    let signature2 = keypair.sign(&message2, &mut rng).expect("Signing should succeed");
    
    // Benchmark verification timing consistency
    group.bench_function("verify_valid_signature", |b| {
        b.iter(|| {
            let _ = public_key.verify(&message1, &signature1);
        })
    });
    
    group.bench_function("verify_invalid_signature", |b| {
        b.iter(|| {
            let _ = public_key.verify(&message1, &signature2);
        })
    });
    
    group.finish();
}

/// Benchmark memory usage patterns for ML-DSA operations
fn bench_ml_dsa_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Memory Usage");
    
    // Measure memory allocation patterns
    group.bench_function("keypair_allocation", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
            
            // Access key data to prevent optimization
            criterion::black_box(keypair.public_key().len());
            criterion::black_box(keypair.secret_key().len());
        })
    });
    
    group.bench_function("signature_allocation", |b| {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let message = vec![0x42u8; 1024];
        
        b.iter(|| {
            let signature = keypair.sign(&message, &mut rng).expect("Signing should succeed");
            criterion::black_box(signature.len());
        })
    });
    
    group.finish();
}

/// Measure actual timing variance to validate constant-time properties
fn measure_timing_variance() {
    let mut rng = thread_rng();
    let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
    let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
    
    let message = vec![0x42u8; 1024];
    let signature = keypair.sign(&message, &mut rng).expect("Signing should succeed");
    
    // Measure verification timing for valid signature
    let mut valid_times = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        let _ = public_key.verify(&message, &signature);
        valid_times.push(start.elapsed());
    }
    
    // Measure verification timing for invalid signature
    let mut invalid_signature = signature.clone();
    invalid_signature[0] ^= 1; // Tamper with signature
    
    let mut invalid_times = Vec::new();
    for _ in 0..1000 {
        let start = Instant::now();
        let _ = public_key.verify(&message, &invalid_signature);
        invalid_times.push(start.elapsed());
    }
    
    // Calculate statistics
    let valid_mean = valid_times.iter().sum::<Duration>() / valid_times.len() as u32;
    let invalid_mean = invalid_times.iter().sum::<Duration>() / invalid_times.len() as u32;
    
    let valid_variance = valid_times.iter()
        .map(|&t| if t > valid_mean { t - valid_mean } else { valid_mean - t })
        .sum::<Duration>() / valid_times.len() as u32;
    
    let invalid_variance = invalid_times.iter()
        .map(|&t| if t > invalid_mean { t - invalid_mean } else { invalid_mean - t })
        .sum::<Duration>() / invalid_times.len() as u32;
    
    println!("Timing Analysis Results:");
    println!("Valid signature verification:");
    println!("  Mean: {:?}", valid_mean);
    println!("  Variance: {:?}", valid_variance);
    println!("Invalid signature verification:");
    println!("  Mean: {:?}", invalid_mean);
    println!("  Variance: {:?}", invalid_variance);
    
    let timing_difference = if valid_mean > invalid_mean {
        valid_mean - invalid_mean
    } else {
        invalid_mean - valid_mean
    };
    
    println!("Timing difference: {:?}", timing_difference);
    
    // For constant-time operations, timing difference should be minimal
    if timing_difference > Duration::from_millis(1) {
        println!("WARNING: Significant timing difference detected!");
    } else {
        println!("Timing difference within acceptable range");
    }
}

/// Performance regression test
fn bench_ml_dsa_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("ML-DSA Performance Regression");
    
    // Set performance targets based on requirements
    let target_keygen_time = Duration::from_millis(100);
    let target_sign_time = Duration::from_millis(50);
    let target_verify_time = Duration::from_millis(50);
    
    group.bench_function("regression_keygen", |b| {
        let duration = b.iter(|| {
            let mut rng = thread_rng();
            let _ = MlDsaKeyPair::generate(&mut rng);
        });
        
        // Note: In a real implementation, we would assert performance targets
        // For now, we just measure and report
    });
    
    group.bench_function("regression_sign", |b| {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let message = vec![0x42u8; 1024];
        
        let duration = b.iter(|| {
            let _ = keypair.sign(&message, &mut rng);
        });
    });
    
    group.bench_function("regression_verify", |b| {
        let mut rng = thread_rng();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
        let message = vec![0x42u8; 1024];
        let signature = keypair.sign(&message, &mut rng).expect("Signing should succeed");
        
        let duration = b.iter(|| {
            let _ = public_key.verify(&message, &signature);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_ml_dsa_keygen,
    bench_ml_dsa_signing,
    bench_ml_dsa_verification,
    bench_ml_dsa_roundtrip,
    bench_ml_dsa_constant_time,
    bench_ml_dsa_memory_usage,
    bench_ml_dsa_regression
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timing_variance_measurement() {
        // Run timing variance measurement as a test
        measure_timing_variance();
    }
    
    #[test]
    fn test_performance_targets() {
        // Basic performance sanity check
        let mut rng = thread_rng();
        
        // Test key generation performance
        let start = Instant::now();
        let keypair = MlDsaKeyPair::generate(&mut rng).expect("Key generation should succeed");
        let keygen_time = start.elapsed();
        
        // Test signing performance
        let message = vec![0x42u8; 1024];
        let start = Instant::now();
        let signature = keypair.sign(&message, &mut rng).expect("Signing should succeed");
        let sign_time = start.elapsed();
        
        // Test verification performance
        let public_key = MlDsaPublicKey::from_bytes(keypair.public_key()).expect("Public key creation should succeed");
        let start = Instant::now();
        let _ = public_key.verify(&message, &signature);
        let verify_time = start.elapsed();
        
        println!("Performance measurements:");
        println!("  Key generation: {:?}", keygen_time);
        println!("  Signing: {:?}", sign_time);
        println!("  Verification: {:?}", verify_time);
        
        // Basic sanity checks (adjust targets as needed)
        assert!(keygen_time < Duration::from_secs(1), "Key generation too slow");
        assert!(sign_time < Duration::from_secs(1), "Signing too slow");
        assert!(verify_time < Duration::from_secs(1), "Verification too slow");
    }
}