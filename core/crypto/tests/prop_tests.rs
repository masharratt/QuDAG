use proptest::prelude::*;
use qudag_crypto::kem::ml_kem::*;

prop_compose! {
    fn arb_keypair()(mut rng in any::<[u8; 32]>()) -> Result<KeyPair, KEMError> {
        let mut rng = rand::rngs::StdRng::from_seed(rng);
        generate_keypair(&mut rng)
    }
}

proptest! {
    #[test]
    fn test_encapsulation_decapsulation_roundtrip(keypair in arb_keypair()) {
        let keypair = keypair.unwrap();
        let (shared_secret1, ciphertext) = encapsulate(&keypair.public_key).unwrap();
        let shared_secret2 = decapsulate(&keypair.secret_key, &ciphertext).unwrap();
        
        assert!(bool::from(constant_time_compare(&shared_secret1, &shared_secret2)));
    }

    #[test]
    fn test_key_uniqueness(
        keypair1 in arb_keypair(),
        keypair2 in arb_keypair()
    ) {
        let keypair1 = keypair1.unwrap();
        let keypair2 = keypair2.unwrap();
        
        // Different keys should be generated
        prop_assert_ne!(keypair1.public_key, keypair2.public_key);
        prop_assert_ne!(keypair1.secret_key, keypair2.secret_key);
        
        // Cross encapsulation/decapsulation should fail
        let (_, ct1) = encapsulate(&keypair1.public_key).unwrap();
        let (_, ct2) = encapsulate(&keypair2.public_key).unwrap();
        
        prop_assert!(decapsulate(&keypair2.secret_key, &ct1).is_err());
        prop_assert!(decapsulate(&keypair1.secret_key, &ct2).is_err());
    }

    #[test]
    fn test_invalid_inputs(
        invalid_pk in prop::collection::vec(any::<u8>(), 0..1024),
        invalid_sk in prop::collection::vec(any::<u8>(), 0..1024),
        invalid_ct in prop::collection::vec(any::<u8>(), 0..1024),
    ) {
        if invalid_pk.len() != PUBLIC_KEY_BYTES {
            prop_assert!(encapsulate(&invalid_pk).is_err());
        }
        
        if invalid_sk.len() != SECRET_KEY_BYTES || invalid_ct.len() != CIPHERTEXT_BYTES {
            prop_assert!(decapsulate(&invalid_sk, &invalid_ct).is_err());
        }
    }
}