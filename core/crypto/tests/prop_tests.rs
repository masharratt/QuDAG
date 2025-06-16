use proptest::prelude::*;
use qudag_crypto::{kem, signatures, encryption, fingerprint::Fingerprint};
use rand::rngs::{StdRng, SeedableRng};

// ML-KEM property tests
prop_compose! {
    fn arb_kem_keypair()(mut rng in any::<[u8; 32]>()) -> Result<kem::KeyPair, kem::KEMError> {
        let mut rng = StdRng::from_seed(rng);
        kem::generate_keypair(&mut rng)
    }
}

proptest! {
    #[test]
    fn test_kem_roundtrip(keypair in arb_kem_keypair()) {
        let keypair = keypair.unwrap();
        let (shared_secret1, ciphertext) = kem::encapsulate(&keypair.public_key).unwrap();
        let shared_secret2 = kem::decapsulate(&keypair.secret_key, &ciphertext).unwrap();
        
        prop_assert!(bool::from(kem::constant_time_compare(&shared_secret1, &shared_secret2)));
    }

    #[test]
    fn test_kem_key_uniqueness(
        keypair1 in arb_kem_keypair(),
        keypair2 in arb_kem_keypair()
    ) {
        let keypair1 = keypair1.unwrap();
        let keypair2 = keypair2.unwrap();
        
        // Different keys should be generated
        prop_assert_ne!(keypair1.public_key, keypair2.public_key);
        prop_assert_ne!(keypair1.secret_key, keypair2.secret_key);
        
        // Cross encapsulation/decapsulation should fail
        let (_, ct1) = kem::encapsulate(&keypair1.public_key).unwrap();
        let (_, ct2) = kem::encapsulate(&keypair2.public_key).unwrap();
        
        let result1 = kem::decapsulate(&keypair2.secret_key, &ct1);
        let result2 = kem::decapsulate(&keypair1.secret_key, &ct2);
        
        prop_assert!(result1.is_err() || result2.is_err());
    }
}

// ML-DSA property tests
prop_compose! {
    fn arb_dsa_keypair()(mut rng in any::<[u8; 32]>()) -> Result<signatures::KeyPair, signatures::SignatureError> {
        let mut rng = StdRng::from_seed(rng);
        signatures::generate_keypair(&mut rng)
    }
}

proptest! {
    #[test]
    fn test_signature_roundtrip(
        keypair in arb_dsa_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..1024)
    ) {
        let keypair = keypair.unwrap();
        let signature = signatures::sign(&keypair.secret_key, &message).unwrap();
        let is_valid = signatures::verify(&keypair.public_key, &message, &signature).unwrap();
        prop_assert!(is_valid);
    }

    #[test]
    fn test_signature_tampering(
        keypair in arb_dsa_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..1024),
        tamper_index in 0usize..1024,
        tamper_byte in any::<u8>()
    ) {
        let keypair = keypair.unwrap();
        let mut signature = signatures::sign(&keypair.secret_key, &message).unwrap();
        
        if tamper_index < signature.len() {
            signature[tamper_index] ^= tamper_byte;
            let is_valid = signatures::verify(&keypair.public_key, &message, &signature).unwrap();
            prop_assert!(!is_valid);
        }
    }
}

// HQC encryption property tests
prop_compose! {
    fn arb_enc_keypair()(mut rng in any::<[u8; 32]>()) -> Result<encryption::KeyPair, encryption::EncryptionError> {
        let mut rng = StdRng::from_seed(rng);
        encryption::generate_keypair(&mut rng)
    }
}

proptest! {
    #[test]
    fn test_encryption_roundtrip(
        keypair in arb_enc_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..32)
    ) {
        let keypair = keypair.unwrap();
        let mut rng = thread_rng();
        
        let ciphertext = encryption::encrypt(&mut rng, &keypair.public_key, &message).unwrap();
        let decrypted = encryption::decrypt(&keypair.secret_key, &ciphertext).unwrap();
        
        prop_assert_eq!(message, decrypted);
    }

    #[test]
    fn test_encryption_tampering(
        keypair in arb_enc_keypair(),
        message in prop::collection::vec(any::<u8>(), 1..32),
        tamper_index in 0usize..1024,
        tamper_byte in any::<u8>()
    ) {
        let keypair = keypair.unwrap();
        let mut rng = thread_rng();
        
        let mut ciphertext = encryption::encrypt(&mut rng, &keypair.public_key, &message).unwrap();
        
        if tamper_index < ciphertext.len() {
            ciphertext[tamper_index] ^= tamper_byte;
            
            match encryption::decrypt(&keypair.secret_key, &ciphertext) {
                Ok(decrypted) => prop_assert_ne!(message, decrypted),
                Err(_) => prop_assert!(true), // Error is also acceptable
            }
        }
    }
}

// Fingerprint property tests
proptest! {
    #[test]
    fn test_fingerprint_properties(
        data in prop::collection::vec(any::<u8>(), 0..1024)
    ) {
        let mut rng = rand::thread_rng();
        
        // Property: Can generate and verify fingerprint for any data
        let (fingerprint, public_key) = Fingerprint::generate(&data, &mut rng).unwrap();
        prop_assert!(fingerprint.verify(&public_key).is_ok());
        
        // Property: Different runs produce different fingerprints
        let (fingerprint2, _) = Fingerprint::generate(&data, &mut rng).unwrap();
        prop_assert_ne!(fingerprint.data(), fingerprint2.data());
        
        // Property: Fingerprint data length is consistent
        prop_assert_eq!(fingerprint.data().len(), 64);
    }
    
    #[test]
    fn test_fingerprint_verification_properties(
        data1 in prop::collection::vec(any::<u8>(), 0..1024),
        data2 in prop::collection::vec(any::<u8>(), 0..1024)
    ) {
        let mut rng = rand::thread_rng();
        
        // Generate two fingerprints
        let (fp1, key1) = Fingerprint::generate(&data1, &mut rng).unwrap();
        let (fp2, key2) = Fingerprint::generate(&data2, &mut rng).unwrap();
        
        // Property: Cannot verify fingerprint with wrong key
        if data1 != data2 {
            prop_assert!(fp1.verify(&key2).is_err());
            prop_assert!(fp2.verify(&key1).is_err());
        }
    }
}