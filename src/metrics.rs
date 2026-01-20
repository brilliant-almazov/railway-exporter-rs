//! Prometheus metrics definitions and management.
//!
//! This module provides the metrics registry for Railway usage data.
//!
//! ## Metrics Exposed
//!
//! ### Per-Service Metrics (labels: service, project, icon, group)
//!
//! | Metric | Description |
//! |--------|-------------|
//! | `railway_cpu_usage_vcpu_minutes` | CPU usage in vCPU-minutes |
//! | `railway_memory_usage_gb_minutes` | Memory usage in GB-minutes |
//! | `railway_disk_usage_gb_minutes` | Disk usage in GB-minutes |
//! | `railway_network_tx_gb` | Network egress in GB |
//! | `railway_service_cost_usd` | Current cost in USD |
//! | `railway_service_estimated_monthly_usd` | Estimated monthly cost |
//!
//! ### Per-Project Metrics (labels: project)
//!
//! | Metric | Description |
//! |--------|-------------|
//! | `railway_current_usage_usd` | Total current usage |
//! | `railway_estimated_monthly_usd` | Estimated monthly total |
//! | `railway_daily_average_usd` | Average daily spending |
//! | `railway_days_in_billing_period` | Days elapsed |
//! | `railway_days_remaining_in_month` | Days remaining |
//! | `railway_exporter_last_scrape_timestamp` | Last scrape timestamp |
//! | `railway_exporter_scrape_duration_seconds` | Scrape duration |
//! | `railway_api_up` | API availability (1/0) |
//!
//! ### Exporter Process Metrics (no labels)
//!
//! | Metric | Description |
//! |--------|-------------|
//! | `railway_exporter_memory_bytes` | Exporter memory usage |
//! | `railway_exporter_cpu_percent` | Exporter CPU usage |

use prometheus::{Encoder, GaugeVec, Opts, Registry, TextEncoder};
use sysinfo::System;

/// Prometheus metrics registry for Railway data.
pub struct Metrics {
    // Per-service metrics (labels: service, project, icon, group)
    /// CPU usage in vCPU-minutes per service.
    pub cpu_usage: GaugeVec,
    /// Memory usage in GB-minutes per service.
    pub memory_usage: GaugeVec,
    /// Disk usage in GB-minutes per service.
    pub disk_usage: GaugeVec,
    /// Network egress in GB per service.
    pub network_tx: GaugeVec,
    /// Current cost in USD per service.
    pub service_cost: GaugeVec,
    /// Estimated monthly cost in USD per service.
    pub service_estimated_monthly: GaugeVec,

    // Per-project metrics (labels: project)
    /// Total current usage in USD per project.
    pub current_usage: GaugeVec,
    /// Estimated monthly total in USD per project.
    pub estimated_monthly: GaugeVec,
    /// Average daily spending in USD per project.
    pub daily_average: GaugeVec,
    /// Days elapsed in billing period.
    pub days_in_billing_period: GaugeVec,
    /// Days remaining in month.
    pub days_remaining_in_month: GaugeVec,
    /// Timestamp of last successful scrape.
    pub last_scrape_timestamp: GaugeVec,
    /// Duration of API scrape in seconds.
    pub scrape_duration_seconds: GaugeVec,
    /// Whether Railway API is reachable (1=up, 0=down).
    pub api_up: GaugeVec,

    // Exporter process metrics (no labels)
    /// Memory usage of exporter process in bytes.
    pub exporter_memory_bytes: GaugeVec,
    /// CPU usage percentage of exporter process.
    pub exporter_cpu_percent: GaugeVec,

