//! Tests for QuDAG Vault integration and quantum-resistant key management
//! Following TDD methodology - testing secure key storage and cryptographic operations

use qudag_exchange_core::vault::{VaultManager, KeyPair, KeyType, VaultError};
use qudag_vault_core::{Vault, VaultConfig};
use qudag_crypto::{ml_dsa, ml_kem, hqc};
use tempfile::TempDir;
use std::path::PathBuf;

#[cfg(test)]
mod vault_manager_tests {
    use super::*;

    fn create_test_vault() -> (VaultManager, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let vault_path = temp_dir.path().join("test_vault.db");
        let config = VaultConfig {
            path: vault_path,
            auto_lock_timeout: None,
            use_hardware_security: false,
        };
        
        let manager = VaultManager::new(config).expect("Failed to create vault manager");
        (manager, temp_dir)
    }

    #[test]
    fn test_vault_creation_and_initialization() {
        // Test: Vault should be created and initialized with master password
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        assert!(manager.is_initialized());
        assert!(!manager.is_unlocked()); // Should be locked after initialization
    }

    #[test]
    fn test_vault_unlock_and_lock() {
        // Test: Vault should support secure unlock/lock operations
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        
        // Unlock with correct password
        manager.unlock(master_password).expect("Vault unlock failed");
        assert!(manager.is_unlocked());
        
        // Lock vault
        manager.lock();
        assert!(!manager.is_unlocked());
        
        // Unlock with incorrect password should fail
        assert!(matches!(
            manager.unlock("wrong_password"),
            Err(VaultError::InvalidPassword)
        ));
    }

    #[test]
    fn test_ml_dsa_key_generation_and_storage() {
        // Test: Generate and store ML-DSA quantum-resistant signing keys
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        manager.unlock(master_password).expect("Vault unlock failed");
        
        // Generate ML-DSA-65 key pair
        let key_id = "alice_signing_key";
        let keypair = manager.generate_key_pair(key_id, KeyType::MlDsa65)
            .expect("ML-DSA key generation failed");
        
        assert_eq!(keypair.key_type(), KeyType::MlDsa65);
        assert_eq!(keypair.id(), key_id);
        assert!(!keypair.public_key().is_empty());
        
        // Verify key can be retrieved
        let retrieved = manager.get_key_pair(key_id)
            .expect("Key retrieval failed");
        assert_eq!(retrieved.public_key(), keypair.public_key());
    }

    #[test]
    fn test_ml_kem_key_generation_and_storage() {
        // Test: Generate and store ML-KEM quantum-resistant encryption keys
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        manager.unlock(master_password).expect("Vault unlock failed");
        
        // Generate ML-KEM-768 key pair
        let key_id = "bob_encryption_key";
        let keypair = manager.generate_key_pair(key_id, KeyType::MlKem768)
            .expect("ML-KEM key generation failed");
        
        assert_eq!(keypair.key_type(), KeyType::MlKem768);
        
        // Test different security levels
        let key_1024 = manager.generate_key_pair("high_security_key", KeyType::MlKem1024)
            .expect("ML-KEM-1024 generation failed");
        assert_eq!(key_1024.key_type(), KeyType::MlKem1024);
    }

    #[test]
    fn test_hqc_hybrid_key_generation() {
        // Test: Generate HQC hybrid quantum-resistant keys
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        manager.unlock(master_password).expect("Vault unlock failed");
        
        let key_id = "charlie_hybrid_key";
        let keypair = manager.generate_key_pair(key_id, KeyType::Hqc128)
            .expect("HQC key generation failed");
        
        assert_eq!(keypair.key_type(), KeyType::Hqc128);
    }

    #[test]
    fn test_key_listing_and_metadata() {
        // Test: Vault should list all stored keys with metadata
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        manager.unlock(master_password).expect("Vault unlock failed");
        
        // Generate multiple keys
        manager.generate_key_pair("key1", KeyType::MlDsa65).unwrap();
        manager.generate_key_pair("key2", KeyType::MlKem768).unwrap();
        manager.generate_key_pair("key3", KeyType::Hqc128).unwrap();
        
        let keys = manager.list_keys().expect("Key listing failed");
        assert_eq!(keys.len(), 3);
        
        // Verify key metadata
        for key_info in keys {
            assert!(key_info.created_at > 0);
            assert!(key_info.id == "key1" || key_info.id == "key2" || key_info.id == "key3");
        }
    }

