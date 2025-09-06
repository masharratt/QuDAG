# QuDAG DAG Consensus System

QuDAG implements a sophisticated **DAG (Directed Acyclic Graph) consensus** mechanism using the **QR-Avalanche** algorithm, providing high-throughput, low-latency consensus with quantum-resistant security guarantees.

## Overview

The DAG consensus system provides:

- **QR-Avalanche Consensus**: Quantum-resistant adaptation of Avalanche protocol
- **High Throughput**: 10,000+ transactions per second theoretical capacity
- **Low Latency**: Sub-second finality under normal conditions  
- **Byzantine Fault Tolerance**: Secure with < 1/3 adversarial nodes
- **Parallel Processing**: Concurrent vertex validation and consensus
- **Quantum Security**: ML-DSA signatures on all vertices

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                 DAG Consensus System                    │
├─────────────────────────────────────────────────────────┤
│  Application Layer                                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │   Message   │ │  Vertex     │ │  Consensus  │      │
│  │  Creation   │ │ Management  │ │   Query     │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
├─────────────────────────────────────────────────────────┤
│  Consensus Layer                                       │
│  ┌─────────────────────────────────────────────────────┐ │
│  │            QR-Avalanche Engine                      │ │
│  │  ├── Vertex Validation   ├── Conflict Detection    │ │
│  │  ├── Preference Tracking ├── Confidence Building   │ │
│  │  └── Finality Decision   └── State Synchronization │ │
│  └─────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  Graph Layer                                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐      │
│  │    DAG      │ │ Tip Select  │ │   Parent    │      │
│  │  Structure  │ │ Algorithm   │ │ Validation  │      │
│  └─────────────┘ └─────────────┘ └─────────────┘      │
├─────────────────────────────────────────────────────────┤
│  Cryptographic Layer                                   │
│  │   ML-DSA    │   BLAKE3    │  Quantum Fingerprints  │
└─────────────────────────────────────────────────────────┘
```

## Core Concepts

### DAG Structure

The DAG is composed of vertices (nodes) connected by directed edges (references):

```rust
pub struct Vertex {
    pub id: VertexId,
    pub parents: Vec<VertexId>,
    pub payload: Vec<u8>,
    pub timestamp: u64,
    pub signature: MlDsaSignature,
    pub nonce: u64,
}

pub struct DAG {
    vertices: HashMap<VertexId, Vertex>,
    tips: HashSet<VertexId>,
    finalized: HashSet<VertexId>,
    conflicts: HashMap<VertexId, HashSet<VertexId>>,
}
```

### Vertex Relationships

```
    Genesis
       │
    ┌──▼──┐    ┌─────┐
    │  A  │    │  B  │
    └──┬──┘    └──┬──┘
       │          │
    ┌──▼──┐    ┌──▼──┐
    │  C  │    │  D  │
    └──┬──┘    └──┬──┘
       │          │
       └──┐   ┌───┘
          ▼   ▼
       ┌─────────┐
       │    E    │  ← New vertex references both C and D
       └─────────┘
```

## QR-Avalanche Consensus

### Algorithm Overview

QR-Avalanche adapts the Avalanche consensus protocol with quantum-resistant cryptography:

1. **Vertex Creation**: New vertices reference multiple parents (tips)
2. **Network Propagation**: Vertices broadcast to network peers
3. **Preference Sampling**: Nodes query network for vertex preferences
4. **Confidence Building**: Repeated sampling builds confidence in decisions
5. **Finality**: High confidence triggers irreversible finality

### Implementation

```rust
pub struct QRAvalanche {
    dag: Arc<RwLock<DAG>>,
    preferences: HashMap<VertexId, Preference>,
    confidence: HashMap<VertexId, f64>,
    finality_threshold: f64,
    sample_size: usize,
    crypto: Arc<CryptoManager>,
}

#[derive(Clone, Debug)]
pub struct Preference {
    vertex_id: VertexId,
    preferred: bool,
    confidence: f64,
    last_updated: Instant,
}

impl QRAvalanche {
    pub async fn add_vertex(&mut self, vertex: Vertex) -> Result<()> {
        // 1. Validate vertex structure and signature
        self.validate_vertex(&vertex).await?;
        
        // 2. Check for conflicts with existing vertices
        let conflicts = self.detect_conflicts(&vertex).await?;
        
        // 3. Add to DAG
        let mut dag = self.dag.write().await;
        dag.add_vertex(vertex.clone())?;
        
        // 4. Update tips
        dag.update_tips(&vertex);
        
        // 5. Initialize preference
        self.preferences.insert(vertex.id, Preference {
            vertex_id: vertex.id,
            preferred: true,
            confidence: 0.5,
            last_updated: Instant::now(),
        });
        
        // 6. Start consensus process
        if !conflicts.is_empty() {
            self.resolve_conflicts(vertex.id, conflicts).await?;
        }
        
        Ok(())
    }
    
