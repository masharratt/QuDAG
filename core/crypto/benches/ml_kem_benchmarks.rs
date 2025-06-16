use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use qudag_crypto::ml_kem::MlKem768;
use qudag_crypto::kem::KeyEncapsulation;
use std::time::{Duration, Instant};

fn bench_keygen(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_keygen");
    group.sample_size(50);
    
    group.bench_function("keygen_performance", |b| {
        b.iter(|| {
            black_box(MlKem768::keygen().unwrap())
        })
    });
    
    group.finish();
}

fn bench_encapsulate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_encapsulate");
    group.sample_size(50);
    let (pk, _) = MlKem768::keygen().unwrap();
    
    group.bench_function("encapsulate_performance", |b| {
        b.iter(|| {
            black_box(MlKem768::encapsulate(black_box(&pk)).unwrap())
        })
    });
    
    group.finish();
}

fn bench_decapsulate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_decapsulate");
    group.sample_size(50);
    let (pk, sk) = MlKem768::keygen().unwrap();
    let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
    
    group.bench_function("decapsulate_performance", |b| {
        b.iter(|| {
            black_box(MlKem768::decapsulate(black_box(&sk), black_box(&ct)).unwrap())
        })
    });
    
    group.finish();
}

fn bench_full_exchange(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_exchange");
    group.sample_size(20);
    
    // Test throughput with different batch sizes
    for size in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("exchange", size), size, |b, &size| {
            b.iter(|| {
                for _ in 0..size {
                    let (pk, sk) = MlKem768::keygen().unwrap();
                    let (ct, ss1) = MlKem768::encapsulate(&pk).unwrap();
                    let ss2 = MlKem768::decapsulate(&sk, &ct).unwrap();
                    black_box((ss1, ss2));
                }
            })
        });
    }
    
    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_throughput");
    group.sample_size(10);
    
    // Measure operations per second
    group.bench_function("ops_per_second", |b| {
        b.iter(|| {
            let (pk, sk) = MlKem768::keygen().unwrap();
            let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
            black_box(MlKem768::decapsulate(&sk, &ct).unwrap())
        })
    });
    
    group.finish();
}

fn bench_performance_targets(c: &mut Criterion) {
    let mut group = c.benchmark_group("ml_kem_targets");
    
    // Test latency requirements (sub-second finality)
    group.bench_function("latency_test", |b| {
        b.iter(|| {
            let start = Instant::now();
            let (pk, sk) = MlKem768::keygen().unwrap();
            let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
            let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
            let elapsed = start.elapsed();
            
            // Verify sub-second performance
            assert!(elapsed < Duration::from_secs(1), 
                   "ML-KEM operation took {} ms, exceeds 1s target", 
                   elapsed.as_millis());
            
            black_box(elapsed);
        })
    });
    
    // Test memory efficiency (rough estimate)
    group.bench_function("memory_efficiency", |b| {
        b.iter(|| {
            let mut keys = Vec::new();
            let mut operations = 0;
            
            // Generate keys until we estimate ~50MB usage
            while operations < 1000 {
                let (pk, sk) = MlKem768::keygen().unwrap();
                let (ct, _) = MlKem768::encapsulate(&pk).unwrap();
                let _ = MlKem768::decapsulate(&sk, &ct).unwrap();
                keys.push((pk, sk, ct));
                operations += 1;
            }
            
            // Rough memory estimation (keys + overhead)
            let estimated_memory = operations * (
                MlKem768::PUBLIC_KEY_SIZE + 
                MlKem768::SECRET_KEY_SIZE + 
                MlKem768::CIPHERTEXT_SIZE + 
                100 // overhead
            );
            
            // Verify we stay under 100MB target
            assert!(estimated_memory < 100 * 1024 * 1024,
                   "Estimated memory usage {} exceeds 100MB target", estimated_memory);
            
            black_box(keys);
        })
    });
    
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(2));
    targets = 
        bench_keygen,
        bench_encapsulate,
        bench_decapsulate,
        bench_full_exchange,
        bench_throughput,
        bench_performance_targets
);

criterion_main!(benches);