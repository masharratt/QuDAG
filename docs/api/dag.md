# DAG Module API

The `qudag_dag` module implements a directed acyclic graph (DAG) consensus mechanism using QR-Avalanche for quantum-resistant distributed consensus.

## Core Types

### DAGConsensus

The main consensus engine handling vertex ordering and finality.

```rust
pub struct DAGConsensus {
    // private fields
}

impl DAGConsensus {
    pub fn new() -> Self;
    pub fn with_config(config: ConsensusConfig) -> Self;
    pub fn add_vertex(&mut self, vertex: Vertex) -> Result<(), ConsensusError>;
    pub fn get_tips(&self) -> HashSet<String>;
    pub fn get_confidence(&self, id: &str) -> Option<Confidence>;
    pub fn get_total_order(&self) -> Result<Vec<String>, ConsensusError>;
}
```

### Vertex

Represents a message or transaction in the DAG.

```rust
pub struct Vertex {
    pub id: String,
    pub parents: Vec<String>,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub payload: Vec<u8>,
}
```

### Confidence

Finality status for vertices.

```rust
pub enum Confidence {
    Pending,
    HighConfidence,
    Final,
}
```

### ConsensusConfig

Configuration parameters for the consensus algorithm.

```rust
pub struct ConsensusConfig {
    pub query_sample_size: usize,
    pub finality_threshold: f64,
    pub finality_timeout: Duration,
    pub confirmation_depth: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            query_sample_size: 20,
            finality_threshold: 0.80,
            finality_timeout: Duration::from_secs(1),
            confirmation_depth: 4,
        }
    }
}
```

## Error Types

### ConsensusError

```rust
pub enum ConsensusError {
    InvalidVertex(String),
    ForkDetected(String),
    ValidationError(String),
    ConsensusTimeout,
    FinalityError(String),
}
```

## Example Usage

### Basic DAG Operations

```rust
use qudag_dag::{DAGConsensus, Vertex, ConsensusError};

// Create a new DAG consensus instance
let mut dag = DAGConsensus::new();

// Create and add a vertex
let vertex = Vertex {
    id: "vertex1".to_string(),
    parents: vec![],
    timestamp: 0,
    signature: vec![],
    payload: b"Hello".to_vec(),
};

// Add vertex to DAG
dag.add_vertex(vertex)?;

// Get current tips (vertices with no children)
let tips = dag.get_tips();

// Check vertex finality
if let Some(confidence) = dag.get_confidence("vertex1") {
    match confidence {
        Confidence::Final => println!("Vertex is final"),
        Confidence::HighConfidence => println!("Vertex has high confidence"),
        Confidence::Pending => println!("Vertex is still pending"),
    }
}
```

### Custom Configuration

```rust
use qudag_dag::{DAGConsensus, ConsensusConfig};
use std::time::Duration;

let config = ConsensusConfig {
    query_sample_size: 30,
    finality_threshold: 0.85,
    finality_timeout: Duration::from_secs(2),
    confirmation_depth: 5,
};

let dag = DAGConsensus::with_config(config);
```

### Error Handling

```rust
use qudag_dag::{DAGConsensus, ConsensusError};

fn handle_vertex_addition(dag: &mut DAGConsensus, vertex: Vertex) {
    match dag.add_vertex(vertex) {
        Ok(()) => println!("Vertex added successfully"),
        Err(ConsensusError::InvalidVertex(msg)) => {
            eprintln!("Invalid vertex: {}", msg);
        }
        Err(ConsensusError::ForkDetected(msg)) => {
            eprintln!("Fork detected: {}", msg);
            // Implement fork resolution strategy
        }
        Err(e) => eprintln!("Error adding vertex: {}", e),
    }
}
```

## Best Practices

1. **Vertex Creation**
   - Always ensure parent vertices exist before adding new vertices
   - Validate vertex signatures before addition
   - Use monotonically increasing timestamps

2. **Performance Optimization**
   - Monitor the DAG size and prune old vertices when possible
   - Adjust consensus parameters based on network conditions
   - Cache commonly accessed vertices

3. **Fork Handling**
   - Implement proper fork detection and resolution
   - Consider using a fork choice rule
   - Maintain consistent total ordering

## Configuration Guidelines

1. **Query Sample Size**
   - Larger values increase security but reduce performance
   - Recommended range: 20-50 peers
   - Adjust based on network size

2. **Finality Threshold**
   - Higher values increase security but may delay finality
   - Recommended range: 0.75-0.85
   - Consider network latency when adjusting

3. **Confirmation Depth**
   - Affects confidence in finality
   - Recommended range: 4-6 confirmations
   - Balance between security and latency