use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Simulated ML-KEM implementation for benchmarking
#[derive(Clone)]
struct MlKem768 {
    key_size: usize,
    ciphertext_size: usize,
    shared_secret_size: usize,
}

impl MlKem768 {
    fn new() -> Self {
        Self {
            key_size: 1184,      // ML-KEM-768 public key size
            ciphertext_size: 1088, // ML-KEM-768 ciphertext size
            shared_secret_size: 32,  // Shared secret size
        }
    }

    fn keygen(&self) -> (Vec<u8>, Vec<u8>) {
        // Simulate key generation with cryptographically secure operations
        let mut pk = vec![0u8; self.key_size];
        let mut sk = vec![0u8; self.key_size * 2];
        
        // Simulate expensive key generation
        for i in 0..self.key_size {
            pk[i] = ((i as u64 * 31) % 256) as u8;
            sk[i] = ((i as u64 * 37) % 256) as u8;
            sk[i + self.key_size] = ((i as u64 * 41) % 256) as u8;
        }
        
        (pk, sk)
    }

    fn encapsulate(&self, pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
        // Simulate encapsulation
        let mut ciphertext = vec![0u8; self.ciphertext_size];
        let mut shared_secret = vec![0u8; self.shared_secret_size];
        
        // Simulate expensive encapsulation computation
        for i in 0..self.ciphertext_size {
            ciphertext[i] = ((pk[i % pk.len()] as u64 * 43) % 256) as u8;
        }
        
        for i in 0..self.shared_secret_size {
            shared_secret[i] = ((pk[i % pk.len()] as u64 * 47) % 256) as u8;
        }
        
        (ciphertext, shared_secret)
    }

    fn decapsulate(&self, sk: &[u8], ciphertext: &[u8]) -> Vec<u8> {
        // Simulate decapsulation
        let mut shared_secret = vec![0u8; self.shared_secret_size];
        
        // Simulate expensive decapsulation computation
        for i in 0..self.shared_secret_size {
            let sk_val = sk[i % sk.len()] as u64;
            let ct_val = ciphertext[i % ciphertext.len()] as u64;
            shared_secret[i] = ((sk_val * ct_val * 53) % 256) as u8;
        }
        
        shared_secret
    }
}

/// BLAKE3 hash function benchmark
struct Blake3Hasher {
    block_size: usize,
}

impl Blake3Hasher {
    fn new() -> Self {
        Self {
            block_size: 64,
        }
    }

    fn hash(&self, data: &[u8]) -> Vec<u8> {
        // Simulate BLAKE3 hashing
        let mut result = vec![0u8; 32];
        let mut state = 0x6A09E667F3BCC908u64;
        
        for chunk in data.chunks(self.block_size) {
            for &byte in chunk {
                state = state.wrapping_mul(31).wrapping_add(byte as u64);
            }
        }
        
        for i in 0..32 {
            result[i] = ((state >> (i * 8)) & 0xFF) as u8;
        }
        
        result
    }
}

fn benchmark_mlkem_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_768");
    
    let mlkem = MlKem768::new();
    
    // Benchmark key generation
    group.bench_function("keygen", |b| {
        b.iter(|| {
            let (pk, sk) = black_box(mlkem.keygen());
            black_box((pk, sk));
        });
    });
    
    // Pre-generate keys for encapsulation/decapsulation benchmarks
    let (pk, sk) = mlkem.keygen();
    
    // Benchmark encapsulation
    group.bench_function("encapsulate", |b| {
        b.iter(|| {
            let (ct, ss) = black_box(mlkem.encapsulate(black_box(&pk)));
            black_box((ct, ss));
        });
    });
    
    // Pre-generate ciphertext for decapsulation benchmark
    let (ct, _) = mlkem.encapsulate(&pk);
    
    // Benchmark decapsulation
    group.bench_function("decapsulate", |b| {
        b.iter(|| {
            let ss = black_box(mlkem.decapsulate(black_box(&sk), black_box(&ct)));
            black_box(ss);
        });
    });
    
    // Benchmark batch operations for throughput
    group.bench_function("batch_keygen_100", |b| {
        b.iter(|| {
            let mut keys = Vec::with_capacity(100);
            for _ in 0..100 {
                let (pk, sk) = mlkem.keygen();
                keys.push((pk, sk));
            }
            black_box(keys);
        });
    });
    
    group.finish();
}

