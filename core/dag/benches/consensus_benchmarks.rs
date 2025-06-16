#[macro_use]
extern crate criterion;

use criterion::{Criterion, BenchmarkId, Throughput};
use qudag_dag::{DAGConsensus, Vertex, ConsensusConfig};
use std::time::Duration;

fn create_test_vertex(id: &str, parents: Vec<&str>, timestamp: u64) -> Vertex {
    Vertex {
        id: id.to_string(),
        parents: parents.into_iter().map(String::from).collect(),
        timestamp,
        signature: vec![],
        payload: vec![],
    }
}

// Benchmark vertex addition
fn bench_add_vertex(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_addition");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut dag = DAGConsensus::new();
                for i in 0..size {
                    let parents = if i == 0 { 
                        vec![] 
                    } else { 
                        vec![format!("V{}", i-1)]
                    };
                    let vertex = Vertex {
                        id: format!("V{}", i),
                        parents,
                        timestamp: i as u64,
                        signature: vec![],
                        payload: vec![],
                    };
                    dag.add_vertex(vertex).unwrap();
                }
            });
        });
    }
    group.finish();
}

// Benchmark consensus finalization
fn bench_finalization(c: &mut Criterion) {
    let mut group = c.benchmark_group("finalization");
    group.measurement_time(Duration::from_secs(10));
    
    for depth in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            b.iter(|| {
                let config = ConsensusConfig {
                    query_sample_size: 5,
                    finality_threshold: 0.8,
                    finality_timeout: Duration::from_secs(5),
                    confirmation_depth: depth,
                };
                
                let mut dag = DAGConsensus::with_config(config);
                
                // Create a chain of vertices
                for i in 0..depth {
                    let parents = if i == 0 { 
                        vec![] 
                    } else { 
                        vec![format!("V{}", i-1)]
                    };
                    let vertex = Vertex {
                        id: format!("V{}", i),
                        parents,
                        timestamp: i as u64,
                        signature: vec![],
                        payload: vec![],
                    };
                    dag.add_vertex(vertex).unwrap();
                }
                
                // Wait for finalization of all vertices
                std::thread::sleep(Duration::from_millis(50));
            });
        });
    }
    group.finish();
}

// Benchmark concurrent path processing
fn bench_concurrent_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_paths");
    group.measurement_time(Duration::from_secs(10));
    
    for paths in [2, 4, 8].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(paths), paths, |b, &paths| {
            b.iter(|| {
                let mut dag = DAGConsensus::new();
                
                // Create genesis vertex
                let genesis = create_test_vertex("genesis", vec![], 0);
                dag.add_vertex(genesis).unwrap();
                
                // Create concurrent paths
                for path in 0..paths {
                    for depth in 1..5 {
                        let id = format!("P{}V{}", path, depth);
                        let parent = if depth == 1 {
                            "genesis".to_string()
                        } else {
                            format!("P{}V{}", path, depth-1)
                        };
                        
                        let vertex = create_test_vertex(&id, vec![&parent], depth as u64);
                        dag.add_vertex(vertex).unwrap();
                    }
                }
                
                // Wait for processing
                std::thread::sleep(Duration::from_millis(20));
            });
        });
    }
    group.finish();
}

// Benchmark DAG operations under load
fn bench_dag_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("dag_operations");
    group.measurement_time(Duration::from_secs(10));
    
    let operations = vec![
        ("small_dag", 50),
        ("medium_dag", 200),
        ("large_dag", 500),
    ];
    
    for (name, size) in operations {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut dag = DAGConsensus::new();
                
                // Add vertices in a mesh pattern
                for i in 0..size {
                    let mut parents = Vec::new();
                    if i > 0 { parents.push(format!("V{}", i-1)); }
                    if i > 4 { parents.push(format!("V{}", i-5)); }
                    
                    let vertex = Vertex {
                        id: format!("V{}", i),
                        parents,
                        timestamp: i as u64,
                        signature: vec![],
                        payload: vec![],
                    };
                    dag.add_vertex(vertex).unwrap();
                }
                
                // Perform various operations
                dag.get_tips();
                if size > 0 {
                    dag.get_confidence(&format!("V{}", size/2));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_add_vertex,
    bench_finalization,
    bench_concurrent_paths,
    bench_dag_operations
);
criterion_main!(benches);