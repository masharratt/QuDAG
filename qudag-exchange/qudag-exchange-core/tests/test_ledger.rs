//! Tests for the rUv (Resource Utilization Voucher) ledger and account management
//! Following TDD methodology - these tests are written before implementation

use qudag_exchange_core::ledger::{Account, AccountId, Balance, Ledger, LedgerError};
use proptest::prelude::*;
use std::collections::HashMap;

#[cfg(test)]
mod account_tests {
    use super::*;

    #[test]
    fn test_new_account_creation() {
        // Test: New account should be created with zero balance by default
        let account_id = AccountId::new();
        let account = Account::new(account_id.clone());
        
        assert_eq!(account.id(), &account_id);
        assert_eq!(account.balance(), Balance::zero());
        assert_eq!(account.nonce(), 0);
        assert!(account.is_active());
    }

    #[test]
    fn test_account_with_initial_balance() {
        // Test: Account can be created with initial balance
        let account_id = AccountId::new();
        let initial_balance = Balance::from_ruv(1000);
        let account = Account::with_balance(account_id.clone(), initial_balance);
        
        assert_eq!(account.balance(), initial_balance);
    }

    #[test]
    fn test_account_id_from_public_key() {
        // Test: AccountId should be derivable from quantum-resistant public key
        let public_key = vec![1, 2, 3, 4, 5]; // Mock ML-DSA public key
        let account_id = AccountId::from_public_key(&public_key);
        
        // Verify deterministic generation
        let account_id2 = AccountId::from_public_key(&public_key);
        assert_eq!(account_id, account_id2);
    }

    #[test]
    fn test_account_metadata() {
        // Test: Account should support metadata for dark domain registration
        let mut account = Account::new(AccountId::new());
        
        account.set_metadata("dark_domain", "alice.dark");
        assert_eq!(account.get_metadata("dark_domain"), Some("alice.dark"));
        
        account.set_metadata("quantum_fingerprint", "QF:1234:5678");
        assert_eq!(account.get_metadata("quantum_fingerprint"), Some("QF:1234:5678"));
    }
}

#[cfg(test)]
mod balance_tests {
    use super::*;

    #[test]
    fn test_balance_arithmetic_operations() {
        // Test: Balance should support safe arithmetic operations
        let balance1 = Balance::from_ruv(1000);
        let balance2 = Balance::from_ruv(500);
        
        // Addition
        let sum = balance1.checked_add(balance2).expect("Addition failed");
        assert_eq!(sum, Balance::from_ruv(1500));
        
        // Subtraction
        let diff = balance1.checked_sub(balance2).expect("Subtraction failed");
        assert_eq!(diff, Balance::from_ruv(500));
        
        // Multiplication for fees
        let doubled = balance1.checked_mul(2).expect("Multiplication failed");
        assert_eq!(doubled, Balance::from_ruv(2000));
    }

    #[test]
    fn test_balance_overflow_protection() {
        // Test: Balance operations should prevent overflow
        let max_balance = Balance::max();
        let one = Balance::from_ruv(1);
        
        assert!(max_balance.checked_add(one).is_none());
    }

    #[test]
    fn test_balance_underflow_protection() {
        // Test: Balance operations should prevent underflow
        let small_balance = Balance::from_ruv(100);
        let large_balance = Balance::from_ruv(200);
        
        assert!(small_balance.checked_sub(large_balance).is_none());
    }

    #[test]
    fn test_balance_display_formatting() {
        // Test: Balance should have human-readable display format
        let balance = Balance::from_ruv(1_234_567);
        let formatted = balance.to_string();
        
        assert!(formatted.contains("1,234,567 rUv") || formatted.contains("1234567 rUv"));
    }

    #[test]
    fn test_balance_serialization() {
        // Test: Balance should be serializable for network/storage
        let balance = Balance::from_ruv(42_000);
        
        // JSON serialization
        let json = serde_json::to_string(&balance).expect("JSON serialization failed");
        let deserialized: Balance = serde_json::from_str(&json).expect("JSON deserialization failed");
        assert_eq!(balance, deserialized);
        
        // Bincode serialization for efficient storage
        let bytes = bincode::serialize(&balance).expect("Bincode serialization failed");
        let deserialized: Balance = bincode::deserialize(&bytes).expect("Bincode deserialization failed");
        assert_eq!(balance, deserialized);
    }
}

#[cfg(test)]
mod ledger_tests {
    use super::*;

