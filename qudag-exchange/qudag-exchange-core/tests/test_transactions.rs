//! Tests for transaction structure, serialization, signing, and verification
//! Following TDD methodology - defining transaction behavior before implementation

use qudag_exchange_core::transaction::{
    Transaction, TransactionType, TransactionStatus, TransactionError,
    TransactionBuilder, TransactionHash, SignedTransaction
};
use qudag_exchange_core::ledger::{AccountId, Balance};
use qudag_crypto::ml_dsa::{PublicKey, SecretKey};
use blake3::Hasher;
use proptest::prelude::*;
use serde_json;
use bincode;

#[cfg(test)]
mod transaction_structure_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_transaction_creation() {
        // Test: Basic transaction creation with all required fields
        let sender = AccountId::from_string("alice");
        let recipient = AccountId::from_string("bob");
        let amount = Balance::from_ruv(100);
        let nonce = 0u64;
        
        let tx = Transaction::new(
            sender.clone(),
            recipient.clone(),
            amount,
            nonce,
        );
        
        assert_eq!(tx.sender(), &sender);
        assert_eq!(tx.recipient(), &recipient);
        assert_eq!(tx.amount(), amount);
        assert_eq!(tx.nonce(), nonce);
        assert!(tx.timestamp() > 0);
        assert_eq!(tx.tx_type(), TransactionType::Transfer);
    }

    #[test]
    fn test_transaction_builder_pattern() {
        // Test: Builder pattern for complex transaction creation
        let sender = AccountId::from_string("alice");
        let recipient = AccountId::from_string("bob");
        
        let tx = TransactionBuilder::new()
            .sender(sender.clone())
            .recipient(recipient.clone())
            .amount(Balance::from_ruv(100))
            .nonce(5)
            .fee(Balance::from_ruv(1))
            .memo("Payment for services")
            .expiry(SystemTime::now() + std::time::Duration::from_secs(3600))
            .build()
            .expect("Transaction building failed");
        
        assert_eq!(tx.sender(), &sender);
        assert_eq!(tx.fee(), Balance::from_ruv(1));
        assert_eq!(tx.memo(), Some("Payment for services"));
        assert!(tx.is_valid_at(SystemTime::now()));
    }

    #[test]
    fn test_transaction_types() {
        // Test: Different transaction types for various operations
        let account = AccountId::from_string("alice");
        
        // Transfer transaction
        let transfer_tx = TransactionBuilder::new()
            .tx_type(TransactionType::Transfer)
            .sender(account.clone())
            .recipient(AccountId::from_string("bob"))
            .amount(Balance::from_ruv(100))
            .build()
            .unwrap();
        assert_eq!(transfer_tx.tx_type(), TransactionType::Transfer);
        
        // Vault operation transaction
        let vault_tx = TransactionBuilder::new()
            .tx_type(TransactionType::VaultOperation)
            .sender(account.clone())
            .vault_action("create_backup")
            .build()
            .unwrap();
        assert_eq!(vault_tx.tx_type(), TransactionType::VaultOperation);
        
        // Dark domain registration
        let domain_tx = TransactionBuilder::new()
            .tx_type(TransactionType::DarkDomainRegistration)
            .sender(account.clone())
            .domain_name("alice.dark")
            .quantum_fingerprint("QF:ABCD:1234")
            .build()
            .unwrap();
        assert_eq!(domain_tx.tx_type(), TransactionType::DarkDomainRegistration);
        
        // Resource metering
        let metering_tx = TransactionBuilder::new()
            .tx_type(TransactionType::ResourceMetering)
            .sender(account.clone())
            .resource_type("compute")
            .resource_units(1000)
            .build()
            .unwrap();
        assert_eq!(metering_tx.tx_type(), TransactionType::ResourceMetering);
    }

    #[test]
    fn test_transaction_validation() {
        // Test: Transaction validation rules
        let sender = AccountId::from_string("alice");
        let recipient = AccountId::from_string("bob");
        
        // Valid transaction
        let valid_tx = Transaction::new(
            sender.clone(),
            recipient.clone(),
            Balance::from_ruv(100),
            0,
        );
        assert!(valid_tx.validate().is_ok());
        
        // Invalid: sender equals recipient
        let self_transfer = Transaction::new(
            sender.clone(),
            sender.clone(),
            Balance::from_ruv(100),
            0,
        );
        assert!(matches!(
            self_transfer.validate(),
            Err(TransactionError::InvalidRecipient)
        ));
        
        // Invalid: zero amount
        let zero_amount = Transaction::new(
            sender.clone(),
            recipient.clone(),
            Balance::zero(),
            0,
        );
        assert!(matches!(
            zero_amount.validate(),
            Err(TransactionError::InvalidAmount)
        ));
        
        // Invalid: expired transaction
        let expired_tx = TransactionBuilder::new()
            .sender(sender)
            .recipient(recipient)
            .amount(Balance::from_ruv(100))
            .expiry(SystemTime::now() - std::time::Duration::from_secs(1))
            .build()
            .unwrap();
        assert!(matches!(
            expired_tx.validate(),
            Err(TransactionError::Expired)
        ));
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_transaction_json_serialization() {
        // Test: Transactions should be JSON serializable
        let tx = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("bob"),
            Balance::from_ruv(100),
            42,
        );
        
        // Serialize to JSON
        let json = serde_json::to_string_pretty(&tx).expect("JSON serialization failed");
        assert!(json.contains("\"sender\""));
        assert!(json.contains("\"recipient\""));
        assert!(json.contains("\"amount\""));
        assert!(json.contains("\"nonce\""));
        
        // Deserialize from JSON
        let deserialized: Transaction = serde_json::from_str(&json)
            .expect("JSON deserialization failed");
        
        assert_eq!(tx.sender(), deserialized.sender());
        assert_eq!(tx.recipient(), deserialized.recipient());
        assert_eq!(tx.amount(), deserialized.amount());
        assert_eq!(tx.nonce(), deserialized.nonce());
        assert_eq!(tx.timestamp(), deserialized.timestamp());
    }

    #[test]
    fn test_transaction_bincode_serialization() {
        // Test: Binary serialization for efficient storage/transmission
        let tx = TransactionBuilder::new()
            .sender(AccountId::from_string("alice"))
            .recipient(AccountId::from_string("bob"))
            .amount(Balance::from_ruv(100))
            .nonce(10)
            .fee(Balance::from_ruv(1))
            .memo("Test transaction")
            .build()
            .unwrap();
        
        // Serialize to binary
        let bytes = bincode::serialize(&tx).expect("Bincode serialization failed");
        assert!(bytes.len() < 1000); // Reasonable size for a transaction
        
        // Deserialize from binary
        let deserialized: Transaction = bincode::deserialize(&bytes)
            .expect("Bincode deserialization failed");
        
        assert_eq!(tx, deserialized);
    }

    #[test]
    fn test_canonical_serialization() {
        // Test: Canonical serialization for consistent hashing
        let tx = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("bob"),
            Balance::from_ruv(100),
            0,
        );
        
        // Multiple serializations should produce identical bytes
        let bytes1 = tx.to_canonical_bytes();
        let bytes2 = tx.to_canonical_bytes();
        assert_eq!(bytes1, bytes2);
        
        // Order of fields shouldn't matter for canonical form
        let tx2 = TransactionBuilder::new()
            .nonce(0) // Set fields in different order
            .amount(Balance::from_ruv(100))
            .recipient(AccountId::from_string("bob"))
            .sender(AccountId::from_string("alice"))
            .timestamp(tx.timestamp()) // Use same timestamp
            .build()
            .unwrap();
        
        let bytes3 = tx2.to_canonical_bytes();
        assert_eq!(bytes1, bytes3);
    }
}

