use libfuzzer_sys::fuzz_target;
use qudag_protocol::{consensus::DAGConsensus, message::Message, node::Node};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

/// Test for timing uniformity in consensus operations
fn verify_consensus_timing<F>(op: F) -> (bool, Duration)
where
    F: Fn() -> Result<(), anyhow::Error> + Send + Sync
{
    let iterations = 100;
    let mut timings = Vec::with_capacity(iterations);
    
    for _ in 0..iterations {
        let start = std::time::Instant::now();
        let _ = op();
        timings.push(start.elapsed());
    }
    
    let mean = timings.iter().sum::<Duration>() / iterations as u32;
    let variance = timings.iter()
        .map(|t| {
            let diff = t.as_nanos() as i128 - mean.as_nanos() as i128;
            diff * diff
        })
        .sum::<i128>() / iterations as i128;
    
    let max_allowed_variance = 10000;
    (variance < max_allowed_variance, mean) // Return both timing check result and mean duration
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Fuzz consensus with timing analysis
    let consensus = DAGConsensus::new();
    let (consensus_timing, mean_duration) = verify_consensus_timing(|| {
        consensus.process_message(data).map_err(|e| e.into())
    });
    assert!(consensus_timing, "Consensus processing timing variance too high (mean: {:?})", mean_duration);

    // Fuzz DAG operations with different graph sizes
    if data.len() >= 64 {
        let dag_timing = verify_consensus_timing(|| {
            consensus.process_dag_update(&data[..64]).map_err(|e| e.into())
        });
        assert!(dag_timing, "DAG update timing variance too high");
        
        // Test graph consistency
        if data.len() >= 96 {
            let node_count = (data[64] % 10) + 2; // 2-11 nodes
            let mut nodes = Vec::new();
            
            // Create test network with memory pooling
            let node_pool = Arc::new(Mutex::new(Vec::with_capacity(node_count as usize)));
            for i in 0..node_count {
                let node = Node::new(format!("node{}", i));
                node_pool.lock().unwrap().push(node);
            }
            nodes = node_pool.lock().unwrap().clone();
            
            // Test parallel message processing with batching
            let updates = data[65..96].chunks(4)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<_>>();
            
            // Process updates in parallel with message batching
            let batch_size = 4;
            let node_arc = Arc::new(nodes);
            updates.chunks(batch_size).for_each(|batch| {
                let nodes = Arc::clone(&node_arc);
                batch.par_iter().for_each(|update| {
                    nodes.iter().for_each(|node| {
                        if let Err(e) = node.process_update(update) {
                            tracing::warn!("Update processing error: {}", e);
                        }
                    });
                });
            });
        }
    }

    // Fuzz protocol messages with adversarial inputs
    if data.len() >= 128 {
        let message = Message::from_bytes(data);
        let validation_timing = verify_consensus_timing(|| {
            consensus.validate_message(&message).map_err(|e| e.into())
        });
        assert!(validation_timing, "Message validation timing variance too high");
        
        // Test message mutations
        if data.len() >= 256 {
            // Flip random bits
            let mut mutated = data[128..256].to_vec();
            for i in 0..mutated.len() {
                if data[i] % 2 == 0 {
                    mutated[i] ^= 1;
                }
            }
            let _ = Message::from_bytes(&mutated);
            
            // Test truncated messages
            for len in (128..256).step_by(8) {
                let _ = Message::from_bytes(&data[..len]);
            }
        }
    }
    
    // Test consensus edge cases
    if data.len() >= 32 {
        // Empty DAG update
        let _ = consensus.process_dag_update(&[0; 32]);
        
        // Maximum sized update
        let large_update = vec![0xFF; 32];
        let _ = consensus.process_dag_update(&large_update);
        
        // Alternating pattern
        let alt_update = (0..32).map(|i| if i % 2 == 0 {0} else {255}).collect::<Vec<_>>();
        let _ = consensus.process_dag_update(&alt_update);
    }
});