    #[test]
    fn test_ledger_creation_and_initialization() {
        // Test: Ledger should be created empty and support initialization
        let ledger = Ledger::new();
        assert_eq!(ledger.total_accounts(), 0);
        assert_eq!(ledger.total_supply(), Balance::zero());
    }

    #[test]
    fn test_ledger_account_creation() {
        // Test: Ledger should support creating new accounts
        let mut ledger = Ledger::new();
        let account_id = AccountId::new();
        
        ledger.create_account(account_id.clone()).expect("Account creation failed");
        
        assert_eq!(ledger.total_accounts(), 1);
        assert!(ledger.account_exists(&account_id));
        
        // Creating duplicate account should fail
        assert!(matches!(
            ledger.create_account(account_id),
            Err(LedgerError::AccountAlreadyExists)
        ));
    }

    #[test]
    fn test_ledger_balance_operations() {
        // Test: Ledger should support credit and debit operations
        let mut ledger = Ledger::new();
        let account_id = AccountId::new();
        ledger.create_account(account_id.clone()).expect("Account creation failed");
        
        // Credit operation
        let credit_amount = Balance::from_ruv(1000);
        ledger.credit(&account_id, credit_amount).expect("Credit failed");
        assert_eq!(ledger.get_balance(&account_id).unwrap(), credit_amount);
        assert_eq!(ledger.total_supply(), credit_amount);
        
        // Debit operation
        let debit_amount = Balance::from_ruv(300);
        ledger.debit(&account_id, debit_amount).expect("Debit failed");
        assert_eq!(ledger.get_balance(&account_id).unwrap(), Balance::from_ruv(700));
        
        // Insufficient funds
        let large_debit = Balance::from_ruv(1000);
        assert!(matches!(
            ledger.debit(&account_id, large_debit),
            Err(LedgerError::InsufficientBalance { .. })
        ));
    }

    #[test]
    fn test_ledger_transfer_operations() {
        // Test: Ledger should support atomic transfers between accounts
        let mut ledger = Ledger::new();
        let alice = AccountId::new();
        let bob = AccountId::new();
        
        ledger.create_account(alice.clone()).expect("Alice account creation failed");
        ledger.create_account(bob.clone()).expect("Bob account creation failed");
        
        // Give Alice some rUv
        let initial_amount = Balance::from_ruv(1000);
        ledger.credit(&alice, initial_amount).expect("Credit failed");
        
        // Transfer from Alice to Bob
        let transfer_amount = Balance::from_ruv(400);
        ledger.transfer(&alice, &bob, transfer_amount).expect("Transfer failed");
        
        assert_eq!(ledger.get_balance(&alice).unwrap(), Balance::from_ruv(600));
        assert_eq!(ledger.get_balance(&bob).unwrap(), Balance::from_ruv(400));
        assert_eq!(ledger.total_supply(), initial_amount); // Total supply unchanged
        
        // Test transfer with insufficient funds
        let large_transfer = Balance::from_ruv(700);
        assert!(matches!(
            ledger.transfer(&alice, &bob, large_transfer),
            Err(LedgerError::InsufficientBalance { .. })
        ));
    }

    #[test]
    fn test_ledger_nonce_increment() {
        // Test: Ledger should track account nonces for replay protection
        let mut ledger = Ledger::new();
        let account_id = AccountId::new();
        ledger.create_account(account_id.clone()).expect("Account creation failed");
        
        assert_eq!(ledger.get_nonce(&account_id).unwrap(), 0);
        
        ledger.increment_nonce(&account_id).expect("Nonce increment failed");
        assert_eq!(ledger.get_nonce(&account_id).unwrap(), 1);
        
        ledger.increment_nonce(&account_id).expect("Nonce increment failed");
        assert_eq!(ledger.get_nonce(&account_id).unwrap(), 2);
    }

    #[test]
    fn test_ledger_fee_collection() {
        // Test: Ledger should support fee collection mechanism
        let mut ledger = Ledger::new();
        let user = AccountId::new();
        let fee_collector = AccountId::from_string("fee_collector");
        
        ledger.create_account(user.clone()).expect("User account creation failed");
        ledger.create_account(fee_collector.clone()).expect("Fee collector account creation failed");
        
        // Give user some rUv
        ledger.credit(&user, Balance::from_ruv(1000)).expect("Credit failed");
        
        // Execute transfer with fee
        let transfer_amount = Balance::from_ruv(100);
        let fee = Balance::from_ruv(5);
        ledger.transfer_with_fee(&user, &fee_collector, transfer_amount, fee)
            .expect("Transfer with fee failed");
        
        assert_eq!(ledger.get_balance(&user).unwrap(), Balance::from_ruv(895)); // 1000 - 100 - 5
        assert_eq!(ledger.get_balance(&fee_collector).unwrap(), Balance::from_ruv(105)); // 100 + 5
    }

