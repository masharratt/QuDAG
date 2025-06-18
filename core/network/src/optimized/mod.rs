//! Optimized network implementations for high-performance message processing
//! 
//! This module provides optimized implementations with focus on:
//! - Zero-copy message processing
//! - Adaptive batching algorithms
//! - Lock-free data structures
//! - NUMA-aware memory allocation

// pub mod zero_copy;
pub mod adaptive_batch;
// pub mod lock_free;
// pub mod numa_aware;

// pub use zero_copy::ZeroCopyConnection;
pub use adaptive_batch::AdaptiveBatcher;
// pub use lock_free::LockFreeMessageQueue;
// pub use numa_aware::NumaAllocator;