#[cfg(test)]
mod hashing_tests {
    use super::*;

    #[test]
    fn test_transaction_hash_computation() {
        // Test: Transaction hashing using BLAKE3
        let tx = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("bob"),
            Balance::from_ruv(100),
            0,
        );
        
        let hash = tx.hash();
        
        // Hash should be 32 bytes (256 bits)
        assert_eq!(hash.as_bytes().len(), 32);
        
        // Hash should be deterministic
        let hash2 = tx.hash();
        assert_eq!(hash, hash2);
        
        // Different transactions should have different hashes
        let tx2 = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("charlie"),
            Balance::from_ruv(100),
            0,
        );
        assert_ne!(tx.hash(), tx2.hash());
    }

    #[test]
    fn test_transaction_hash_display() {
        // Test: Transaction hash should have hex display format
        let tx = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("bob"),
            Balance::from_ruv(100),
            0,
        );
        
        let hash = tx.hash();
        let hash_str = hash.to_string();
        
        // Should be 64 hex characters (32 bytes * 2)
        assert_eq!(hash_str.len(), 64);
        assert!(hash_str.chars().all(|c| c.is_ascii_hexdigit()));
        
        // Should be parseable back
        let parsed = TransactionHash::from_hex(&hash_str).expect("Hash parsing failed");
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_merkle_root_computation() {
        // Test: Compute Merkle root for transaction batches
        let transactions: Vec<Transaction> = (0..8)
            .map(|i| Transaction::new(
                AccountId::from_string("alice"),
                AccountId::from_string(&format!("recipient_{}", i)),
                Balance::from_ruv(100),
                i as u64,
            ))
            .collect();
        
        let hashes: Vec<TransactionHash> = transactions.iter()
            .map(|tx| tx.hash())
            .collect();
        
        let merkle_root = TransactionHash::compute_merkle_root(&hashes);
        
        // Merkle root should be deterministic
        let merkle_root2 = TransactionHash::compute_merkle_root(&hashes);
        assert_eq!(merkle_root, merkle_root2);
        
        // Different transaction sets should have different roots
        let mut different_hashes = hashes.clone();
        different_hashes[0] = transactions[1].hash(); // Change first hash
        let different_root = TransactionHash::compute_merkle_root(&different_hashes);
        assert_ne!(merkle_root, different_root);
    }
}

