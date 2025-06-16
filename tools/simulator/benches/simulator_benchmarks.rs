use criterion::{criterion_group, criterion_main, Criterion};
use qudag_simulator::{
    network::{NetworkSimulator, SimulatorConfig},
    scenarios::{ScenarioConfig, NetworkConditions},
};
use std::time::Duration;

pub fn benchmark_simulator(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulator");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Network setup benchmark
    group.bench_function("network_setup", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let config = SimulatorConfig {
                        node_count: 10,
                        latency_ms: 50,
                        drop_rate: 0.01,
                        partition_prob: 0.0,
                    };

                    let (mut sim, _) = NetworkSimulator::new(config);

                    // Add nodes
                    for _ in 0..10 {
                        sim.add_node(Default::default()).await.unwrap();
                    }

                    sim
                })
        })
    });

    // Message routing benchmark
    group.bench_function("message_routing", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    let config = ScenarioConfig {
                        node_count: 10,
                        duration: Duration::from_secs(10),
                        msg_rate: 1000.0,
                        network: NetworkConditions {
                            latency: Duration::from_millis(50),
                            loss_rate: 0.01,
                            partition_prob: 0.0,
                        },
                    };

                    qudag_simulator::scenarios::test_basic_connectivity(config).await.unwrap()
                })
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_simulator);
criterion_main!(benches);