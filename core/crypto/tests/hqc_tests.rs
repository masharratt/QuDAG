use qudag_crypto::encryption::{AsymmetricEncryption, EncryptionError};
use qudag_crypto::hqc::Hqc256;
use proptest::prelude::*;

#[test]
fn test_hqc_key_generation() {
    let (pk, sk) = Hqc256::keygen().expect("Key generation should succeed");
    assert!(!pk.as_bytes().is_empty());
    assert!(!sk.as_bytes().is_empty());
}

#[test]
fn test_hqc_encryption_decryption() {
    let message = b"Test message for HQC encryption";
    let (pk, sk) = Hqc256::keygen().expect("Key generation should succeed");
    
    let ciphertext = Hqc256::encrypt(&pk, message).expect("Encryption should succeed");
    let decrypted = Hqc256::decrypt(&sk, &ciphertext).expect("Decryption should succeed");
    
    assert_eq!(message, decrypted.as_slice());
}

#[test]
fn test_hqc_invalid_ciphertext() {
    let (_, sk) = Hqc256::keygen().expect("Key generation should succeed");
    let mut invalid_ciphertext = vec![0u8; Hqc256::CIPHERTEXT_SIZE];
    rand::thread_rng().fill_bytes(&mut invalid_ciphertext);
    
    let result = Hqc256::decrypt(&sk, &invalid_ciphertext);
    assert!(matches!(result, Err(EncryptionError::DecryptionError)));
}

#[test]
fn test_hqc_long_message() {
    let message = vec![0u8; 1024];
    let (pk, sk) = Hqc256::keygen().expect("Key generation should succeed");
    
    let ciphertext = Hqc256::encrypt(&pk, &message).expect("Encryption should succeed");
    let decrypted = Hqc256::decrypt(&sk, &ciphertext).expect("Decryption should succeed");
    
    assert_eq!(message, decrypted);
}

proptest! {
    #[test]
    fn test_hqc_random_keys_and_messages(
        message in prop::collection::vec(any::<u8>(), 1..1000),
        pk_bytes in prop::collection::vec(0u8..255, Hqc256::PUBLIC_KEY_SIZE),
        ct_bytes in prop::collection::vec(0u8..255, Hqc256::CIPHERTEXT_SIZE)
    ) {
        // Ensure we can handle random/malformed inputs without panicking
        let pk = Hqc256::PublicKey::from_bytes(&pk_bytes).unwrap_or_else(|_| panic!("Failed to create public key"));
        
        // Attempt encryption with random public key
        let _ = Hqc256::encrypt(&pk, &message);
    }
}