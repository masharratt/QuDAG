//! Transaction types and processing for QuDAG Exchange

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use crate::error::{Error, Result};
use crate::resource::ResourceContribution;
use crate::ruv::RuvAmount;

/// Types of transactions in the QuDAG Exchange
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransactionType {
    /// Transfer rUv between wallets
    Transfer {
        /// Sender address
        from: String,
        /// Recipient address
        to: String,
        /// Amount to transfer
        amount: RuvAmount,
    },
    
    /// Mint new rUv from resource contribution
    Mint {
        /// Beneficiary address
        to: String,
        /// Resource contribution proof
        contribution: ResourceContribution,
    },
    
    /// Burn rUv (remove from circulation)
    Burn {
        /// Address burning rUv
        from: String,
        /// Amount to burn
        amount: RuvAmount,
    },
    
    /// Fee distribution to validators
    FeeDistribution {
        /// Fee amount
        amount: RuvAmount,
        /// Recipients and their shares
        recipients: Vec<(String, u32)>,
    },
    
    /// Smart contract execution
    Execute {
        /// Contract address
        contract: String,
        /// Execution payload
        payload: Vec<u8>,
        /// Gas limit in rUv
        gas_limit: RuvAmount,
    },
}

/// A transaction in the QuDAG Exchange
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction ID
    pub id: String,
    
    /// Transaction type and data
    pub tx_type: TransactionType,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// Transaction fee
    pub fee: RuvAmount,
    
    /// Signature (quantum-resistant)
    pub signature: Option<Vec<u8>>,
    
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(tx_type: TransactionType, fee: RuvAmount) -> Self {
        let mut tx = Self {
            id: String::new(),
            tx_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            fee,
            signature: None,
            metadata: None,
        };
        
        // Generate ID from hash
        tx.id = tx.calculate_hash();
        tx
    }

    /// Calculate transaction hash
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha3_256::new();
        
        // Hash transaction data (excluding ID and signature)
        match &self.tx_type {
            TransactionType::Transfer { from, to, amount } => {
                hasher.update(b"transfer");
                hasher.update(from.as_bytes());
                hasher.update(to.as_bytes());
                hasher.update(amount.as_units().to_bytes_le());
            }
            TransactionType::Mint { to, contribution } => {
                hasher.update(b"mint");
                hasher.update(to.as_bytes());
                hasher.update(&contribution.agent_id.as_bytes());
                hasher.update(contribution.total_value().as_units().to_bytes_le());
            }
            TransactionType::Burn { from, amount } => {
                hasher.update(b"burn");
                hasher.update(from.as_bytes());
                hasher.update(amount.as_units().to_bytes_le());
            }
            TransactionType::FeeDistribution { amount, recipients } => {
                hasher.update(b"fee_distribution");
                hasher.update(amount.as_units().to_bytes_le());
                for (addr, share) in recipients {
                    hasher.update(addr.as_bytes());
                    hasher.update(&share.to_le_bytes());
                }
            }
            TransactionType::Execute { contract, payload, gas_limit } => {
                hasher.update(b"execute");
                hasher.update(contract.as_bytes());
                hasher.update(payload);
                hasher.update(gas_limit.as_units().to_bytes_le());
            }
        }
        
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(self.fee.as_units().to_bytes_le());
        
        hex::encode(hasher.finalize())
    }

    /// Verify transaction validity
    pub fn verify(&self) -> Result<()> {
        // Check minimum fee
        if self.fee.is_zero() {
            return Err(Error::InvalidTransaction {
                reason: "Transaction fee cannot be zero".to_string(),
            });
        }

        // Verify transaction-specific rules
        match &self.tx_type {
            TransactionType::Transfer { from, to, amount } => {
                if from == to {
                    return Err(Error::InvalidTransaction {
                        reason: "Cannot transfer to same address".to_string(),
                    });
                }
                if amount.is_zero() {
                    return Err(Error::InvalidTransaction {
                        reason: "Transfer amount cannot be zero".to_string(),
                    });
                }
            }
            TransactionType::Mint { contribution, .. } => {
                if !contribution.verified {
                    return Err(Error::InvalidTransaction {
                        reason: "Resource contribution not verified".to_string(),
                    });
                }
                if contribution.total_value().is_zero() {
                    return Err(Error::InvalidTransaction {
                        reason: "Contribution value cannot be zero".to_string(),
                    });
                }
            }
            TransactionType::Burn { amount, .. } => {
                if amount.is_zero() {
                    return Err(Error::InvalidTransaction {
                        reason: "Burn amount cannot be zero".to_string(),
                    });
                }
            }
            TransactionType::FeeDistribution { recipients, .. } => {
                if recipients.is_empty() {
                    return Err(Error::InvalidTransaction {
                        reason: "Fee distribution must have recipients".to_string(),
                    });
                }
                let total_shares: u32 = recipients.iter().map(|(_, share)| share).sum();
                if total_shares != 100 {
                    return Err(Error::InvalidTransaction {
                        reason: "Fee distribution shares must sum to 100".to_string(),
                    });
                }
            }
            TransactionType::Execute { gas_limit, .. } => {
                if gas_limit.is_zero() {
                    return Err(Error::InvalidTransaction {
                        reason: "Gas limit cannot be zero".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Sign the transaction (placeholder - actual implementation would use quantum-resistant signatures)
    pub fn sign(&mut self, _private_key: &[u8]) -> Result<()> {
        // TODO: Implement actual quantum-resistant signing
        self.signature = Some(vec![0; 64]);
        Ok(())
    }

    /// Verify transaction signature
    pub fn verify_signature(&self, _public_key: &[u8]) -> Result<bool> {
        // TODO: Implement actual signature verification
        Ok(self.signature.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_transaction() {
        let tx = Transaction::new(
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "bob".to_string(),
                amount: RuvAmount::from_ruv(100),
            },
            RuvAmount::from_ruv(1),
        );

        assert!(tx.verify().is_ok());
        assert!(!tx.id.is_empty());
    }

    #[test]
    fn test_invalid_transfer() {
        let tx = Transaction::new(
            TransactionType::Transfer {
                from: "alice".to_string(),
                to: "alice".to_string(), // Same address
                amount: RuvAmount::from_ruv(100),
            },
            RuvAmount::from_ruv(1),
        );

        assert!(tx.verify().is_err());
    }

    #[test]
    fn test_mint_transaction() {
        let mut contribution = ResourceContribution::new("agent1".to_string());
        contribution.total_ruv = RuvAmount::from_ruv(50);
        contribution.verify();

        let tx = Transaction::new(
            TransactionType::Mint {
                to: "agent1".to_string(),
                contribution,
            },
            RuvAmount::from_ruv(1),
        );

        assert!(tx.verify().is_ok());
    }
}