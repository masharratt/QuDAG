//! Main ledger for QuDAG Exchange state management

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::resource::{ResourceContribution, ResourceMeter};
use crate::ruv::RuvAmount;
use crate::transaction::{Transaction, TransactionType};
use crate::wallet::{Wallet, WalletManager};

/// Ledger state for the QuDAG Exchange
#[derive(Clone)]
pub struct Ledger {
    /// Wallet manager
    wallets: Arc<RwLock<WalletManager>>,
    
    /// Transaction pool (pending transactions)
    tx_pool: Arc<DashMap<String, Transaction>>,
    
    /// Confirmed transactions
    confirmed_txs: Arc<DashMap<String, Transaction>>,
    
    /// Resource metering service
    resource_meter: Arc<RwLock<ResourceMeter>>,
    
    /// Current epoch
    epoch: Arc<RwLock<u64>>,
    
    /// Total rUv supply
    total_supply: Arc<RwLock<RuvAmount>>,
}

impl Ledger {
    /// Create a new ledger
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(RwLock::new(WalletManager::new())),
            tx_pool: Arc::new(DashMap::new()),
            confirmed_txs: Arc::new(DashMap::new()),
            resource_meter: Arc::new(RwLock::new(ResourceMeter::new())),
            epoch: Arc::new(RwLock::new(0)),
            total_supply: Arc::new(RwLock::new(RuvAmount::default())),
        }
    }

    /// Create or get a wallet
    pub fn get_or_create_wallet(&self, address: String, vault_backed: bool) -> Wallet {
        let mut wallets = self.wallets.write();
        if let Some(wallet) = wallets.get_wallet(&address) {
            wallet.clone()
        } else {
            wallets.create_wallet(address, vault_backed).clone()
        }
    }

    /// Get wallet balance
    pub fn get_balance(&self, address: &str) -> Option<RuvAmount> {
        let wallets = self.wallets.read();
        wallets.get_wallet(address).map(|w| w.balance.clone())
    }

    /// Submit a transaction to the pool
    pub fn submit_transaction(&self, mut tx: Transaction) -> Result<String> {
        // Verify transaction
        tx.verify()?;

        // Check if transaction already exists
        if self.tx_pool.contains_key(&tx.id) || self.confirmed_txs.contains_key(&tx.id) {
            return Err(Error::InvalidTransaction {
                reason: "Transaction already exists".to_string(),
            });
        }

        // For transfers, check sender balance
        if let TransactionType::Transfer { from, amount, .. } = &tx.tx_type {
            let wallets = self.wallets.read();
            if let Some(sender) = wallets.get_wallet(from) {
                if !sender.can_afford(amount, &tx.fee)? {
                    return Err(Error::InsufficientBalance {
                        required: (amount.as_ruv() + tx.fee.as_ruv()) as u128,
                        available: sender.balance().as_ruv() as u128,
                    });
                }
            } else {
                return Err(Error::Wallet(format!("Sender wallet not found: {}", from)));
            }
        }

        let tx_id = tx.id.clone();
        self.tx_pool.insert(tx_id.clone(), tx);
        Ok(tx_id)
    }

    /// Process a transaction from the pool
    pub fn process_transaction(&self, tx_id: &str) -> Result<()> {
        // Remove from pool
        let tx = self.tx_pool.remove(tx_id)
            .ok_or_else(|| Error::InvalidTransaction {
                reason: "Transaction not found in pool".to_string(),
            })?
            .1;

        // Process based on type
        match &tx.tx_type {
            TransactionType::Transfer { .. } => {
                self.process_transfer(&tx)?;
            }
            TransactionType::Mint { to, contribution } => {
                self.process_mint(to, contribution)?;
            }
            TransactionType::Burn { from, amount } => {
                self.process_burn(from, amount, &tx.fee)?;
            }
            TransactionType::FeeDistribution { .. } => {
                self.process_fee_distribution(&tx)?;
            }
            TransactionType::Execute { .. } => {
                // Contract execution not implemented yet
                return Err(Error::Other("Contract execution not implemented".to_string()));
            }
        }

        // Add to confirmed transactions
        self.confirmed_txs.insert(tx.id.clone(), tx);
        
        Ok(())
    }

    /// Process a transfer transaction
    fn process_transfer(&self, tx: &Transaction) -> Result<()> {
        let mut wallets = self.wallets.write();
        wallets.process_transaction(tx)
    }

    /// Process a mint transaction
    fn process_mint(&self, to: &str, contribution: &ResourceContribution) -> Result<()> {
        // Verify contribution
        if !contribution.verified {
            return Err(Error::InvalidTransaction {
                reason: "Contribution not verified".to_string(),
            });
        }

        // Update wallet balance
        let mut wallets = self.wallets.write();
        if let Some(wallet) = wallets.get_wallet_mut(to) {
            wallet.balance = wallet.balance.checked_add(contribution.total_value())?;
        } else {
            // Create wallet if it doesn't exist
            let mut wallet = Wallet::new(to.to_string());
            wallet.balance = contribution.total_value().clone();
            wallets.create_wallet(to.to_string(), false);
        }

        // Update total supply
        let mut total_supply = self.total_supply.write();
        *total_supply = total_supply.checked_add(contribution.total_value())?;

        Ok(())
    }

    /// Process a burn transaction
    fn process_burn(&self, from: &str, amount: &RuvAmount, fee: &RuvAmount) -> Result<()> {
        let mut wallets = self.wallets.write();
        
        if let Some(wallet) = wallets.get_wallet_mut(from) {
            let total = amount.checked_add(fee)?;
            wallet.balance = wallet.balance.checked_sub(&total)?;
        } else {
            return Err(Error::Wallet(format!("Wallet not found: {}", from)));
        }

        // Update total supply (decrease by burn amount, not fee)
        let mut total_supply = self.total_supply.write();
        *total_supply = total_supply.checked_sub(amount)?;

        Ok(())
    }

    /// Process fee distribution
    fn process_fee_distribution(&self, tx: &Transaction) -> Result<()> {
        if let TransactionType::FeeDistribution { amount, recipients } = &tx.tx_type {
            let mut wallets = self.wallets.write();
            
            for (addr, share) in recipients {
                let share_amount = (amount.as_ruv() * (*share as u64)) / 100;
                let ruv_share = RuvAmount::from_ruv(share_amount);
                
                if let Some(wallet) = wallets.get_wallet_mut(addr) {
                    wallet.balance = wallet.balance.checked_add(&ruv_share)?;
                } else {
                    // Create wallet if it doesn't exist
                    let mut wallet = Wallet::new(addr.clone());
                    wallet.balance = ruv_share;
                    wallets.create_wallet(addr.clone(), false);
                }
            }
        }
        
        Ok(())
    }

    /// Start tracking a resource contribution
    pub fn start_resource_contribution(&self, agent_id: String) {
        let mut meter = self.resource_meter.write();
        meter.start_contribution(agent_id);
    }

    /// Record a resource metric
    pub fn record_resource_metric(
        &self, 
        agent_id: &str, 
        metric: crate::resource::ResourceMetrics
    ) -> Result<()> {
        let mut meter = self.resource_meter.write();
        meter.record_metric(agent_id, metric)
    }

    /// Finalize a resource contribution and create mint transaction
    pub fn finalize_resource_contribution(&self, agent_id: &str) -> Result<Option<String>> {
        let mut meter = self.resource_meter.write();
        
        if let Some(contribution) = meter.finalize_contribution(agent_id) {
            if contribution.total_value().is_zero() {
                return Ok(None);
            }

            // Create mint transaction
            let tx = Transaction::new(
                TransactionType::Mint {
                    to: agent_id.to_string(),
                    contribution,
                },
                RuvAmount::from_ruv(0), // No fee for minting
            );

            let tx_id = self.submit_transaction(tx)?;
            Ok(Some(tx_id))
        } else {
            Ok(None)
        }
    }

    /// Get current epoch
    pub fn current_epoch(&self) -> u64 {
        *self.epoch.read()
    }

    /// Advance to next epoch
    pub fn advance_epoch(&self) {
        let mut epoch = self.epoch.write();
        *epoch += 1;
    }

    /// Get total supply
    pub fn total_supply(&self) -> RuvAmount {
        self.total_supply.read().clone()
    }

    /// Get transaction pool size
    pub fn tx_pool_size(&self) -> usize {
        self.tx_pool.len()
    }

    /// Get a transaction by ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<Transaction> {
        self.confirmed_txs.get(tx_id)
            .map(|entry| entry.value().clone())
            .or_else(|| self.tx_pool.get(tx_id).map(|entry| entry.value().clone()))
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