    /// The Prometheus registry holding all metrics.
    pub registry: Registry,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Creates a new metrics registry with all Railway metrics registered.
    pub fn new() -> Self {
        let registry = Registry::new();

        // Label sets
        let service_labels = &["service", "project", "icon", "group"];
        let project_labels = &["project"];
        let no_labels: &[&str] = &[];

        // Per-service metrics
        let cpu_usage = GaugeVec::new(
            Opts::new(
                "railway_cpu_usage_vcpu_minutes",
                "CPU usage in vCPU-minutes",
            ),
            service_labels,
        )
        .unwrap();

        let memory_usage = GaugeVec::new(
            Opts::new(
                "railway_memory_usage_gb_minutes",
                "Memory usage in GB-minutes",
            ),
            service_labels,
        )
        .unwrap();

        let disk_usage = GaugeVec::new(
            Opts::new("railway_disk_usage_gb_minutes", "Disk usage in GB-minutes"),
            service_labels,
        )
        .unwrap();

        let network_tx = GaugeVec::new(
            Opts::new("railway_network_tx_gb", "Network egress in GB"),
            service_labels,
        )
        .unwrap();

        let service_cost = GaugeVec::new(
            Opts::new("railway_service_cost_usd", "Current service cost in USD"),
            service_labels,
        )
        .unwrap();

        let service_estimated_monthly = GaugeVec::new(
            Opts::new(
                "railway_service_estimated_monthly_usd",
                "Estimated monthly service cost in USD",
            ),
            service_labels,
        )
        .unwrap();

        // Per-project metrics
        let current_usage = GaugeVec::new(
            Opts::new("railway_current_usage_usd", "Total current usage in USD"),
            project_labels,
        )
        .unwrap();

        let estimated_monthly = GaugeVec::new(
            Opts::new(
                "railway_estimated_monthly_usd",
                "Estimated monthly total in USD",
            ),
            project_labels,
        )
        .unwrap();

        let daily_average = GaugeVec::new(
            Opts::new("railway_daily_average_usd", "Average daily spending in USD"),
            project_labels,
        )
        .unwrap();

        let days_in_billing_period = GaugeVec::new(
            Opts::new(
                "railway_days_in_billing_period",
                "Days elapsed in billing period",
            ),
            project_labels,
        )
        .unwrap();

        let days_remaining_in_month = GaugeVec::new(
            Opts::new("railway_days_remaining_in_month", "Days remaining in month"),
            project_labels,
        )
        .unwrap();

        let last_scrape_timestamp = GaugeVec::new(
            Opts::new(
                "railway_exporter_last_scrape_timestamp",
                "Unix timestamp of last successful scrape",
            ),
            project_labels,
        )
        .unwrap();

        let scrape_duration_seconds = GaugeVec::new(
            Opts::new(
                "railway_exporter_scrape_duration_seconds",
                "Duration of API scrape in seconds",
            ),
            project_labels,
        )
        .unwrap();

        let api_up = GaugeVec::new(
            Opts::new(
                "railway_api_up",
                "Whether Railway API is reachable (1=up, 0=down)",
            ),
            project_labels,
        )
        .unwrap();

        // Exporter process metrics
        let exporter_memory_bytes = GaugeVec::new(
            Opts::new(
                "railway_exporter_memory_bytes",
                "Memory usage of exporter process in bytes",
            ),
            no_labels,
        )
        .unwrap();

        let exporter_cpu_percent = GaugeVec::new(
            Opts::new(
                "railway_exporter_cpu_percent",
                "CPU usage percentage of exporter process",
            ),
            no_labels,
        )
        .unwrap();

        // Register all metrics
        registry.register(Box::new(cpu_usage.clone())).unwrap();
        registry.register(Box::new(memory_usage.clone())).unwrap();
        registry.register(Box::new(disk_usage.clone())).unwrap();
        registry.register(Box::new(network_tx.clone())).unwrap();
        registry.register(Box::new(service_cost.clone())).unwrap();
        registry
            .register(Box::new(service_estimated_monthly.clone()))
            .unwrap();
        registry.register(Box::new(current_usage.clone())).unwrap();
        registry
            .register(Box::new(estimated_monthly.clone()))
            .unwrap();
        registry.register(Box::new(daily_average.clone())).unwrap();
        registry
            .register(Box::new(days_in_billing_period.clone()))
            .unwrap();
        registry
            .register(Box::new(days_remaining_in_month.clone()))
            .unwrap();
        registry
            .register(Box::new(last_scrape_timestamp.clone()))
            .unwrap();
        registry
            .register(Box::new(scrape_duration_seconds.clone()))
            .unwrap();
        registry.register(Box::new(api_up.clone())).unwrap();
        registry
            .register(Box::new(exporter_memory_bytes.clone()))
            .unwrap();
        registry
            .register(Box::new(exporter_cpu_percent.clone()))
            .unwrap();

        Self {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_tx,
            service_cost,
            service_estimated_monthly,
            current_usage,
            estimated_monthly,
            daily_average,
            days_in_billing_period,
            days_remaining_in_month,
            last_scrape_timestamp,
            scrape_duration_seconds,
            api_up,
            exporter_memory_bytes,
            exporter_cpu_percent,
            registry,
        }
    }

    /// Encodes all metrics in Prometheus text format.
    pub fn encode(&self) -> String {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buffer)
            .unwrap();
        String::from_utf8(buffer).unwrap()
    }

    /// Updates exporter process metrics (CPU, memory).
    pub fn update_process_metrics(&self) {
        let mut sys = System::new();
        let pid = sysinfo::get_current_pid().unwrap();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);

        if let Some(process) = sys.process(pid) {
            self.exporter_memory_bytes
                .with_label_values(&[])
                .set(process.memory() as f64);
            self.exporter_cpu_percent
                .with_label_values(&[])
                .set(process.cpu_usage() as f64);
        }
    }

    /// Resets all metric values.
    pub fn reset(&self) {
        self.cpu_usage.reset();
        self.memory_usage.reset();
        self.disk_usage.reset();
        self.network_tx.reset();
        self.service_cost.reset();
        self.service_estimated_monthly.reset();
        self.current_usage.reset();
        self.estimated_monthly.reset();
        self.daily_average.reset();
        self.days_in_billing_period.reset();
        self.days_remaining_in_month.reset();
        self.last_scrape_timestamp.reset();
        self.scrape_duration_seconds.reset();
        self.api_up.reset();
        self.exporter_memory_bytes.reset();
        self.exporter_cpu_percent.reset();
    }
}