#[cfg(test)]
mod signing_tests {
    use super::*;
    use qudag_exchange_core::vault::VaultManager;
    use tempfile::TempDir;

    fn create_test_keypair() -> (PublicKey, SecretKey) {
        // Mock keypair generation for testing
        // In real implementation, this would use qudag_crypto::ml_dsa
        let secret = SecretKey::generate();
        let public = secret.public_key();
        (public, secret)
    }

    #[test]
    fn test_transaction_signing() {
        // Test: Sign transaction with ML-DSA quantum-resistant signature
        let tx = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("bob"),
            Balance::from_ruv(100),
            0,
        );
        
        let (public_key, secret_key) = create_test_keypair();
        
        // Sign transaction
        let signed_tx = tx.sign(&secret_key).expect("Transaction signing failed");
        
        assert_eq!(signed_tx.transaction(), &tx);
        assert_eq!(signed_tx.public_key(), &public_key);
        assert!(!signed_tx.signature().is_empty());
        
        // Verify signature
        assert!(signed_tx.verify().expect("Signature verification failed"));
    }

    #[test]
    fn test_signature_verification() {
        // Test: Verify transaction signatures
        let tx = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("bob"),
            Balance::from_ruv(100),
            0,
        );
        
        let (public_key, secret_key) = create_test_keypair();
        let signed_tx = tx.sign(&secret_key).unwrap();
        
        // Valid signature should verify
        assert!(signed_tx.verify().unwrap());
        
        // Modified transaction should fail verification
        let mut modified_tx = tx.clone();
        modified_tx.set_amount(Balance::from_ruv(200)); // Tamper with amount
        let tampered_signed = SignedTransaction::new(
            modified_tx,
            signed_tx.signature().clone(),
            public_key.clone(),
        );
        assert!(!tampered_signed.verify().unwrap());
        
        // Wrong public key should fail
        let (wrong_public, _) = create_test_keypair();
        let wrong_key_signed = SignedTransaction::new(
            tx.clone(),
            signed_tx.signature().clone(),
            wrong_public,
        );
        assert!(!wrong_key_signed.verify().unwrap());
    }

    #[test]
    fn test_multi_signature_support() {
        // Test: Support for multi-signature transactions
        let tx = Transaction::new(
            AccountId::from_string("multisig_wallet"),
            AccountId::from_string("bob"),
            Balance::from_ruv(1000),
            0,
        );
        
        // Create multiple signers
        let signers: Vec<(PublicKey, SecretKey)> = (0..3)
            .map(|_| create_test_keypair())
            .collect();
        
        // Each signer signs the transaction
        let signatures: Vec<_> = signers.iter()
            .map(|(_, secret)| tx.create_signature(secret).unwrap())
            .collect();
        
        // Create multi-signed transaction
        let public_keys: Vec<_> = signers.iter().map(|(pub_key, _)| pub_key.clone()).collect();
        let multi_signed = tx.with_multi_signatures(signatures, public_keys, 2); // 2-of-3
        
        // Verify multi-signature
        assert!(multi_signed.verify_multisig().unwrap());
        
        // Should fail with insufficient signatures
        let insufficient_signed = tx.with_multi_signatures(
            signatures[..1].to_vec(), // Only one signature
            public_keys.clone(),
            2, // Requires 2
        );
        assert!(!insufficient_signed.verify_multisig().unwrap());
    }
}

