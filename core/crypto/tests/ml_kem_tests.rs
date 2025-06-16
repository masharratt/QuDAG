use qudag_crypto::kem::{KEMError, KeyEncapsulation};
use qudag_crypto::ml_kem::MlKem768;
use proptest::prelude::*;
use hex_literal::hex;

// Official ML-KEM-768 test vectors
const TEST_SEED: [u8; 32] = hex!("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
const TEST_PK: [u8; MlKem768::PUBLIC_KEY_SIZE] = include!(".test_vectors/mlkem768_pk.txt");
const TEST_SK: [u8; MlKem768::SECRET_KEY_SIZE] = include!(".test_vectors/mlkem768_sk.txt");
const TEST_CT: [u8; MlKem768::CIPHERTEXT_SIZE] = include!(".test_vectors/mlkem768_ct.txt");
const TEST_SS: [u8; MlKem768::SHARED_SECRET_SIZE] = include!(".test_vectors/mlkem768_ss.txt");

#[test]
fn test_mlkem_key_generation() {
    let (pk, sk) = MlKem768::keygen().expect("Key generation should succeed");
    
    // Verify key sizes
    assert_eq!(pk.as_bytes().len(), MlKem768::PUBLIC_KEY_SIZE);
    assert_eq!(sk.as_bytes().len(), MlKem768::SECRET_KEY_SIZE);
    
    // Verify keys are not all zeros
    assert_ne!(pk.as_bytes(), &[0u8; MlKem768::PUBLIC_KEY_SIZE]);
    assert_ne!(sk.as_bytes(), &[0u8; MlKem768::SECRET_KEY_SIZE]);
}

#[test]
fn test_mlkem_encapsulation_decapsulation() {
    let (pk, sk) = MlKem768::keygen().expect("Key generation should succeed");
    let (ciphertext, shared_secret_1) = MlKem768::encapsulate(&pk).expect("Encapsulation should succeed");
    let shared_secret_2 = MlKem768::decapsulate(&sk, &ciphertext).expect("Decapsulation should succeed");
    
    // Verify sizes
    assert_eq!(ciphertext.as_bytes().len(), MlKem768::CIPHERTEXT_SIZE);
    assert_eq!(shared_secret_1.as_bytes().len(), MlKem768::SHARED_SECRET_SIZE);
    assert_eq!(shared_secret_2.as_bytes().len(), MlKem768::SHARED_SECRET_SIZE);
    
    // Verify shared secrets match
    assert_eq!(shared_secret_1.as_bytes(), shared_secret_2.as_bytes());
    
    // Verify ciphertext and shared secret are not all zeros
    assert_ne!(ciphertext.as_bytes(), &[0u8; MlKem768::CIPHERTEXT_SIZE]);
    assert_ne!(shared_secret_1.as_bytes(), &[0u8; MlKem768::SHARED_SECRET_SIZE]);
}

#[test]
fn test_mlkem_with_test_vectors() {
    // Test decapsulation with known test vectors
    let ss = MlKem768::decapsulate(
        &MlKem768::SecretKey(TEST_SK),
        &MlKem768::Ciphertext(TEST_CT)
    ).expect("Decapsulation with test vectors should succeed");
    
    assert_eq!(ss.as_bytes(), &TEST_SS);
}

#[test]
fn test_mlkem_invalid_inputs() {
    let (_, sk) = MlKem768::keygen().expect("Key generation should succeed");
    
    // Test with invalid ciphertext length
    let short_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE - 1];
    let result = MlKem768::decapsulate(&sk, &short_ct);
    assert!(matches!(result, Err(KEMError::InvalidParameters)));
    
    // Test with random invalid ciphertext
    let mut invalid_ct = vec![0u8; MlKem768::CIPHERTEXT_SIZE];
    rand::thread_rng().fill_bytes(&mut invalid_ct);
    let result = MlKem768::decapsulate(&sk, &invalid_ct);
    assert!(matches!(result, Err(KEMError::DecapsulationError)));
}

proptest! {
    #[test]
    fn test_mlkem_random_keys(
        pk_bytes in prop::collection::vec(0u8..255, MlKem768::PUBLIC_KEY_SIZE),
        ct_bytes in prop::collection::vec(0u8..255, MlKem768::CIPHERTEXT_SIZE)
    ) {
        // Ensure we can handle random/malformed inputs without panicking
        let pk = MlKem768::PublicKey::from_bytes(&pk_bytes).unwrap_or_else(|_| panic!("Failed to create public key"));
        let ct = MlKem768::Ciphertext::from_bytes(&ct_bytes).unwrap_or_else(|_| panic!("Failed to create ciphertext"));
        
        // Attempt encapsulation with random public key
        let _ = MlKem768::encapsulate(&pk);
    }
}