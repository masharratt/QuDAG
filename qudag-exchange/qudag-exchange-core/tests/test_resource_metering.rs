//! Tests for resource metering and rUv (Resource Utilization Voucher) consumption
//! Following TDD methodology - defining resource tracking before implementation

use qudag_exchange_core::metering::{
    ResourceMeter, ResourceType, ResourceUsage, MeteringPolicy,
    MeteringError, ResourceCost, UsageReport
};
use qudag_exchange_core::ledger::{AccountId, Balance, Ledger};
use std::time::{Duration, Instant};
use proptest::prelude::*;

#[cfg(test)]
mod resource_type_tests {
    use super::*;

    #[test]
    fn test_resource_type_definitions() {
        // Test: Define all supported resource types
        assert_eq!(ResourceType::Compute.name(), "compute");
        assert_eq!(ResourceType::Storage.name(), "storage");
        assert_eq!(ResourceType::Bandwidth.name(), "bandwidth");
        assert_eq!(ResourceType::Memory.name(), "memory");
        assert_eq!(ResourceType::GpuCompute.name(), "gpu_compute");
        assert_eq!(ResourceType::QuantumOperations.name(), "quantum_ops");
        assert_eq!(ResourceType::VaultAccess.name(), "vault_access");
        assert_eq!(ResourceType::ConsensusVoting.name(), "consensus_voting");
    }

    #[test]
    fn test_resource_units() {
        // Test: Each resource type has appropriate units
        assert_eq!(ResourceType::Compute.unit(), "cpu_seconds");
        assert_eq!(ResourceType::Storage.unit(), "bytes_hours");
        assert_eq!(ResourceType::Bandwidth.unit(), "bytes");
        assert_eq!(ResourceType::Memory.unit(), "mb_seconds");
        assert_eq!(ResourceType::GpuCompute.unit(), "gpu_seconds");
        assert_eq!(ResourceType::QuantumOperations.unit(), "operations");
    }

    #[test]
    fn test_resource_cost_calculation() {
        // Test: Calculate rUv cost for different resource usage
        let cost_table = ResourceCost::default();
        
        // CPU computation: 1 rUv per CPU-second
        let cpu_cost = cost_table.calculate(
            ResourceType::Compute,
            1000, // 1000 CPU-seconds
        );
        assert_eq!(cpu_cost, Balance::from_ruv(1000));
        
        // Storage: 0.001 rUv per MB-hour
        let storage_cost = cost_table.calculate(
            ResourceType::Storage,
            1024 * 1024 * 24, // 1 MB for 24 hours
        );
        assert_eq!(storage_cost, Balance::from_ruv(24));
        
        // Bandwidth: 0.0001 rUv per KB
        let bandwidth_cost = cost_table.calculate(
            ResourceType::Bandwidth,
            1024 * 1000, // 1 MB
        );
        assert_eq!(bandwidth_cost, Balance::from_ruv(100));
    }

    #[test]
    fn test_custom_pricing_policy() {
        // Test: Support custom pricing policies
        let mut custom_policy = ResourceCost::new();
        
        // Set premium pricing for GPU compute
        custom_policy.set_rate(ResourceType::GpuCompute, 10.0); // 10 rUv per GPU-second
        
        let gpu_cost = custom_policy.calculate(
            ResourceType::GpuCompute,
            60, // 1 minute of GPU time
        );
        assert_eq!(gpu_cost, Balance::from_ruv(600));
        
        // Discount for bulk storage
        custom_policy.set_bulk_discount(ResourceType::Storage, 1_000_000, 0.5); // 50% off after 1GB
        
        let bulk_storage_cost = custom_policy.calculate(
            ResourceType::Storage,
            2_000_000, // 2GB
        );
        // First 1GB at full price, second 1GB at 50% discount
        let expected = Balance::from_ruv(1500); // 1000 + 500
        assert_eq!(bulk_storage_cost, expected);
    }
}

#[cfg(test)]
mod resource_meter_tests {
    use super::*;

    #[test]
    fn test_meter_initialization() {
        // Test: Create resource meter with policy
        let policy = MeteringPolicy::default();
        let meter = ResourceMeter::new(policy);
        
        assert_eq!(meter.total_consumed(), Balance::zero());
        assert_eq!(meter.active_sessions(), 0);
    }