#[cfg(test)]
mod transaction_pool_tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_transaction_ordering() {
        // Test: Transactions should be orderable by fee and timestamp
        let mut tx_pool = BTreeMap::new();
        
        // Create transactions with different fees
        let tx1 = TransactionBuilder::new()
            .sender(AccountId::from_string("alice"))
            .recipient(AccountId::from_string("bob"))
            .amount(Balance::from_ruv(100))
            .fee(Balance::from_ruv(1))
            .build()
            .unwrap();
        
        let tx2 = TransactionBuilder::new()
            .sender(AccountId::from_string("alice"))
            .recipient(AccountId::from_string("charlie"))
            .amount(Balance::from_ruv(100))
            .fee(Balance::from_ruv(5)) // Higher fee
            .build()
            .unwrap();
        
        let tx3 = TransactionBuilder::new()
            .sender(AccountId::from_string("alice"))
            .recipient(AccountId::from_string("dave"))
            .amount(Balance::from_ruv(100))
            .fee(Balance::from_ruv(3))
            .build()
            .unwrap();
        
        // Add to pool (in practice, would use priority queue)
        tx_pool.insert(tx1.priority_score(), tx1.clone());
        tx_pool.insert(tx2.priority_score(), tx2.clone());
        tx_pool.insert(tx3.priority_score(), tx3.clone());
        
        // Should be ordered by fee (descending)
        let ordered: Vec<_> = tx_pool.values().rev().collect();
        assert_eq!(ordered[0].fee(), Balance::from_ruv(5));
        assert_eq!(ordered[1].fee(), Balance::from_ruv(3));
        assert_eq!(ordered[2].fee(), Balance::from_ruv(1));
    }

    #[test]
    fn test_transaction_deduplication() {
        // Test: Duplicate transactions should be detected
        let tx = Transaction::new(
            AccountId::from_string("alice"),
            AccountId::from_string("bob"),
            Balance::from_ruv(100),
            0,
        );
        
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        
        assert_eq!(hash1, hash2); // Same transaction produces same hash
        
        // Transaction pool should reject duplicates
        let mut seen_hashes = std::collections::HashSet::new();
        assert!(seen_hashes.insert(hash1));
        assert!(!seen_hashes.insert(hash2)); // Duplicate detected
    }
}