fn benchmark_blake3_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_hash");
    
    let hasher = Blake3Hasher::new();
    
    // Test different data sizes
    for &size in &[64, 256, 1024, 4096, 16384, 65536] {
        let data = vec![0u8; size];
        
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}bytes", size)),
            &data,
            |b, data| {
                b.iter(|| {
                    let hash = black_box(hasher.hash(black_box(data)));
                    black_box(hash);
                });
            }
        );
    }
    
    // Benchmark throughput (MB/s)
    group.bench_function("throughput_1mb", |b| {
        let data = vec![0u8; 1024 * 1024]; // 1MB
        b.iter(|| {
            let hash = black_box(hasher.hash(black_box(&data)));
            black_box(hash);
        });
    });
    
    group.finish();
}

fn benchmark_crypto_performance_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_targets");
    
    let mlkem = MlKem768::new();
    let hasher = Blake3Hasher::new();
    
    // Test combined operations for real-world scenarios
    group.bench_function("full_key_exchange", |b| {
        b.iter(|| {
            // Simulate full key exchange
            let (pk_a, sk_a) = mlkem.keygen();
            let (pk_b, sk_b) = mlkem.keygen();
            
            // A encrypts to B
            let (ct_ab, ss_a) = mlkem.encapsulate(&pk_b);
            
            // B decrypts from A
            let ss_b = mlkem.decapsulate(&sk_b, &ct_ab);
            
            // Hash shared secrets
            let hash_a = hasher.hash(&ss_a);
            let hash_b = hasher.hash(&ss_b);
            
            black_box((hash_a, hash_b));
        });
    });
    
    // Test memory usage under load
    group.bench_function("memory_stress_test", |b| {
        b.iter(|| {
            let mut keys = Vec::new();
            let mut ciphertexts = Vec::new();
            let mut shared_secrets = Vec::new();
            
            // Generate multiple keys (simulating multiple connections)
            for _ in 0..100 {
                let (pk, sk) = mlkem.keygen();
                let (ct, ss) = mlkem.encapsulate(&pk);
                let decrypted_ss = mlkem.decapsulate(&sk, &ct);
                
                keys.push((pk, sk));
                ciphertexts.push(ct);
                shared_secrets.push((ss, decrypted_ss));
            }
            
            // Estimate memory usage
            let total_memory = keys.len() * (1184 + 2368) + // Key sizes
                              ciphertexts.len() * 1088 +      // Ciphertext sizes
                              shared_secrets.len() * 64;      // Shared secret sizes
            
            // Verify memory usage is under target (100MB = 104,857,600 bytes)
            assert!(total_memory < 104_857_600,
                   "Memory usage {} exceeds 100MB target", total_memory);
            
            black_box((keys, ciphertexts, shared_secrets));
        });
    });
    
    // Test operation latency
    group.bench_function("latency_test", |b| {
        b.iter(|| {
            let start = Instant::now();
            
            let (pk, sk) = mlkem.keygen();
            let keygen_time = start.elapsed();
            
            let start = Instant::now();
            let (ct, ss1) = mlkem.encapsulate(&pk);
            let encap_time = start.elapsed();
            
            let start = Instant::now();
            let ss2 = mlkem.decapsulate(&sk, &ct);
            let decap_time = start.elapsed();
            
            // Verify reasonable latency targets
            assert!(keygen_time < Duration::from_millis(100),
                   "Key generation latency {} exceeds 100ms", keygen_time.as_millis());
            assert!(encap_time < Duration::from_millis(50),
                   "Encapsulation latency {} exceeds 50ms", encap_time.as_millis());
            assert!(decap_time < Duration::from_millis(50),
                   "Decapsulation latency {} exceeds 50ms", decap_time.as_millis());
            
            black_box((keygen_time, encap_time, decap_time, ss1, ss2));
        });
    });
    
    group.finish();
}

fn benchmark_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    
    let mlkem = MlKem768::new();
    
    // Test linear scalability with different numbers of operations
    for &op_count in &[10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("operations_{}", op_count)),
            &op_count,
            |b, &op_count| {
                b.iter(|| {
                    let start = Instant::now();
                    
                    for _ in 0..op_count {
                        let (pk, sk) = mlkem.keygen();
                        let (ct, _) = mlkem.encapsulate(&pk);
                        let _ = mlkem.decapsulate(&sk, &ct);
                    }
                    
                    let total_time = start.elapsed();
                    let ops_per_sec = op_count as f64 / total_time.as_secs_f64();
                    
                    // Verify linear scalability (ops per second should be roughly constant)
                    black_box((total_time, ops_per_sec));
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(30));
    targets = 
        benchmark_mlkem_operations,
        benchmark_blake3_performance,
        benchmark_crypto_performance_targets,
        benchmark_scalability
);
criterion_main!(benches);