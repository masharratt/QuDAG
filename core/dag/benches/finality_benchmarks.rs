use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, SamplingMode};
use rand::Rng;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use blake3::Hash;

use qudag_dag::{
    Graph, Node, NodeState, QrAvalanche, ConsensusEvent,
    VerificationMethod, VertexProcessor
};

/// Generates a test DAG with the specified number of nodes
fn generate_test_dag(size: usize) -> (Arc<Graph>, Vec<Hash>) {
    let graph = Arc::new(Graph::new());
    let mut node_hashes = Vec::with_capacity(size);
    
    // Create root node
    let root = Node::new(vec![0], vec![]);
    let root_hash = *root.hash();
    graph.add_node(root).unwrap();
    node_hashes.push(root_hash);
    
    // Create remaining nodes with random parents
    for i in 1..size {
        let data = vec![i as u8];
        let mut parents = Vec::new();
        
        // Select parents from existing nodes
        let parent_count = (i % 5) + 1;
        for _ in 0..parent_count {
            let parent_idx = i % node_hashes.len();
            parents.push(node_hashes[parent_idx]);
        }
        
        let node = Node::new(data, parents);
        let node_hash = *node.hash();
        graph.add_node(node).unwrap();
        node_hashes.push(node_hash);
    }
    
    (graph, node_hashes)
}

fn benchmark_finality_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    // Test different DAG sizes
    let sizes = vec![100, 1000, 10000, 50000, 100000];
    
    let mut group = c.benchmark_group("finality_latency");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.sampling_mode(criterion::SamplingMode::Flat);
    
    for size in sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let (graph, node_hashes) = generate_test_dag(black_box(size));
                    let (consensus, mut events_rx) = QrAvalanche::new(graph.clone());
                    
                    rt.block_on(async {
                        let mut latencies = Vec::with_capacity(node_hashes.len());
                        let mut p99_latency = Duration::ZERO;
                        let mut total_latency = Duration::ZERO;
                        let mut finalized = 0;
                        let mut throughput = 0.0;
                        
                        for &node_hash in &node_hashes {
                            graph.update_node_state(&node_hash, NodeState::Verified).unwrap();
                            
                            let start = Instant::now();
                            consensus.process_node(node_hash).await.unwrap();
                            
                            // Simulate votes with network latency
                            for i in 0..100 {
                                // Add realistic network latency (20-100ms)
                                tokio::time::sleep(Duration::from_millis(20 + (rand::random::<u64>() % 80))).await;
                                let peer_hash = blake3::hash(&[i]);
                                consensus.record_vote(node_hash, peer_hash, true).await.unwrap();
                            }
                            
                            // Wait for finalization
                            if let Ok(ConsensusEvent::NodeFinalized(hash)) = events_rx.try_recv() {
                                assert_eq!(hash, node_hash);
                                let latency = start.elapsed();
                                latencies.push(latency);
                                total_latency += latency;
                                finalized += 1;
                                throughput = finalized as f64 / total_latency.as_secs_f64();
                            }
                        }
                        
                        // Calculate P99 latency
                        latencies.sort();
                        let p99_idx = (latencies.len() as f64 * 0.99) as usize;
                        p99_latency = latencies[p99_idx];

                        // Return aggregate metrics
                        (total_latency / finalized as u32, p99_latency, throughput)
                    })
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_vertex_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("vertex_processing");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    // Test concurrent validation performance
    let concurrent_sizes = vec![10, 50, 100];
    
    for size in concurrent_sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let (graph, node_hashes) = generate_test_dag(black_box(size));
                    let (processor, mut events_rx) = VertexProcessor::new(graph.clone());
                    
                    rt.block_on(async {
                        let start = Instant::now();
                        let mut processed = 0;
                        
                        // Submit vertices for concurrent processing
                        for &node_hash in &node_hashes {
                            graph.update_node_state(&node_hash, NodeState::Verified).unwrap();
                            processor.submit_vertex(node_hash).await.unwrap();
                            processed += 1;
                        }
                        
                        start.elapsed().as_nanos() as f64 / processed as f64
                    })
                })
            },
        );
    }
    
    group.finish();
}

fn benchmark_validation_methods(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("validation_methods");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    
    // Test different verification methods
    let methods = vec![
        VerificationMethod::BasicMajority,
        VerificationMethod::WeightedStake,
        VerificationMethod::MultiRound,
        VerificationMethod::QuantumResistant,
    ];
    
    for method in methods {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", method)),
            &method,
            |b, &method| {
                b.iter(|| {
                    let (graph, node_hashes) = generate_test_dag(100);
                    let (mut consensus, mut events_rx) = QrAvalanche::new(graph.clone());
                    consensus.add_verification_method(method);
                    
                    rt.block_on(async {
                        let start = Instant::now();
                        
                        for &node_hash in &node_hashes[..10] {
                            graph.update_node_state(&node_hash, NodeState::Verified).unwrap();
                            consensus.process_node(node_hash).await.unwrap();
                            
                            // Generate votes
                            for i in 0..20 {
                                let peer_hash = blake3::hash(&[i]);
                                consensus.record_vote(node_hash, peer_hash, true).await.unwrap();
                            }
                            
                            // Wait for finalization
                            while events_rx.try_recv().is_ok() {}
                        }
                        
                        start.elapsed()
                    })
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    finality_benches,
    benchmark_finality_latency,
    benchmark_vertex_processing,
    benchmark_validation_methods
);
criterion_main!(finality_benches);