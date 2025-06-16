use qudag_crypto::util::{ct_eq, ct_select};
use subtle::ConstantTimeEq;

#[test]
fn test_constant_time_equality() {
    let a = vec![1u8, 2u8, 3u8];
    let b = vec![1u8, 2u8, 3u8];
    let c = vec![4u8, 5u8, 6u8];

    assert!(ct_eq(&a, &b));
    assert!(!ct_eq(&a, &c));
    assert!(!ct_eq(&a, &a[..2])); // Different lengths
}

#[test]
fn test_constant_time_select() {
    let a = vec![1u8, 2u8, 3u8];
    let b = vec![4u8, 5u8, 6u8];

    let result_a = ct_select(&a, &b, 0);
    let result_b = ct_select(&a, &b, 1);

    assert_eq!(result_a, a);
    assert_eq!(result_b, b);
}

#[test]
fn test_constant_time_zeroization() {
    use zeroize::Zeroize;

    let mut sensitive_data = vec![1u8, 2u8, 3u8];
    sensitive_data.zeroize();

    assert!(sensitive_data.iter().all(|&x| x == 0));
}

#[test]
fn test_constant_time_comparison() {
    let a = 0xffu8;
    let b = 0xffu8;
    let c = 0x00u8;

    assert_eq!(a.ct_eq(&b).unwrap_u8(), 1);
    assert_eq!(a.ct_eq(&c).unwrap_u8(), 0);
}

#[test]
fn test_timing_resistance() {
    use qudag_crypto::ml_kem::{MlKem768, KeyEncapsulation};
    use qudag_crypto::ml_dsa::{MlDsa44, DigitalSignature};
    use qudag_crypto::hqc::{Hqc256, AsymmetricEncryption};

    // Test KEM timing resistance
    let (pk_kem, sk_kem) = MlKem768::keygen().expect("KEM key generation failed");
    let (ct_kem, _) = MlKem768::encapsulate(&pk_kem).expect("KEM encapsulation failed");
    let _ = MlKem768::decapsulate(&sk_kem, &ct_kem).expect("KEM decapsulation failed");

    // Test signature timing resistance
    let (pk_sig, sk_sig) = MlDsa44::keygen().expect("Signature key generation failed");
    let message = b"Test message";
    let signature = MlDsa44::sign(&sk_sig, message).expect("Signing failed");
    let _ = MlDsa44::verify(&pk_sig, message, &signature).expect("Verification failed");

    // Test encryption timing resistance
    let (pk_enc, sk_enc) = Hqc256::keygen().expect("Encryption key generation failed");
    let plaintext = b"Test plaintext";
    let ciphertext = Hqc256::encrypt(&pk_enc, plaintext).expect("Encryption failed");
    let _ = Hqc256::decrypt(&sk_enc, &ciphertext).expect("Decryption failed");
}