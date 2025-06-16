use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_crypto::{
    kem::{self, ml_kem},
    signatures::{self, ml_dsa},
    encryption::{self, hqc},
};
use rand::thread_rng;

fn benchmark_ml_kem(c: &mut Criterion) {
    let mut rng = thread_rng();
    let keypair = ml_kem::generate_keypair(&mut rng).unwrap();

    c.bench_function("ml_kem_keygen", |b| {
        b.iter(|| ml_kem::generate_keypair(&mut rng))
    });

    c.bench_function("ml_kem_encap", |b| {
        b.iter(|| ml_kem::encapsulate(black_box(&keypair.public_key)))
    });

    let (_, ciphertext) = ml_kem::encapsulate(&keypair.public_key).unwrap();
    c.bench_function("ml_kem_decap", |b| {
        b.iter(|| ml_kem::decapsulate(black_box(&keypair.secret_key), black_box(&ciphertext)))
    });
}

fn benchmark_ml_dsa(c: &mut Criterion) {
    let mut rng = thread_rng();
    let message = b"benchmark test message";
    let keypair = ml_dsa::generate_keypair(&mut rng).unwrap();

    c.bench_function("ml_dsa_keygen", |b| {
        b.iter(|| ml_dsa::generate_keypair(&mut rng))
    });

    c.bench_function("ml_dsa_sign", |b| {
        b.iter(|| ml_dsa::sign(black_box(&keypair.secret_key), black_box(message)))
    });

    let signature = ml_dsa::sign(&keypair.secret_key, message).unwrap();
    c.bench_function("ml_dsa_verify", |b| {
        b.iter(|| ml_dsa::verify(black_box(&keypair.public_key), black_box(message), black_box(&signature)))
    });
}

fn benchmark_hqc(c: &mut Criterion) {
    let mut rng = thread_rng();
    let message = b"benchmark test message";
    let keypair = hqc::generate_keypair(&mut rng).unwrap();

    c.bench_function("hqc_keygen", |b| {
        b.iter(|| hqc::generate_keypair(&mut rng))
    });

    c.bench_function("hqc_encrypt", |b| {
        b.iter(|| hqc::encrypt(&mut rng, black_box(&keypair.public_key), black_box(message)))
    });

    let ciphertext = hqc::encrypt(&mut rng, &keypair.public_key, message).unwrap();
    c.bench_function("hqc_decrypt", |b| {
        b.iter(|| hqc::decrypt(black_box(&keypair.secret_key), black_box(&ciphertext)))
    });
}

criterion_group!(
    benches,
    benchmark_ml_kem,
    benchmark_ml_dsa,
    benchmark_hqc
);
criterion_main!(benches);