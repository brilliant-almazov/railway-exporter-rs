//! # Railway Exporter
//!
//! A Prometheus exporter for monitoring Railway.app billing and usage metrics.
//!
//! This library provides functionality to:
//! - Query Railway GraphQL API for usage data
//! - Calculate costs based on Hobby/Pro pricing plans
//! - Expose metrics in Prometheus format
//! - Serve metrics via HTTP (Prometheus, JSON, WebSocket)
//!
//! ## Architecture
//!
//! - `config` - YAML configuration loading
//! - `metrics` - Prometheus metrics definitions
//! - `pricing` - Railway pricing calculations
//! - `client` - Railway GraphQL API client
//! - `types` - Shared data types
//! - `state` - Application state management
//! - `collector` - Metrics collection logic
//! - `server` - HTTP server and handlers

pub mod client;
pub mod collector;
pub mod config;
pub mod handlers;
pub mod metrics;
pub mod pricing;
pub mod server;
pub mod state;
pub mod types;
pub mod utils;

pub use config::{Config, Plan};
pub use metrics::Metrics;
pub use state::AppState;

#[cfg(test)]
#[path = "config_test.rs"]
mod config_test;

#[cfg(test)]
#[path = "types_test.rs"]
mod types_test;

#[cfg(test)]
#[path = "state_test.rs"]
mod state_test;

#[cfg(test)]
#[path = "client_test.rs"]
mod client_test;

#[cfg(test)]
#[path = "collector_test.rs"]
mod collector_test;

#[cfg(test)]
#[path = "server_test.rs"]
mod server_test;

#[cfg(test)]
#[path = "metrics_test.rs"]
mod metrics_test;

#[cfg(test)]
#[path = "pricing_test.rs"]
mod pricing_test;