    #[test]
    fn test_ledger_account_locking() {
        // Test: Ledger should support account locking for security
        let mut ledger = Ledger::new();
        let account_id = AccountId::new();
        ledger.create_account(account_id.clone()).expect("Account creation failed");
        ledger.credit(&account_id, Balance::from_ruv(1000)).expect("Credit failed");
        
        // Lock account
        ledger.lock_account(&account_id).expect("Account lock failed");
        
        // Operations on locked account should fail
        assert!(matches!(
            ledger.debit(&account_id, Balance::from_ruv(100)),
            Err(LedgerError::AccountLocked)
        ));
        
        // Unlock account
        ledger.unlock_account(&account_id).expect("Account unlock failed");
        
        // Operations should work again
        ledger.debit(&account_id, Balance::from_ruv(100)).expect("Debit failed after unlock");
    }

    #[test]
    fn test_ledger_snapshot_and_restore() {
        // Test: Ledger should support snapshots for checkpointing
        let mut ledger = Ledger::new();
        let account = AccountId::new();
        ledger.create_account(account.clone()).expect("Account creation failed");
        ledger.credit(&account, Balance::from_ruv(1000)).expect("Credit failed");
        
        // Create snapshot
        let snapshot = ledger.create_snapshot();
        
        // Make changes
        ledger.debit(&account, Balance::from_ruv(500)).expect("Debit failed");
        assert_eq!(ledger.get_balance(&account).unwrap(), Balance::from_ruv(500));
        
        // Restore from snapshot
        ledger.restore_snapshot(snapshot).expect("Snapshot restore failed");
        assert_eq!(ledger.get_balance(&account).unwrap(), Balance::from_ruv(1000));
    }
}

// Property-based tests using proptest
proptest! {
    #[test]
    fn prop_balance_addition_commutative(
        a in 0u64..u64::MAX/2,
        b in 0u64..u64::MAX/2
    ) {
        let balance_a = Balance::from_ruv(a);
        let balance_b = Balance::from_ruv(b);
        
        let sum1 = balance_a.checked_add(balance_b);
        let sum2 = balance_b.checked_add(balance_a);
        
        prop_assert_eq!(sum1, sum2);
    }
    
    #[test]
    fn prop_ledger_transfer_preserves_total_supply(
        initial_alice in 1000u64..1_000_000,
        initial_bob in 0u64..1_000_000,
        transfer_amount in 1u64..1000
    ) {
        let mut ledger = Ledger::new();
        let alice = AccountId::from_string("alice");
        let bob = AccountId::from_string("bob");
        
        ledger.create_account(alice.clone()).unwrap();
        ledger.create_account(bob.clone()).unwrap();
        
        ledger.credit(&alice, Balance::from_ruv(initial_alice)).unwrap();
        ledger.credit(&bob, Balance::from_ruv(initial_bob)).unwrap();
        
        let total_before = ledger.total_supply();
        
        // Only transfer if Alice has enough
        if transfer_amount <= initial_alice {
            ledger.transfer(&alice, &bob, Balance::from_ruv(transfer_amount)).unwrap();
        }
        
        let total_after = ledger.total_supply();
        prop_assert_eq!(total_before, total_after);
    }
    
    #[test]
    fn prop_ledger_operations_maintain_invariants(
        ops in prop::collection::vec(
            prop_oneof![
                (0..10u64).prop_map(|id| LedgerOp::CreateAccount(AccountId::from_string(&id.to_string()))),
                (0..10u64, 0..1000u64).prop_map(|(id, amount)| 
                    LedgerOp::Credit(AccountId::from_string(&id.to_string()), Balance::from_ruv(amount))
                ),
                (0..10u64, 0..100u64).prop_map(|(id, amount)| 
                    LedgerOp::Debit(AccountId::from_string(&id.to_string()), Balance::from_ruv(amount))
                ),
            ],
            0..50
        )
    ) {
        let mut ledger = Ledger::new();
        let mut expected_supply = Balance::zero();
        
        for op in ops {
            match op {
                LedgerOp::CreateAccount(id) => {
                    let _ = ledger.create_account(id);
                },
                LedgerOp::Credit(id, amount) => {
                    if ledger.account_exists(&id) {
                        if let Ok(_) = ledger.credit(&id, amount) {
                            expected_supply = expected_supply.checked_add(amount).unwrap_or(expected_supply);
                        }
                    }
                },
                LedgerOp::Debit(id, amount) => {
                    if ledger.account_exists(&id) {
                        if let Ok(_) = ledger.debit(&id, amount) {
                            expected_supply = expected_supply.checked_sub(amount).unwrap_or(expected_supply);
                        }
                    }
                },
            }
        }
        
        // Invariant: total supply equals sum of all balances
        prop_assert_eq!(ledger.total_supply(), expected_supply);
        
        // Invariant: no negative balances
        for account_id in ledger.all_accounts() {
            let balance = ledger.get_balance(&account_id).unwrap();
            prop_assert!(balance >= Balance::zero());
        }
    }
}

