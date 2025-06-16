use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_crypto::hqc::{HQC, CIPHERTEXT_LENGTH};

fn benchmark_hqc(c: &mut Criterion) {
    c.bench_function("hqc_keygen", |b| {
        b.iter(|| {
            black_box(HQC::keygen().expect("Key generation failed"));
        });
    });

    let (pk, sk) = HQC::keygen().expect("Key generation failed");
    let message = b"Test message for HQC benchmarking";
    
    c.bench_function("hqc_encrypt", |b| {
        b.iter(|| {
            black_box(pk.encrypt(black_box(message)).expect("Encryption failed"));
        });
    });

    let ciphertext = pk.encrypt(message).expect("Encryption failed");
    
    c.bench_function("hqc_decrypt", |b| {
        b.iter(|| {
            black_box(sk.decrypt(black_box(&ciphertext)).expect("Decryption failed"));
        });
    });
}

criterion_group!(benches, benchmark_hqc);
criterion_main!(benches);