    pub async fn query_preferences(&self, vertex_id: VertexId) -> Result<f64> {
        let sample_peers = self.select_sample_peers().await?;
        let mut positive_responses = 0;
        
        for peer in sample_peers.iter() {
            let response = self.network.query_preference(peer, vertex_id).await?;
            if response.preferred {
                positive_responses += 1;
            }
        }
        
        Ok(positive_responses as f64 / sample_peers.len() as f64)
    }
    
    pub async fn update_confidence(&mut self, vertex_id: VertexId) -> Result<()> {
        let preference_ratio = self.query_preferences(vertex_id).await?;
        
        if let Some(pref) = self.preferences.get_mut(&vertex_id) {
            // Update confidence using exponential moving average
            let alpha = 0.1; // Learning rate
            pref.confidence = alpha * preference_ratio + (1.0 - alpha) * pref.confidence;
            pref.preferred = preference_ratio > 0.5;
            pref.last_updated = Instant::now();
            
            // Check for finality
            if pref.confidence >= self.finality_threshold {
                self.finalize_vertex(vertex_id).await?;
            }
        }
        
        Ok(())
    }
    
    async fn finalize_vertex(&mut self, vertex_id: VertexId) -> Result<()> {
        let mut dag = self.dag.write().await;
        
        // Mark as finalized
        dag.finalized.insert(vertex_id);
        
        // Finalize all ancestors if not already finalized
        let ancestors = dag.get_ancestors(vertex_id)?;
        for ancestor_id in ancestors {
            dag.finalized.insert(ancestor_id);
        }
        
        // Remove from tips if present
        dag.tips.remove(&vertex_id);
        
        // Emit finality event
        self.emit_finality_event(vertex_id).await;
        
        tracing::info!("Vertex {} finalized", vertex_id);
        Ok(())
    }
}
```

### Consensus Properties

| Property | Value | Description |
|----------|-------|-------------|
| **Safety** | Byzantine Fault Tolerant | Correct decisions with < 1/3 adversarial nodes |
| **Liveness** | Progress Guarantee | System makes progress under network assumptions |
| **Finality** | Probabilistic | High confidence finality (>99.9%) |
| **Throughput** | 10,000+ TPS | Theoretical maximum transactions per second |
| **Latency** | <1 second | P99 finality latency under normal conditions |
| **Memory** | O(n log n) | Memory complexity for n vertices |

## Tip Selection Algorithm

Selecting appropriate parent vertices is crucial for DAG health:

### Weighted Random Selection

```rust
pub struct TipSelector {
    dag: Arc<RwLock<DAG>>,
    selection_algorithm: SelectionAlgorithm,
    max_parents: usize,
}

#[derive(Clone, Debug)]
pub enum SelectionAlgorithm {
    UniformRandom,
    WeightedByAge,
    WeightedByConfidence,
    URTS, // Uniform Random Tip Selection
    CW,   // Cumulative Weight
}

impl TipSelector {
    pub async fn select_tips(&self, count: usize) -> Result<Vec<VertexId>> {
        let dag = self.dag.read().await;
        let tips: Vec<_> = dag.tips.iter().cloned().collect();
        
        if tips.is_empty() {
            return Err(TipSelectionError::NoTips);
        }
        
        match self.selection_algorithm {
            SelectionAlgorithm::UniformRandom => {
                Ok(self.uniform_random_selection(&tips, count))
            }
            SelectionAlgorithm::WeightedByConfidence => {
                Ok(self.confidence_weighted_selection(&dag, &tips, count).await?)
            }
            SelectionAlgorithm::URTS => {
                Ok(self.urts_selection(&dag, &tips, count).await?)
            }
            _ => Ok(self.uniform_random_selection(&tips, count))
        }
    }
    
    fn uniform_random_selection(&self, tips: &[VertexId], count: usize) -> Vec<VertexId> {
        use rand::seq::SliceRandom;
        
        let mut rng = rand::thread_rng();
        let actual_count = std::cmp::min(count, tips.len());
        
        tips.choose_multiple(&mut rng, actual_count).cloned().collect()
    }
    