#[derive(Debug, Clone)]
enum LedgerOp {
    CreateAccount(AccountId),
    Credit(AccountId, Balance),
    Debit(AccountId, Balance),
}

#[cfg(test)]
mod concurrency_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    #[tokio::test]
    async fn test_concurrent_transfers() {
        // Test: Ledger should handle concurrent transfers safely
        let ledger = Arc::new(Mutex::new(Ledger::new()));
        
        // Setup accounts
        let alice = AccountId::from_string("alice");
        let bob = AccountId::from_string("bob");
        let charlie = AccountId::from_string("charlie");
        
        {
            let mut l = ledger.lock().await;
            l.create_account(alice.clone()).unwrap();
            l.create_account(bob.clone()).unwrap();
            l.create_account(charlie.clone()).unwrap();
            l.credit(&alice, Balance::from_ruv(10_000)).unwrap();
        }
        
        // Spawn multiple concurrent transfers
        let mut handles = vec![];
        
        for i in 0..10 {
            let ledger_clone = Arc::clone(&ledger);
            let alice_clone = alice.clone();
            let target = if i % 2 == 0 { bob.clone() } else { charlie.clone() };
            
            let handle = tokio::spawn(async move {
                let mut l = ledger_clone.lock().await;
                l.transfer(&alice_clone, &target, Balance::from_ruv(100)).ok();
            });
            
            handles.push(handle);
        }
        
        // Wait for all transfers
        for handle in handles {
            handle.await.unwrap();
        }
        
        // Verify final state
        let l = ledger.lock().await;
        let alice_balance = l.get_balance(&alice).unwrap();
        let bob_balance = l.get_balance(&bob).unwrap();
        let charlie_balance = l.get_balance(&charlie).unwrap();
        
        // Total should still be 10,000
        let total = alice_balance.checked_add(bob_balance).unwrap()
            .checked_add(charlie_balance).unwrap();
        assert_eq!(total, Balance::from_ruv(10_000));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    #[ignore] // Run with --ignored flag for performance tests
    fn bench_ledger_operations() {
        let mut ledger = Ledger::new();
        let num_accounts = 10_000;
        let num_transfers = 100_000;
        
        // Create accounts
        let start = Instant::now();
        for i in 0..num_accounts {
            let account = AccountId::from_string(&format!("account_{}", i));
            ledger.create_account(account.clone()).unwrap();
            ledger.credit(&account, Balance::from_ruv(1000)).unwrap();
        }
        let account_creation_time = start.elapsed();
        
        // Perform transfers
        let start = Instant::now();
        for i in 0..num_transfers {
            let from = AccountId::from_string(&format!("account_{}", i % num_accounts));
            let to = AccountId::from_string(&format!("account_{}", (i + 1) % num_accounts));
            let _ = ledger.transfer(&from, &to, Balance::from_ruv(1));
        }
        let transfer_time = start.elapsed();
        
        println!("Account creation time for {} accounts: {:?}", num_accounts, account_creation_time);
        println!("Transfer time for {} transfers: {:?}", num_transfers, transfer_time);
        println!("Average transfer time: {:?}", transfer_time / num_transfers as u32);
        
        // Assert reasonable performance
        assert!(account_creation_time.as_secs() < 5, "Account creation too slow");
        assert!(transfer_time.as_secs() < 10, "Transfers too slow");
    }
}
