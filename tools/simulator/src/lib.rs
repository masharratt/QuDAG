#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Network simulator for testing and validating QuDAG protocol behavior.

pub mod network;
pub mod metrics;
pub mod scenarios;
pub mod conditions;
pub mod attacks;
pub mod visualization;
pub mod reports;

#[cfg(test)]
mod tests {
    mod network_tests;
    mod metrics_tests;
    mod scenarios_tests;
    mod integration_tests;
    mod property_tests;
}