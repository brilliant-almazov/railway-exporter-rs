//! # Railway Exporter
//!
//! A Prometheus exporter for monitoring Railway.app billing and usage metrics.
//!
//! This library provides functionality to:
//! - Query Railway GraphQL API for usage data
//! - Calculate costs based on Hobby/Pro pricing plans
//! - Expose metrics in Prometheus format
//!
//! ## Example
//!
//! ```rust
//! use railway_exporter::{Metrics, pricing};
//!
//! let metrics = Metrics::new();
//!
//! // Get price for CPU usage on Pro plan
//! let cpu_price = pricing::get_price("pro", "CPU_USAGE");
//! assert_eq!(cpu_price, 0.000231);
//! ```

pub mod config;
pub mod metrics;
pub mod pricing;
pub mod railway;

pub use config::{Config, Plan};
pub use metrics::Metrics;