    #[test]
    fn test_basic_resource_tracking() {
        // Test: Track resource usage for a single operation
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let account = AccountId::from_string("alice");
        
        // Start tracking compute usage
        let session = meter.start_session(account.clone(), ResourceType::Compute)
            .expect("Failed to start session");
        
        // Simulate some computation
        std::thread::sleep(Duration::from_millis(100));
        
        // Stop tracking and get usage
        let usage = meter.stop_session(session)
            .expect("Failed to stop session");
        
        assert_eq!(usage.account_id(), &account);
        assert_eq!(usage.resource_type(), ResourceType::Compute);
        assert!(usage.units_consumed() > 0);
        assert!(usage.ruv_cost() > Balance::zero());
    }

    #[test]
    fn test_concurrent_resource_tracking() {
        // Test: Track multiple resources simultaneously
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let account = AccountId::from_string("alice");
        
        // Start multiple resource sessions
        let compute_session = meter.start_session(
            account.clone(),
            ResourceType::Compute
        ).unwrap();
        
        let memory_session = meter.start_session(
            account.clone(),
            ResourceType::Memory
        ).unwrap();
        
        let storage_session = meter.start_session(
            account.clone(),
            ResourceType::Storage
        ).unwrap();
        
        assert_eq!(meter.active_sessions(), 3);
        
        // Stop sessions and verify independent tracking
        let compute_usage = meter.stop_session(compute_session).unwrap();
        let memory_usage = meter.stop_session(memory_session).unwrap();
        let storage_usage = meter.stop_session(storage_session).unwrap();
        
        assert_eq!(meter.active_sessions(), 0);
        assert_ne!(compute_usage.session_id(), memory_usage.session_id());
        assert_ne!(memory_usage.session_id(), storage_usage.session_id());
    }

    #[test]
    fn test_resource_limits() {
        // Test: Enforce resource usage limits
        let mut policy = MeteringPolicy::default();
        policy.set_limit(ResourceType::Compute, 3600); // 1 hour max
        policy.set_limit(ResourceType::Memory, 1024 * 1024); // 1 GB max
        
        let mut meter = ResourceMeter::new(policy);
        let account = AccountId::from_string("alice");
        
        // Start session that will exceed limit
        let session = meter.start_session(account, ResourceType::Compute).unwrap();
        
        // Simulate exceeding limit
        meter.record_usage(session, 4000); // Exceed 1 hour limit
        
        let result = meter.check_limits(session);
        assert!(matches!(result, Err(MeteringError::LimitExceeded { .. })));
    }

    #[test]
    fn test_metered_execution() {
        // Test: Execute operations with automatic metering
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let mut ledger = Ledger::new();
        
        let account = AccountId::from_string("alice");
        ledger.create_account(account.clone()).unwrap();
        ledger.credit(&account, Balance::from_ruv(1000)).unwrap();
        
        // Execute metered operation
        let result = meter.execute_metered(
            &account,
            &mut ledger,
            ResourceType::Compute,
            100, // Estimated cost
            || {
                // Simulate some computation
                let mut sum = 0;
                for i in 0..1000000 {
                    sum += i;
                }
                Ok(sum)
            },
        );
        
        assert!(result.is_ok());
        
        // Verify rUv was deducted
        let final_balance = ledger.get_balance(&account).unwrap();
        assert!(final_balance < Balance::from_ruv(1000));
        assert!(final_balance >= Balance::from_ruv(900)); // At least 100 rUv spent
    }

    #[test]
    fn test_insufficient_balance_handling() {
        // Test: Handle operations when account lacks rUv
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let mut ledger = Ledger::new();
        
        let account = AccountId::from_string("bob");
        ledger.create_account(account.clone()).unwrap();
        ledger.credit(&account, Balance::from_ruv(10)).unwrap(); // Only 10 rUv
        
        // Try expensive operation
        let result = meter.execute_metered(
            &account,
            &mut ledger,
            ResourceType::GpuCompute,
            1000, // Requires 1000 rUv
            || {
                // This should not execute
                panic!("Operation should not run with insufficient balance");
            },
        );
        
        assert!(matches!(result, Err(MeteringError::InsufficientBalance { .. })));
        
        // Balance should remain unchanged
        assert_eq!(ledger.get_balance(&account).unwrap(), Balance::from_ruv(10));
    }
}