    async fn confidence_weighted_selection(
        &self, 
        dag: &DAG, 
        tips: &[VertexId], 
        count: usize
    ) -> Result<Vec<VertexId>> {
        // Calculate weights based on confidence and age
        let mut weighted_tips = Vec::new();
        
        for &tip_id in tips {
            let vertex = dag.get_vertex(tip_id)?;
            let confidence = self.get_vertex_confidence(tip_id).await?;
            let age_factor = self.calculate_age_factor(vertex.timestamp);
            
            let weight = confidence * age_factor;
            weighted_tips.push((tip_id, weight));
        }
        
        // Sort by weight and select top candidates
        weighted_tips.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(weighted_tips.into_iter()
            .take(count)
            .map(|(id, _)| id)
            .collect())
    }
    
    fn calculate_age_factor(&self, timestamp: u64) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let age = now.saturating_sub(timestamp);
        
        // Exponential decay: newer vertices have higher weight
        (-0.001 * age as f64).exp()
    }
}
```

## Conflict Detection and Resolution

### Conflict Types

```rust
#[derive(Clone, Debug)]
pub enum ConflictType {
    DoubleSpend,        // Same UTXO spent in multiple vertices
    Ordering,           // Conflicting vertex ordering
    StateTransition,    // Invalid state transitions
    Temporal,           // Timestamp conflicts
}

pub struct ConflictDetector {
    dag: Arc<RwLock<DAG>>,
    conflict_graph: HashMap<VertexId, HashSet<VertexId>>,
}

impl ConflictDetector {
    pub async fn detect_conflicts(&mut self, vertex: &Vertex) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        
        // Check for double-spend conflicts
        if let Some(double_spend) = self.check_double_spend(vertex).await? {
            conflicts.push(double_spend);
        }
        
        // Check for ordering conflicts
        if let Some(ordering) = self.check_ordering_conflict(vertex).await? {
            conflicts.push(ordering);
        }
        
        // Check for temporal conflicts
        if let Some(temporal) = self.check_temporal_conflict(vertex).await? {
            conflicts.push(temporal);
        }
        
        Ok(conflicts)
    }
    
    async fn resolve_conflict(&mut self, conflict: Conflict) -> Result<Resolution> {
        match conflict.conflict_type {
            ConflictType::DoubleSpend => {
                // Use first-seen rule with confidence weighting
                let resolutions = self.evaluate_double_spend_options(conflict).await?;
                Ok(self.select_highest_confidence_resolution(resolutions))
            }
            ConflictType::Ordering => {
                // Use topological ordering with timestamp preference
                Ok(self.resolve_ordering_conflict(conflict).await?)
            }
            ConflictType::Temporal => {
                // Reject vertices with invalid timestamps
                Ok(Resolution::Reject(conflict.conflicting_vertices))
            }
            _ => Ok(Resolution::Accept(conflict.base_vertex))
        }
    }
}
```

## Performance Optimizations

### Parallel Vertex Processing

```rust
use tokio::task::JoinSet;
use std::sync::Arc;

pub struct ParallelConsensus {
    consensus: Arc<QRAvalanche>,
    worker_pool: Arc<ThreadPool>,
    batch_size: usize,
}

impl ParallelConsensus {
    pub async fn process_vertices_batch(&self, vertices: Vec<Vertex>) -> Result<()> {
        let mut join_set = JoinSet::new();
        
        // Process vertices in parallel batches
        for batch in vertices.chunks(self.batch_size) {
            let batch_clone = batch.to_vec();
            let consensus = Arc::clone(&self.consensus);
            
            join_set.spawn(async move {
                consensus.process_batch(batch_clone).await
            });
        }
        
        // Wait for all batches to complete
        while let Some(result) = join_set.join_next().await {
            result??; // Handle both JoinError and consensus errors
        }
        
        Ok(())
    }
    
    pub async fn parallel_tip_selection(&self, requests: usize) -> Result<Vec<Vec<VertexId>>> {
        let mut join_set = JoinSet::new();
        
        for _ in 0..requests {
            let consensus = Arc::clone(&self.consensus);
            join_set.spawn(async move {
                consensus.tip_selector.select_tips(2).await
            });
        }
        
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            results.push(result??);
        }
        