/// Ledger statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerStats {
    /// Current epoch
    pub epoch: u64,
    /// Total supply
    pub total_supply: u64,
    /// Number of wallets
    pub wallet_count: usize,
    /// Pending transactions
    pub pending_txs: usize,
    /// Confirmed transactions
    pub confirmed_txs: usize,
}

impl Ledger {
    /// Get ledger statistics
    pub fn stats(&self) -> LedgerStats {
        let wallets = self.wallets.read();
        
        LedgerStats {
            epoch: self.current_epoch(),
            total_supply: self.total_supply().as_ruv(),
            wallet_count: wallets.wallets.len(),
            pending_txs: self.tx_pool.len(),
            confirmed_txs: self.confirmed_txs.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::ResourceMetrics;

    #[test]
    fn test_ledger_creation() {
        let ledger = Ledger::new();
        assert_eq!(ledger.current_epoch(), 0);
        assert!(ledger.total_supply().is_zero());
    }

    #[test]
    fn test_wallet_creation() {
        let ledger = Ledger::new();
        let wallet = ledger.get_or_create_wallet("alice".to_string(), false);
        assert_eq!(wallet.address, "alice");
        assert!(wallet.balance.is_zero());
    }

    #[test]
    fn test_resource_contribution_flow() {
        let ledger = Ledger::new();
        
        // Start contribution
        ledger.start_resource_contribution("agent1".to_string());
        
        // Record metrics
        let metric = ResourceMetrics {
            resource_type: crate::resource::ResourceType::Cpu,
            amount: 100.0,
            duration: 3600,
            quality_score: 1.0,
            timestamp: 0,
        };
        
        ledger.record_resource_metric("agent1", metric).unwrap();
        
        // Finalize and mint
        let tx_id = ledger.finalize_resource_contribution("agent1").unwrap().unwrap();
        
        // Process the mint transaction
        ledger.process_transaction(&tx_id).unwrap();
        
        // Check balance
        let balance = ledger.get_balance("agent1").unwrap();
        assert_eq!(balance.as_ruv(), 10); // 100 * 1 * 1.0 * 0.1 = 10 rUv
    }
}