#[cfg(test)]
mod usage_report_tests {
    use super::*;

    #[test]
    fn test_usage_report_generation() {
        // Test: Generate comprehensive usage reports
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let alice = AccountId::from_string("alice");
        let bob = AccountId::from_string("bob");
        
        // Simulate various resource usage
        meter.record_completed_usage(ResourceUsage::new(
            alice.clone(),
            ResourceType::Compute,
            1000,
            Balance::from_ruv(100),
        ));
        
        meter.record_completed_usage(ResourceUsage::new(
            alice.clone(),
            ResourceType::Storage,
            1024 * 1024,
            Balance::from_ruv(50),
        ));
        
        meter.record_completed_usage(ResourceUsage::new(
            bob.clone(),
            ResourceType::Bandwidth,
            2048,
            Balance::from_ruv(20),
        ));
        
        // Generate report for Alice
        let alice_report = meter.generate_report(&alice, None, None)
            .expect("Report generation failed");
        
        assert_eq!(alice_report.account_id(), &alice);
        assert_eq!(alice_report.total_ruv_spent(), Balance::from_ruv(150));
        assert_eq!(alice_report.resource_breakdown().len(), 2);
        
        // Verify resource breakdown
        let compute_usage = alice_report.resource_breakdown()
            .get(&ResourceType::Compute)
            .unwrap();
        assert_eq!(compute_usage.total_units, 1000);
        assert_eq!(compute_usage.total_cost, Balance::from_ruv(100));
    }

    #[test]
    fn test_time_filtered_reports() {
        // Test: Generate reports for specific time periods
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let account = AccountId::from_string("alice");
        
        let now = Instant::now();
        let hour_ago = now - Duration::from_secs(3600);
        let two_hours_ago = now - Duration::from_secs(7200);
        
        // Record usage at different times
        meter.record_completed_usage_at(ResourceUsage::new(
            account.clone(),
            ResourceType::Compute,
            100,
            Balance::from_ruv(10),
        ), two_hours_ago);
        
        meter.record_completed_usage_at(ResourceUsage::new(
            account.clone(),
            ResourceType::Compute,
            200,
            Balance::from_ruv(20),
        ), hour_ago);
        
        meter.record_completed_usage_at(ResourceUsage::new(
            account.clone(),
            ResourceType::Compute,
            300,
            Balance::from_ruv(30),
        ), now);
        
        // Get report for last hour only
        let recent_report = meter.generate_report(
            &account,
            Some(now - Duration::from_secs(3600)),
            Some(now),
        ).unwrap();
        
        assert_eq!(recent_report.total_ruv_spent(), Balance::from_ruv(50)); // 20 + 30
    }
}

#[cfg(test)]
mod quota_management_tests {
    use super::*;

    #[test]
    fn test_account_quotas() {
        // Test: Set and enforce per-account resource quotas
        let mut policy = MeteringPolicy::default();
        let alice = AccountId::from_string("alice");
        let bob = AccountId::from_string("bob");
        
        // Set different quotas for different accounts
        policy.set_account_quota(&alice, ResourceType::Compute, 3600); // 1 hour/day
        policy.set_account_quota(&alice, ResourceType::Storage, 1024 * 1024 * 100); // 100 MB
        
        policy.set_account_quota(&bob, ResourceType::Compute, 7200); // 2 hours/day (premium)
        policy.set_account_quota(&bob, ResourceType::Storage, 1024 * 1024 * 1000); // 1 GB (premium)
        
        let meter = ResourceMeter::new(policy);
        
        // Check available quotas
        let alice_compute_quota = meter.get_remaining_quota(&alice, ResourceType::Compute);
        assert_eq!(alice_compute_quota, 3600);
        
        let bob_compute_quota = meter.get_remaining_quota(&bob, ResourceType::Compute);
        assert_eq!(bob_compute_quota, 7200);
    }

