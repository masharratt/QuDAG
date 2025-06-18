#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Network simulator for testing and validating QuDAG protocol behavior.

pub mod attacks;
pub mod conditions;
pub mod metrics;
pub mod network;
pub mod reports;
pub mod scenarios;
pub mod visualization;

#[cfg(test)]
mod tests {
    mod integration_tests;
    mod metrics_tests;
    mod network_tests;
    mod property_tests;
    mod scenarios_tests;
}
