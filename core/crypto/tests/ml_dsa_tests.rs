use qudag_crypto::signature::{SignatureError, DigitalSignature};
use qudag_crypto::ml_dsa::MlDsa44;
use proptest::prelude::*;

#[test]
fn test_mldsa_key_generation() {
    let (pk, sk) = MlDsa44::keygen().expect("Key generation should succeed");
    assert!(!pk.as_bytes().is_empty());
    assert!(!sk.as_bytes().is_empty());
}

#[test]
fn test_mldsa_sign_verify() {
    let message = b"Test message for ML-DSA signature";
    let (pk, sk) = MlDsa44::keygen().expect("Key generation should succeed");
    
    let signature = MlDsa44::sign(&sk, message).expect("Signing should succeed");
    let verification = MlDsa44::verify(&pk, message, &signature);
    assert!(verification.is_ok());
}

#[test]
fn test_mldsa_invalid_signature() {
    let message = b"Test message for ML-DSA signature";
    let (pk, _) = MlDsa44::keygen().expect("Key generation should succeed");
    
    let mut invalid_signature = vec![0u8; MlDsa44::SIGNATURE_SIZE];
    rand::thread_rng().fill_bytes(&mut invalid_signature);
    
    let verification = MlDsa44::verify(&pk, message, &invalid_signature.into());
    assert!(matches!(verification, Err(SignatureError::VerificationError)));
}

#[test]
fn test_mldsa_message_tampering() {
    let message = b"Original message";
    let tampered_message = b"Tampered message";
    let (pk, sk) = MlDsa44::keygen().expect("Key generation should succeed");
    
    let signature = MlDsa44::sign(&sk, message).expect("Signing should succeed");
    let verification = MlDsa44::verify(&pk, tampered_message, &signature);
    assert!(matches!(verification, Err(SignatureError::VerificationError)));
}

proptest! {
    #[test]
    fn test_mldsa_random_inputs(
        message in prop::collection::vec(any::<u8>(), 1..1000),
        pk_bytes in prop::collection::vec(0u8..255, MlDsa44::PUBLIC_KEY_SIZE),
        sig_bytes in prop::collection::vec(0u8..255, MlDsa44::SIGNATURE_SIZE)
    ) {
        // Ensure we can handle random/malformed inputs without panicking
        let pk = MlDsa44::PublicKey::from_bytes(&pk_bytes).unwrap_or_else(|_| panic!("Failed to create public key"));
        let sig = MlDsa44::Signature::from_bytes(&sig_bytes).unwrap_or_else(|_| panic!("Failed to create signature"));
        
        // Attempt verification with random inputs
        let _ = MlDsa44::verify(&pk, &message, &sig);
    }
}