use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, measurement::WallTime};
use qudag_crypto::kem::MLKem;
use rand::thread_rng;
use std::time::Instant;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Track memory allocations
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: MemoryTracker = MemoryTracker;

struct MemoryTracker;

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

fn bench_keygen(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_keygen");
    group.sample_size(100); // Increase sample size for better statistics
    
    // Reset allocation counter
    ALLOCATED.store(0, Ordering::SeqCst);
    
    group.bench_function("keygen_performance", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            black_box(MLKem::generate_keypair(&mut rng).unwrap())
        })
    });
    
    // Measure memory usage
    let mem_usage = ALLOCATED.load(Ordering::SeqCst);
    println!("Key Generation Memory Usage: {} bytes", mem_usage);
    
    // Measure timing consistency
    let mut timings = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let mut rng = thread_rng();
        let _ = MLKem::generate_keypair(&mut rng).unwrap();
        timings.push(start.elapsed());
    }
    
    // Calculate timing statistics
    let mean = timings.iter().sum::<std::time::Duration>() / timings.len() as u32;
    let variance: f64 = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean.as_nanos() as f64;
            diff * diff
        })
        .sum::<f64>() / timings.len() as f64;
    let std_dev = (variance as f64).sqrt();
    
    println!("Key Generation Timing Statistics:");
    println!("  Mean: {:?}", mean);
    println!("  Std Dev: {} ns", std_dev);
    println!("  Coefficient of Variation: {:.2}%", (std_dev / mean.as_nanos() as f64) * 100.0);
    
    group.finish();
}

fn bench_encapsulate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_encapsulate");
    group.sample_size(100);
    let mut rng = thread_rng();
    let (pk, _) = MLKem::generate_keypair(&mut rng).unwrap();
    
    // Reset allocation counter
    ALLOCATED.store(0, Ordering::SeqCst);
    
    group.bench_function("encapsulate_performance", |b| {
        b.iter(|| {
            black_box(MLKem::encapsulate(black_box(&pk.public_key)).unwrap())
        })
    });
    
    // Measure memory usage
    let mem_usage = ALLOCATED.load(Ordering::SeqCst);
    println!("Encapsulation Memory Usage: {} bytes", mem_usage);
    
    // Measure timing consistency
    let mut timings = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let _ = MLKem::encapsulate(&pk.public_key).unwrap();
        timings.push(start.elapsed());
    }
    
    // Calculate timing statistics
    let mean = timings.iter().sum::<std::time::Duration>() / timings.len() as u32;
    let variance: f64 = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as f64 - mean.as_nanos() as f64;
            diff * diff
        })
        .sum::<f64>() / timings.len() as f64;
    let std_dev = (variance as f64).sqrt();
    
    println!("Encapsulation Timing Statistics:");
    println!("  Mean: {:?}", mean);
    println!("  Std Dev: {} ns", std_dev);
    println!("  Coefficient of Variation: {:.2}%", (std_dev / mean.as_nanos() as f64) * 100.0);
    
    group.finish();
}

fn bench_decapsulate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_decapsulate");
    group.sample_size(100);
    let mut rng = thread_rng();
    let (pk, sk) = MLKem::generate_keypair(&mut rng).unwrap();
    let (_, ct) = MLKem::encapsulate(&pk.public_key).unwrap();
    
    // Create an invalid ciphertext for timing comparison
    let mut invalid_ct = ct.clone();
    invalid_ct.as_mut()[0] ^= 0xFF;
    
    // Reset allocation counter
    ALLOCATED.store(0, Ordering::SeqCst);
    
    group.bench_function("decapsulate_valid", |b| {
        b.iter(|| {
            black_box(MLKem::decapsulate(
                black_box(&sk.secret_key),
                black_box(&ct)
            ).unwrap())
        })
    });
    
    group.bench_function("decapsulate_invalid", |b| {
        b.iter(|| {
            let _ = MLKem::decapsulate(
                black_box(&sk.secret_key),
                black_box(&invalid_ct)
            );
        })
    });
    
    // Measure memory usage
    let mem_usage = ALLOCATED.load(Ordering::SeqCst);
    println!("Decapsulation Memory Usage: {} bytes", mem_usage);
    
    // Measure timing consistency between valid and invalid inputs
    let mut timings_valid = Vec::new();
    let mut timings_invalid = Vec::new();
    
    for _ in 0..100 {
        let start = Instant::now();
        let _ = MLKem::decapsulate(&sk.secret_key, &ct).unwrap();
        timings_valid.push(start.elapsed());
        
        let start = Instant::now();
        let _ = MLKem::decapsulate(&sk.secret_key, &invalid_ct);
        timings_invalid.push(start.elapsed());
    }
    
    // Calculate timing statistics
    let mean_valid = timings_valid.iter().sum::<std::time::Duration>() / timings_valid.len() as u32;
    let mean_invalid = timings_invalid.iter().sum::<std::time::Duration>() / timings_invalid.len() as u32;
    
    println!("Decapsulation Timing Analysis:");
    println!("  Valid Mean: {:?}", mean_valid);
    println!("  Invalid Mean: {:?}", mean_invalid);
    println!("  Timing Difference: {:.2}%", 
        ((mean_valid.as_nanos() as f64 - mean_invalid.as_nanos() as f64).abs() 
         / mean_valid.as_nanos() as f64) * 100.0);
    
    group.finish();
}

fn bench_full_exchange(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_exchange");
    group.sample_size(100);
    
    // Test throughput with different batch sizes
    for size in [1, 10, 100, 1000].iter() {
        // Reset allocation counter
        ALLOCATED.store(0, Ordering::SeqCst);
        
        group.bench_with_input(BenchmarkId::new("exchange", size), size, |b, &size| {
            b.iter(|| {
                let mut rng = thread_rng();
                let (pk, sk) = MLKem::generate_keypair(&mut rng).unwrap();
                
                for _ in 0..size {
                    let (ct, ss1) = MLKem::encapsulate(&pk.public_key).unwrap();
                    let ss2 = MLKem::decapsulate(&sk.secret_key, &ct).unwrap();
                    black_box((ss1, ss2));
                }
            })
        });
        
        // Calculate throughput metrics
        let mem_usage = ALLOCATED.load(Ordering::SeqCst);
        println!("Batch Size {}: Memory Usage per Operation: {} bytes", 
            size, mem_usage / size);
    }
    
    // Measure cache performance
    let mut rng = thread_rng();
    let mut keys = Vec::new();
    for _ in 0..5 {
        keys.push(MLKem::generate_keypair(&mut rng).unwrap());
    }
    
    group.bench_function("cache_performance", |b| {
        b.iter(|| {
            for (pk, sk) in &keys {
                let (ct, _) = MLKem::encapsulate(&pk.public_key).unwrap();
                black_box(MLKem::decapsulate(&sk.secret_key, &ct).unwrap());
            }
        })
    });
    
    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_throughput");
    group.sample_size(10); // Reduced sample size due to high iteration count
    
    // Measure operations per second
    group.bench_function("ops_per_second", |b| {
        b.iter(|| {
            let mut rng = thread_rng();
            let (pk, sk) = MLKem::generate_keypair(&mut rng).unwrap();
            let (ct, ss1) = MLKem::encapsulate(&pk.public_key).unwrap();
            black_box(MLKem::decapsulate(&sk.secret_key, &ct).unwrap())
        })
    });
    
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(2));
    targets = 
        bench_keygen,
        bench_encapsulate,
        bench_decapsulate,
        bench_full_exchange,
        bench_throughput
);

criterion_main!(benches);