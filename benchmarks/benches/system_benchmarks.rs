use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use qudag_protocol::{Protocol, Config};
use qudag_simulator::NetworkSimulator;
use std::time::Duration;

fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for node_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("message_throughput", node_count), 
            node_count,
            |b, &n| {
                let simulator = NetworkSimulator::new(n);
                b.iter(|| {
                    simulator.simulate_message_flood(1000); // 1000 messages
                });
            }
        );
    }
    group.finish();
}

fn latency_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for node_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("message_latency", node_count),
            node_count,
            |b, &n| {
                let simulator = NetworkSimulator::new(n);
                b.iter(|| {
                    simulator.measure_message_latency();
                });
            }
        );
    }
    group.finish();
}

fn scalability_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(5);

    for node_count in [100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("network_scalability", node_count),
            node_count,
            |b, &n| {
                let simulator = NetworkSimulator::new(n);
                b.iter(|| {
                    simulator.simulate_full_network_load();
                });
            }
        );
    }
    group.finish();
}

fn resource_usage_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_usage");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    let sizes = [1000, 10000, 100000]; // Number of messages
    for size in sizes.iter() {
        group.bench_with_input(
            BenchmarkId::new("memory_usage", size),
            size,
            |b, &s| {
                let protocol = Protocol::new(Config::default());
                b.iter(|| {
                    protocol.measure_memory_usage(s);
                });
            }
        );
    }
    group.finish();
}

criterion_group!(
    system_benches,
    throughput_benchmarks,
    latency_benchmarks, 
    scalability_benchmarks,
    resource_usage_benchmarks
);
criterion_main!(system_benches);