    #[test]
    fn test_quota_reset_periods() {
        // Test: Quotas reset at specified intervals
        let mut policy = MeteringPolicy::default();
        let account = AccountId::from_string("alice");
        
        // Daily compute quota
        policy.set_account_quota(&account, ResourceType::Compute, 3600);
        policy.set_quota_reset_period(ResourceType::Compute, Duration::from_secs(86400)); // 24 hours
        
        // Hourly bandwidth quota
        policy.set_account_quota(&account, ResourceType::Bandwidth, 1024 * 1024 * 100); // 100 MB/hour
        policy.set_quota_reset_period(ResourceType::Bandwidth, Duration::from_secs(3600)); // 1 hour
        
        let mut meter = ResourceMeter::new(policy);
        
        // Use some quota
        meter.consume_quota(&account, ResourceType::Compute, 1800).unwrap();
        meter.consume_quota(&account, ResourceType::Bandwidth, 1024 * 1024 * 50).unwrap();
        
        // Check remaining
        assert_eq!(meter.get_remaining_quota(&account, ResourceType::Compute), 1800);
        assert_eq!(meter.get_remaining_quota(&account, ResourceType::Bandwidth), 1024 * 1024 * 50);
        
        // Simulate time passing and quota reset
        meter.check_and_reset_quotas();
        
        // After reset (in real implementation)
        // assert_eq!(meter.get_remaining_quota(&account, ResourceType::Compute), 3600);
    }
}

#[cfg(test)]
mod special_operations_tests {
    use super::*;

    #[test]
    fn test_quantum_operations_metering() {
        // Test: Special metering for quantum cryptographic operations
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let account = AccountId::from_string("alice");
        
        // ML-DSA signature generation
        let sig_cost = meter.calculate_operation_cost(
            ResourceType::QuantumOperations,
            "ml_dsa_sign",
            2048, // Key size
        );
        assert_eq!(sig_cost, Balance::from_ruv(5)); // 5 rUv per signature
        
        // ML-KEM encryption
        let enc_cost = meter.calculate_operation_cost(
            ResourceType::QuantumOperations,
            "ml_kem_encrypt",
            768, // Security parameter
        );
        assert_eq!(enc_cost, Balance::from_ruv(3)); // 3 rUv per encryption
        
        // HQC key generation (expensive)
        let keygen_cost = meter.calculate_operation_cost(
            ResourceType::QuantumOperations,
            "hqc_keygen",
            128, // Security level
        );
        assert_eq!(keygen_cost, Balance::from_ruv(20)); // 20 rUv for key generation
    }

    #[test]
    fn test_vault_operations_metering() {
        // Test: Meter vault access operations
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let account = AccountId::from_string("alice");
        
        // Vault unlock
        let unlock_cost = meter.calculate_operation_cost(
            ResourceType::VaultAccess,
            "unlock",
            0,
        );
        assert_eq!(unlock_cost, Balance::from_ruv(1)); // 1 rUv per unlock
        
        // Key retrieval
        let key_cost = meter.calculate_operation_cost(
            ResourceType::VaultAccess,
            "get_key",
            0,
        );
        assert_eq!(key_cost, Balance::from_ruv(2)); // 2 rUv per key access
        
        // Backup creation (expensive)
        let backup_cost = meter.calculate_operation_cost(
            ResourceType::VaultAccess,
            "create_backup",
            1024 * 1024, // Vault size
        );
        assert_eq!(backup_cost, Balance::from_ruv(50)); // 50 rUv for backup
    }

    #[test]
    fn test_consensus_participation_metering() {
        // Test: Meter consensus voting and validation
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let validator = AccountId::from_string("validator1");
        
        // Vote submission
        let vote_cost = meter.calculate_operation_cost(
            ResourceType::ConsensusVoting,
            "submit_vote",
            1, // Single vote
        );
        assert_eq!(vote_cost, Balance::from_ruv(1)); // 1 rUv per vote
        
        // Block validation
        let validation_cost = meter.calculate_operation_cost(
            ResourceType::ConsensusVoting,
            "validate_block",
            100, // Number of transactions
        );
        assert_eq!(validation_cost, Balance::from_ruv(10)); // 0.1 rUv per transaction
        
        // Finality proof generation
        let finality_cost = meter.calculate_operation_cost(
            ResourceType::ConsensusVoting,
            "generate_finality_proof",
            64, // Proof size
        );
        assert_eq!(finality_cost, Balance::from_ruv(25)); // 25 rUv for finality proof
    }
}