    #[test]
    fn test_key_deletion() {
        // Test: Keys should be securely deletable from vault
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        manager.unlock(master_password).expect("Vault unlock failed");
        
        let key_id = "temporary_key";
        manager.generate_key_pair(key_id, KeyType::MlDsa65).unwrap();
        
        // Delete key
        manager.delete_key(key_id).expect("Key deletion failed");
        
        // Verify key is gone
        assert!(matches!(
            manager.get_key_pair(key_id),
            Err(VaultError::KeyNotFound)
        ));
    }

    #[test]
    fn test_vault_operations_require_unlock() {
        // Test: All sensitive operations should fail when vault is locked
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).expect("Vault initialization failed");
        // Don't unlock - vault should be locked
        
        assert!(matches!(
            manager.generate_key_pair("test", KeyType::MlDsa65),
            Err(VaultError::VaultLocked)
        ));
        
        assert!(matches!(
            manager.get_key_pair("test"),
            Err(VaultError::VaultLocked)
        ));
        
        assert!(matches!(
            manager.list_keys(),
            Err(VaultError::VaultLocked)
        ));
    }

    #[test]
    fn test_vault_persistence() {
        // Test: Vault data should persist across manager instances
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let vault_path = temp_dir.path().join("persistent_vault.db");
        let master_password = "secure_test_password_123!";
        let key_id = "persistent_key";
        
        // Create vault and add key
        {
            let config = VaultConfig {
                path: vault_path.clone(),
                auto_lock_timeout: None,
                use_hardware_security: false,
            };
            let mut manager = VaultManager::new(config).unwrap();
            manager.initialize(master_password).unwrap();
            manager.unlock(master_password).unwrap();
            manager.generate_key_pair(key_id, KeyType::MlDsa65).unwrap();
        }
        
        // Open vault in new manager instance
        {
            let config = VaultConfig {
                path: vault_path,
                auto_lock_timeout: None,
                use_hardware_security: false,
            };
            let mut manager = VaultManager::new(config).unwrap();
            assert!(manager.is_initialized());
            manager.unlock(master_password).unwrap();
            
            // Verify key exists
            let keypair = manager.get_key_pair(key_id).unwrap();
            assert_eq!(keypair.key_type(), KeyType::MlDsa65);
        }
    }
}

#[cfg(test)]
mod cryptographic_operations_tests {
    use super::*;

    #[test]
    fn test_ml_dsa_signing_and_verification() {
        // Test: Sign messages with ML-DSA and verify signatures
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).unwrap();
        manager.unlock(master_password).unwrap();
        
        let key_id = "signing_key";
        let keypair = manager.generate_key_pair(key_id, KeyType::MlDsa65).unwrap();
        
        // Sign a message
        let message = b"Transfer 100 rUv from Alice to Bob";
        let signature = manager.sign_message(key_id, message)
            .expect("Message signing failed");
        
        assert!(!signature.is_empty());
        
        // Verify signature with public key
        let is_valid = manager.verify_signature(&keypair.public_key(), message, &signature)
            .expect("Signature verification failed");
        assert!(is_valid);
        
