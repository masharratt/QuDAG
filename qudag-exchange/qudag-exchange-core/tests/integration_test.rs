//! Integration tests for QuDAG Exchange Core

use qudag_exchange_core::{
    Ledger, Transaction,
    types::{AccountId, PublicKey},
};

#[test]
fn test_basic_exchange_flow() {
    // Create ledger
    let ledger = Ledger::new();
    
    // Create accounts
    let alice_id = AccountId::new();
    let bob_id = AccountId::new();
    
    let alice_account = ledger.create_account(alice_id).unwrap();
    let bob_account = ledger.create_account(bob_id).unwrap();
    
    assert_eq!(alice_account.balance, 1000);
    assert_eq!(bob_account.balance, 1000);
    
    // Transfer from Alice to Bob
    ledger.transfer(&alice_id, &bob_id, 100).unwrap();
    
    // Check balances
    assert_eq!(ledger.get_balance(&alice_id).unwrap(), 900);
    assert_eq!(ledger.get_balance(&bob_id).unwrap(), 1100);
    
    // Check total supply remains constant
    assert_eq!(ledger.total_supply(), 2000);
}

#[test]
fn test_transaction_creation_and_verification() {
    let from = AccountId::new();
    let to = AccountId::new();
    
    let mut tx = Transaction::new(
        from,
        to,
        250, // amount
        5,   // fee
        0,   // nonce
        PublicKey(vec![1, 2, 3, 4, 5]),
    );
    
    // Transaction should be invalid without signature
    assert!(!tx.verify().unwrap());
    
    // Add mock signature
    tx.sign(qudag_exchange_core::types::Signature(vec![10, 20, 30]));
    
    // Should now be valid
    assert!(tx.verify().unwrap());
    
    // Total cost should be amount + fee
    assert_eq!(tx.total_cost(), 255);
    
    // Hash should be deterministic
    let hash1 = tx.hash().unwrap();
    let hash2 = tx.hash().unwrap();
    assert_eq!(hash1.0, hash2.0);
}