// Property-based tests
proptest! {
    #[test]
    fn prop_resource_cost_monotonic(
        resource in 0..4u8,
        units1 in 0u64..1_000_000,
        units2 in 0u64..1_000_000,
    ) {
        let resource_type = match resource {
            0 => ResourceType::Compute,
            1 => ResourceType::Storage,
            2 => ResourceType::Bandwidth,
            _ => ResourceType::Memory,
        };
        
        let cost_table = ResourceCost::default();
        let cost1 = cost_table.calculate(resource_type, units1);
        let cost2 = cost_table.calculate(resource_type, units2);
        
        // Cost should be monotonic: more units = higher cost
        if units1 <= units2 {
            prop_assert!(cost1 <= cost2);
        } else {
            prop_assert!(cost1 >= cost2);
        }
    }
    
    #[test]
    fn prop_metering_consistency(
        operations in prop::collection::vec(
            (0..4u8, 1u64..1000),
            1..50
        )
    ) {
        let mut meter = ResourceMeter::new(MeteringPolicy::default());
        let account = AccountId::from_string("test_account");
        let mut total_cost = Balance::zero();
        
        for (res_type, units) in operations {
            let resource_type = match res_type {
                0 => ResourceType::Compute,
                1 => ResourceType::Storage,
                2 => ResourceType::Bandwidth,
                _ => ResourceType::Memory,
            };
            
            let usage = ResourceUsage::new(
                account.clone(),
                resource_type,
                units,
                Balance::from_ruv(units), // Simplified: 1 rUv per unit
            );
            
            meter.record_completed_usage(usage.clone());
            total_cost = total_cost.checked_add(usage.ruv_cost()).unwrap();
        }
        
        // Total in meter should match sum of individual costs
        let report = meter.generate_report(&account, None, None).unwrap();
        prop_assert_eq!(report.total_ruv_spent(), total_cost);
    }
    
    #[test]
    fn prop_quota_enforcement(
        quota in 100u64..10_000,
        attempts in prop::collection::vec(1u64..500, 1..20)
    ) {
        let mut policy = MeteringPolicy::default();
        let account = AccountId::from_string("test_account");
        
        policy.set_account_quota(&account, ResourceType::Compute, quota);
        let mut meter = ResourceMeter::new(policy);
        
        let mut total_consumed = 0u64;
        let mut should_fail = false;
        
        for units in attempts {
            if total_consumed + units > quota {
                should_fail = true;
            }
            
            let result = meter.consume_quota(&account, ResourceType::Compute, units);
            
            if should_fail {
                prop_assert!(result.is_err());
                break;
            } else {
                prop_assert!(result.is_ok());
                total_consumed += units;
            }
        }
        
        // Remaining quota should be correct
        let remaining = meter.get_remaining_quota(&account, ResourceType::Compute);
        prop_assert_eq!(remaining, quota.saturating_sub(total_consumed));
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    #[ignore] // Run with --ignored for performance tests
    async fn bench_concurrent_metering() {
        let meter = Arc::new(Mutex::new(ResourceMeter::new(MeteringPolicy::default())));
        let num_accounts = 100;
        let operations_per_account = 1000;
        
        let start = Instant::now();
        
        let mut handles = vec![];
        
        for i in 0..num_accounts {
            let meter_clone = Arc::clone(&meter);
            let account = AccountId::from_string(&format!("account_{}", i));
            
            let handle = tokio::spawn(async move {
                for j in 0..operations_per_account {
                    let mut m = meter_clone.lock().await;
                    
                    let usage = ResourceUsage::new(
                        account.clone(),
                        if j % 2 == 0 { ResourceType::Compute } else { ResourceType::Memory },
                        j as u64,
                        Balance::from_ruv(j as u64),
                    );
                    
                    m.record_completed_usage(usage);
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let elapsed = start.elapsed();
        
        println!("Concurrent metering benchmark:");
        println!("  Accounts: {}", num_accounts);
        println!("  Operations per account: {}", operations_per_account);
        println!("  Total operations: {}", num_accounts * operations_per_account);
        println!("  Time elapsed: {:?}", elapsed);
        println!("  Operations per second: {:.2}", 
            (num_accounts * operations_per_account) as f64 / elapsed.as_secs_f64());
        
        // Should handle at least 10k operations per second
        assert!(elapsed.as_secs() < 10);
    }
}
