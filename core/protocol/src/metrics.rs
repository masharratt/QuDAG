use metrics::{Counter, Gauge, Histogram};
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

/// Performance metrics for the QuDAG protocol
pub struct ProtocolMetrics {
    // Cryptographic metrics
    pub crypto_operations: Counter,
    pub key_cache_hits: Counter,
    pub key_cache_misses: Counter,
    pub crypto_latency: Histogram,
    
    // Network metrics
    pub messages_processed: Counter,
    pub message_latency: Histogram,
    pub active_connections: Gauge,
    pub connection_errors: Counter,
    pub route_cache_hits: Counter,
    
    // Consensus metrics
    pub consensus_rounds: Counter,
    pub consensus_latency: Histogram,
    pub dag_updates: Counter,
    pub node_count: Gauge,
    
    // Resource metrics
    pub memory_usage: Gauge,
    pub thread_count: Gauge,
    pub queue_depth: Gauge,
    
    // Last update timestamp
    last_update: Arc<RwLock<Instant>>,
    update_interval: Duration,
}

impl ProtocolMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            // Crypto metrics
            crypto_operations: Counter::new(),
            key_cache_hits: Counter::new(),
            key_cache_misses: Counter::new(),
            crypto_latency: Histogram::new(),
            
            // Network metrics
            messages_processed: Counter::new(),
            message_latency: Histogram::new(),
            active_connections: Gauge::new(),
            connection_errors: Counter::new(),
            route_cache_hits: Counter::new(),
            
            // Consensus metrics
            consensus_rounds: Counter::new(),
            consensus_latency: Histogram::new(),
            dag_updates: Counter::new(),
            node_count: Gauge::new(),
            
            // Resource metrics
            memory_usage: Gauge::new(),
            thread_count: Gauge::new(),
            queue_depth: Gauge::new(),
            
            // Update tracking
            last_update: Arc::new(RwLock::new(Instant::now())),
            update_interval: Duration::from_secs(1),
        }
    }
    
    /// Record cryptographic operation
    pub fn record_crypto_op(&self, latency: Duration) {
        self.crypto_operations.increment(1);
        self.crypto_latency.record(latency);
        self.maybe_flush_metrics();
    }
    
    /// Record message processing
    pub fn record_message(&self, latency: Duration) {
        self.messages_processed.increment(1);
        self.message_latency.record(latency);
        self.maybe_flush_metrics();
    }
    
    /// Record consensus round
    pub fn record_consensus(&self, latency: Duration) {
        self.consensus_rounds.increment(1);
        self.consensus_latency.record(latency);
        self.maybe_flush_metrics();
    }
    
    /// Update resource metrics
    pub fn update_resources(&self, memory: u64, threads: u64, queue: u64) {
        self.memory_usage.set(memory);
        self.thread_count.set(threads);
        self.queue_depth.set(queue);
        self.maybe_flush_metrics();
    }
    
    /// Get performance summary
    pub fn get_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            messages_per_second: self.messages_processed.get() as f64 / 
                self.last_update.read().elapsed().as_secs_f64(),
            avg_message_latency: self.message_latency.mean(),
            avg_consensus_latency: self.consensus_latency.mean(),
            active_connections: self.active_connections.get(),
            memory_usage: self.memory_usage.get(),
        }
    }
    
    // Flush metrics if update interval elapsed
    fn maybe_flush_metrics(&self) {
        let mut last_update = self.last_update.write();
        if last_update.elapsed() >= self.update_interval {
            // Reset histograms
            self.crypto_latency.clear();
            self.message_latency.clear();
            self.consensus_latency.clear();
            
            *last_update = Instant::now();
        }
    }
}

/// Performance summary
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub messages_per_second: f64,
    pub avg_message_latency: f64,
    pub avg_consensus_latency: f64,
    pub active_connections: u64,
    pub memory_usage: u64,
}