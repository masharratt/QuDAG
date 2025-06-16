#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Performance benchmarks for the QuDAG protocol.
//!
//! This crate provides comprehensive benchmarking tools and utilities for measuring
//! the performance characteristics of the QuDAG protocol, including throughput,
//! latency, scalability, and resource usage metrics.

pub mod system;
pub mod scenarios;
pub mod metrics;
pub mod utils;

pub use utils::{BenchmarkMetrics, ResourceMonitor};

/// Re-export commonly used types for benchmarking
pub use criterion;
pub use metrics;