// Property-based tests
proptest! {
    #[test]
    fn prop_transaction_serialization_roundtrip(
        sender_id in "[a-z]{5,10}",
        recipient_id in "[a-z]{5,10}",
        amount in 1u64..1_000_000,
        nonce in 0u64..1_000,
        fee in 0u64..100
    ) {
        prop_assume!(sender_id != recipient_id);
        
        let tx = TransactionBuilder::new()
            .sender(AccountId::from_string(&sender_id))
            .recipient(AccountId::from_string(&recipient_id))
            .amount(Balance::from_ruv(amount))
            .nonce(nonce)
            .fee(Balance::from_ruv(fee))
            .build()
            .unwrap();
        
        // JSON roundtrip
        let json = serde_json::to_string(&tx).unwrap();
        let from_json: Transaction = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&tx, &from_json);
        
        // Bincode roundtrip
        let bytes = bincode::serialize(&tx).unwrap();
        let from_bytes: Transaction = bincode::deserialize(&bytes).unwrap();
        prop_assert_eq!(&tx, &from_bytes);
    }
    
    #[test]
    fn prop_transaction_hash_uniqueness(
        txs in prop::collection::vec(
            ("[a-z]{5,10}", "[a-z]{5,10}", 1u64..1_000_000, 0u64..1_000),
            1..100
        )
    ) {
        let mut hashes = std::collections::HashSet::new();
        
        for (i, (sender, recipient, amount, nonce)) in txs.into_iter().enumerate() {
            if sender == recipient {
                continue; // Skip invalid transactions
            }
            
            let tx = Transaction::new(
                AccountId::from_string(&sender),
                AccountId::from_string(&recipient),
                Balance::from_ruv(amount),
                nonce + i as u64, // Ensure unique nonces
            );
            
            let hash = tx.hash();
            prop_assert!(hashes.insert(hash), "Duplicate hash found");
        }
    }
    
    #[test]
    fn prop_transaction_validation_consistency(
        sender in "[a-z]{5,10}",
        recipient in "[a-z]{5,10}",
        amount in 0u64..u64::MAX,
        nonce in 0u64..u64::MAX
    ) {
        let tx = Transaction::new(
            AccountId::from_string(&sender),
            AccountId::from_string(&recipient),
            Balance::from_ruv(amount),
            nonce,
        );
        
        let validation_result = tx.validate();
        
        // Validation rules
        if sender == recipient {
            prop_assert!(validation_result.is_err());
        } else if amount == 0 {
            prop_assert!(validation_result.is_err());
        } else {
            prop_assert!(validation_result.is_ok());
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore] // Run with --ignored for performance tests
    fn bench_transaction_operations() {
        let num_transactions = 100_000;
        
        // Benchmark transaction creation
        let start = Instant::now();
        let transactions: Vec<_> = (0..num_transactions)
            .map(|i| Transaction::new(
                AccountId::from_string("alice"),
                AccountId::from_string(&format!("recipient_{}", i % 1000)),
                Balance::from_ruv(100),
                i as u64,
            ))
            .collect();
        let creation_time = start.elapsed();
        
        // Benchmark hashing
        let start = Instant::now();
        let hashes: Vec<_> = transactions.iter()
            .map(|tx| tx.hash())
            .collect();
        let hashing_time = start.elapsed();
        
        // Benchmark serialization
        let start = Instant::now();
        let serialized: Vec<_> = transactions.iter()
            .map(|tx| bincode::serialize(tx).unwrap())
            .collect();
        let serialization_time = start.elapsed();
        
        println!("Transaction creation: {:?} for {} txs", creation_time, num_transactions);
        println!("Transaction hashing: {:?} for {} txs", hashing_time, num_transactions);
        println!("Transaction serialization: {:?} for {} txs", serialization_time, num_transactions);
        
        // Performance assertions
        assert!(creation_time.as_secs() < 5);
        assert!(hashing_time.as_secs() < 2);
        assert!(serialization_time.as_secs() < 3);
    }
}