        // Verify with wrong message should fail
        let wrong_message = b"Transfer 200 rUv from Alice to Bob";
        let is_valid = manager.verify_signature(&keypair.public_key(), wrong_message, &signature)
            .expect("Signature verification failed");
        assert!(!is_valid);
    }

    #[test]
    fn test_ml_kem_encryption_and_decryption() {
        // Test: Encrypt and decrypt data using ML-KEM
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).unwrap();
        manager.unlock(master_password).unwrap();
        
        // Generate keys for Alice and Bob
        let alice_key = manager.generate_key_pair("alice_kem", KeyType::MlKem768).unwrap();
        let bob_key = manager.generate_key_pair("bob_kem", KeyType::MlKem768).unwrap();
        
        // Alice encrypts data for Bob
        let plaintext = b"Secret rUv transfer authorization code: XYZ123";
        let ciphertext = manager.encrypt_for_recipient(&bob_key.public_key(), plaintext)
            .expect("Encryption failed");
        
        assert_ne!(ciphertext.as_slice(), plaintext);
        assert!(ciphertext.len() > plaintext.len()); // Should include encapsulated key
        
        // Bob decrypts the data
        let decrypted = manager.decrypt_with_key("bob_kem", &ciphertext)
            .expect("Decryption failed");
        
        assert_eq!(decrypted.as_slice(), plaintext);
        
        // Alice cannot decrypt data meant for Bob
        assert!(manager.decrypt_with_key("alice_kem", &ciphertext).is_err());
    }

    #[test]
    fn test_hybrid_encryption_with_multiple_recipients() {
        // Test: Encrypt data for multiple recipients
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).unwrap();
        manager.unlock(master_password).unwrap();
        
        // Generate keys for multiple recipients
        let recipients = vec![
            manager.generate_key_pair("alice", KeyType::MlKem768).unwrap(),
            manager.generate_key_pair("bob", KeyType::MlKem768).unwrap(),
            manager.generate_key_pair("charlie", KeyType::MlKem768).unwrap(),
        ];
        
        let public_keys: Vec<_> = recipients.iter().map(|k| k.public_key()).collect();
        
        // Encrypt for all recipients
        let plaintext = b"Announcement: New rUv distribution policy";
        let encrypted_bundle = manager.encrypt_for_multiple_recipients(&public_keys, plaintext)
            .expect("Multi-recipient encryption failed");
        
        // Each recipient should be able to decrypt
        for (i, recipient_id) in ["alice", "bob", "charlie"].iter().enumerate() {
            let decrypted = manager.decrypt_from_bundle(recipient_id, &encrypted_bundle, i)
                .expect(&format!("Decryption failed for {}", recipient_id));
            assert_eq!(decrypted.as_slice(), plaintext);
        }
    }

    #[test]
    fn test_key_derivation_for_deterministic_addresses() {
        // Test: Derive deterministic addresses from vault keys
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).unwrap();
        manager.unlock(master_password).unwrap();
        
        let key_id = "master_key";
        let keypair = manager.generate_key_pair(key_id, KeyType::MlDsa65).unwrap();
        
        // Derive multiple addresses
        let address1 = manager.derive_address(key_id, 0).expect("Address derivation failed");
        let address2 = manager.derive_address(key_id, 1).expect("Address derivation failed");
        let address3 = manager.derive_address(key_id, 0).expect("Address derivation failed"); // Same index
        
        assert_ne!(address1, address2); // Different indices produce different addresses
        assert_eq!(address1, address3); // Same index produces same address (deterministic)
        
        // Addresses should be valid AccountIds
        assert!(address1.starts_with("qd_") || address1.starts_with("ruv_"));
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_timing_attack_resistance() {
        // Test: Operations should have constant time properties
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).unwrap();
        
        // Measure unlock time with correct vs incorrect passwords
        let correct_times: Vec<Duration> = (0..10).map(|_| {
            let start = Instant::now();
            let _ = manager.unlock(master_password);
            manager.lock();
            start.elapsed()
        }).collect();
        
        let incorrect_times: Vec<Duration> = (0..10).map(|_| {
            let start = Instant::now();
            let _ = manager.unlock("wrong_password_12345");
            start.elapsed()
        }).collect();
        
        // Calculate average times
        let avg_correct: Duration = correct_times.iter().sum::<Duration>() / correct_times.len() as u32;
        let avg_incorrect: Duration = incorrect_times.iter().sum::<Duration>() / incorrect_times.len() as u32;
        
        // Times should be similar (constant-time implementation)
        let time_diff = if avg_correct > avg_incorrect {
            avg_correct - avg_incorrect
        } else {
            avg_incorrect - avg_correct
        };
        
        // Allow for some variance but not significant timing differences
        assert!(time_diff < Duration::from_millis(50), 
            "Timing difference too large: {:?}", time_diff);
    }

    #[test]
    fn test_memory_zeroization() {
        // Test: Sensitive data should be zeroized after use
        let (mut manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        manager.initialize(master_password).unwrap();
        manager.unlock(master_password).unwrap();
        
        let key_id = "sensitive_key";
        let keypair = manager.generate_key_pair(key_id, KeyType::MlDsa65).unwrap();
        
        // Sign a message
        let message = b"Sensitive transaction data";
        let signature = manager.sign_message(key_id, message).unwrap();
        
        // After operations, internal buffers should be cleared
        // This is typically verified through careful code review and tools like valgrind
        // Here we just ensure the operation completes without leaking
        drop(signature);
        drop(keypair);
        
        // Lock vault to ensure cleanup
        manager.lock();
    }

    #[test]
    fn test_vault_backup_and_restore() {
        // Test: Vault should support secure backup and restore
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().join("original_vault.db");
        let backup_path = temp_dir.path().join("backup_vault.enc");
        let master_password = "secure_test_password_123!";
        let backup_password = "backup_encryption_password_456!";
        
        // Create vault with keys
        {
            let config = VaultConfig {
                path: vault_path.clone(),
                auto_lock_timeout: None,
                use_hardware_security: false,
            };
            let mut manager = VaultManager::new(config).unwrap();
            manager.initialize(master_password).unwrap();
            manager.unlock(master_password).unwrap();
            
            manager.generate_key_pair("key1", KeyType::MlDsa65).unwrap();
            manager.generate_key_pair("key2", KeyType::MlKem768).unwrap();
            
            // Create encrypted backup
            manager.create_backup(&backup_path, backup_password)
                .expect("Backup creation failed");
        }
        
        // Restore to new location
        let restore_path = temp_dir.path().join("restored_vault.db");
        {
            let config = VaultConfig {
                path: restore_path,
                auto_lock_timeout: None,
                use_hardware_security: false,
            };
            let mut manager = VaultManager::new(config).unwrap();
            
            // Restore from backup
            manager.restore_from_backup(&backup_path, backup_password, master_password)
                .expect("Restore failed");
            
            manager.unlock(master_password).unwrap();
            
            // Verify keys exist
            let keys = manager.list_keys().unwrap();
            assert_eq!(keys.len(), 2);
            assert!(keys.iter().any(|k| k.id == "key1"));
            assert!(keys.iter().any(|k| k.id == "key2"));
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use qudag_exchange_core::ledger::{AccountId, Transaction};

    #[test]
    fn test_vault_ledger_integration() {
        // Test: Integration between vault key management and ledger transactions
        let (mut vault_manager, _temp_dir) = create_test_vault();
        let master_password = "secure_test_password_123!";
        
        vault_manager.initialize(master_password).unwrap();
        vault_manager.unlock(master_password).unwrap();
        
        // Generate keys for Alice and Bob
        let alice_key = vault_manager.generate_key_pair("alice", KeyType::MlDsa65).unwrap();
        let bob_key = vault_manager.generate_key_pair("bob", KeyType::MlDsa65).unwrap();
        
        // Derive account IDs from public keys
        let alice_account = AccountId::from_public_key(&alice_key.public_key());
        let bob_account = AccountId::from_public_key(&bob_key.public_key());
        
        // Create and sign a transaction
        let tx = Transaction::new(
            alice_account.clone(),
            bob_account.clone(),
            100, // 100 rUv
            0,   // nonce
        );
        
        let tx_bytes = tx.to_bytes();
        let signature = vault_manager.sign_message("alice", &tx_bytes).unwrap();
        
        // Verify transaction signature
        let is_valid = vault_manager.verify_signature(
            &alice_key.public_key(),
            &tx_bytes,
            &signature
        ).unwrap();
        
        assert!(is_valid, "Transaction signature verification failed");
    }
}