        Ok(results)
    }
}
```

### Caching and Memory Management

```rust
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct DAGCache {
    vertex_cache: LruCache<VertexId, Vertex>,
    preference_cache: LruCache<VertexId, Preference>,
    conflict_cache: LruCache<VertexId, Vec<Conflict>>,
    metrics: Arc<CacheMetrics>,
}

impl DAGCache {
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap();
        
        Self {
            vertex_cache: LruCache::new(capacity),
            preference_cache: LruCache::new(capacity),
            conflict_cache: LruCache::new(capacity),
            metrics: Arc::new(CacheMetrics::new()),
        }
    }
    
    pub fn get_vertex(&mut self, id: VertexId) -> Option<&Vertex> {
        match self.vertex_cache.get(&id) {
            Some(vertex) => {
                self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
                Some(vertex)
            }
            None => {
                self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }
    
    pub fn put_vertex(&mut self, id: VertexId, vertex: Vertex) {
        self.vertex_cache.put(id, vertex);
        self.metrics.entries_added.fetch_add(1, Ordering::Relaxed);
    }
}
```

## DAG Visualization

### Graph Structure Analysis

```bash
# Analyze DAG structure
qudag dag analyze --depth 10 --format json

# Visualize DAG topology
qudag dag visualize --output dag.svg --layout hierarchical

# Export DAG for analysis
qudag dag export --format graphml --output dag.graphml

# Performance metrics
qudag dag metrics --realtime --interval 5s
```

**Example Analysis Output:**
```json
{
  "dag_statistics": {
    "total_vertices": 15847,
    "finalized_vertices": 15203,
    "pending_vertices": 644,
    "tips_count": 23,
    "average_parents_per_vertex": 2.3,
    "max_depth": 1847,
    "consensus_efficiency": 0.96
  },
  "performance_metrics": {
    "throughput": {
      "vertices_per_second": 152.7,
      "finality_rate": 147.8
    },
    "latency": {
      "average_confirmation_time_ms": 847,
      "p99_confirmation_time_ms": 1847,
      "network_propagation_ms": 125
    }
  },
  "network_health": {
    "active_nodes": 47,
    "consensus_participation": 0.89,
    "byzantine_fault_tolerance": 0.67
  }
}
```

## Configuration

### Consensus Parameters

```toml
[consensus]
algorithm = "QR-Avalanche"
finality_threshold = 0.95
sample_size = 20
max_query_rounds = 10
confidence_decay = 0.99

[dag]
max_parents_per_vertex = 8
tip_selection_algorithm = "WeightedByConfidence"
conflict_resolution = "FirstSeenWithConfidence"
max_pending_vertices = 1000

[performance]
parallel_processing = true
worker_threads = 8
batch_size = 100
cache_size = 10000
enable_optimizations = true

[security]
signature_verification = true
quantum_resistant = true
replay_protection = true
timestamp_validation = true
```

### Runtime Configuration

```rust
pub struct ConsensusConfig {
    pub finality_threshold: f64,
    pub sample_size: usize,
    pub max_parents: usize,
    pub tip_selection: TipSelectionAlgorithm,
    pub parallel_processing: bool,
    pub cache_size: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            finality_threshold: 0.95,
            sample_size: 20,
            max_parents: 8,
            tip_selection: TipSelectionAlgorithm::WeightedByConfidence,
            parallel_processing: true,
            cache_size: 10000,
        }
    }
}
```

## Integration Examples

### Application Integration

```rust
use qudag_dag::{QrDag, Vertex, VertexId};

pub struct QuDAGApplication {
    dag: Arc<Mutex<QrDag>>,
    event_handler: Arc<EventHandler>,
}

impl QuDAGApplication {
    pub async fn submit_transaction(&self, data: Vec<u8>) -> Result<VertexId> {
        let mut dag = self.dag.lock().await;
        
        // Select parents (tips) for new vertex
        let parents = dag.select_tips(2).await?;
        
        // Create new vertex
        let vertex_id = VertexId::new();
        let vertex = Vertex::new(vertex_id, data, parents);
        
        // Add to DAG and start consensus
        dag.add_vertex(vertex).await?;
        
        // Return vertex ID for tracking
        Ok(vertex_id)
    }
    
    pub async fn wait_for_finality(&self, vertex_id: VertexId) -> Result<()> {
        loop {
            let dag = self.dag.lock().await;
            if dag.is_finalized(vertex_id) {
                return Ok(());
            }
            
            drop(dag);
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
```

This DAG consensus system provides the foundation for high-performance, quantum-resistant distributed consensus in QuDAG, enabling secure and efficient coordination across the decentralized network.