//! Prometheus metrics for Railway usage data.
//!
//! This module provides the metrics registry and helper functions
//! for exposing Railway usage data in Prometheus format.
//!
//! ## Metrics Exposed
//!
//! ### Per-Service Metrics
//!
//! | Metric | Labels | Description |
//! |--------|--------|-------------|
//! | `railway_cpu_usage_vcpu_minutes` | service, project | CPU usage in vCPU-minutes |
//! | `railway_memory_usage_gb_minutes` | service, project | Memory usage in GB-minutes |
//! | `railway_disk_usage_gb_minutes` | service, project | Disk usage in GB-minutes |
//! | `railway_network_tx_gb` | service, project | Network egress in GB |
//! | `railway_network_rx_gb` | service, project | Network ingress in GB |
//! | `railway_service_cost_usd` | service, project | Current cost in USD |
//! | `railway_service_estimated_monthly_usd` | service, project | Estimated monthly cost |
//!
//! ### Project-Level Metrics
//!
//! | Metric | Labels | Description |
//! |--------|--------|-------------|
//! | `railway_current_usage_usd` | project | Total current usage |
//! | `railway_estimated_monthly_usd` | project | Estimated monthly total |
//! | `railway_daily_average_usd` | project | Average daily spending |
//!
//! ### Exporter Metrics
//!
//! | Metric | Labels | Description |
//! |--------|--------|-------------|
//! | `railway_exporter_scrape_duration_seconds` | — | Time spent scraping API |
//! | `railway_exporter_scrape_success` | — | Whether last scrape succeeded |

use prometheus::{Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};

/// Prometheus metrics registry for Railway data.
///
/// # Example
///
/// ```rust
/// use railway_exporter::Metrics;
///
/// let metrics = Metrics::new();
///
/// // Update a metric
/// metrics.cpu_usage.with_label_values(&["my-service", "my-project"]).set(1234.5);
///
/// // Get encoded metrics for Prometheus
/// let output = metrics.encode();
/// assert!(output.contains("railway_cpu_usage_vcpu_minutes"));
/// ```
pub struct Metrics {
    /// CPU usage in vCPU-minutes per service.
    pub cpu_usage: GaugeVec,

    /// Memory usage in GB-minutes per service.
    pub memory_usage: GaugeVec,

    /// Disk usage in GB-minutes per service.
    pub disk_usage: GaugeVec,

    /// Network egress in GB per service.
    pub network_tx: GaugeVec,

    /// Network ingress in GB per service.
    pub network_rx: GaugeVec,

    /// Current cost in USD per service.
    pub service_cost: GaugeVec,

    /// Estimated monthly cost in USD per service.
    pub service_estimated_monthly: GaugeVec,

    /// Total current usage in USD per project.
    pub current_usage: GaugeVec,

    /// Estimated monthly total in USD per project.
    pub estimated_monthly: GaugeVec,

    /// Average daily spending in USD per project.
    pub daily_average: GaugeVec,

    /// Time spent scraping the Railway API.
    pub scrape_duration: Gauge,

    /// Whether the last scrape was successful (1.0 = success, 0.0 = failure).
    pub scrape_success: Gauge,

    /// The Prometheus registry holding all metrics.
    registry: Registry,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Creates a new metrics registry with all Railway metrics registered.
    ///
    /// # Example
    ///
    /// ```rust
    /// use railway_exporter::Metrics;
    ///
    /// let metrics = Metrics::new();
    /// let output = metrics.encode();
    /// assert!(output.contains("railway_"));
    /// ```
    pub fn new() -> Self {
        let registry = Registry::new();

        // Per-service metrics
        let cpu_usage = GaugeVec::new(
            Opts::new(
                "railway_cpu_usage_vcpu_minutes",
                "CPU usage in vCPU-minutes",
            ),
            &["service", "project"],
        )
        .expect("Failed to create cpu_usage metric");

        let memory_usage = GaugeVec::new(
            Opts::new(
                "railway_memory_usage_gb_minutes",
                "Memory usage in GB-minutes",
            ),
            &["service", "project"],
        )
        .expect("Failed to create memory_usage metric");

        let disk_usage = GaugeVec::new(
            Opts::new("railway_disk_usage_gb_minutes", "Disk usage in GB-minutes"),
            &["service", "project"],
        )
        .expect("Failed to create disk_usage metric");

        let network_tx = GaugeVec::new(
            Opts::new("railway_network_tx_gb", "Network egress in GB"),
            &["service", "project"],
        )
        .expect("Failed to create network_tx metric");

        let network_rx = GaugeVec::new(
            Opts::new("railway_network_rx_gb", "Network ingress in GB"),
            &["service", "project"],
        )
        .expect("Failed to create network_rx metric");

        let service_cost = GaugeVec::new(
            Opts::new("railway_service_cost_usd", "Current service cost in USD"),
            &["service", "project"],
        )
        .expect("Failed to create service_cost metric");

        let service_estimated_monthly = GaugeVec::new(
            Opts::new(
                "railway_service_estimated_monthly_usd",
                "Estimated monthly service cost in USD",
            ),
            &["service", "project"],
        )
        .expect("Failed to create service_estimated_monthly metric");

        // Project-level metrics
        let current_usage = GaugeVec::new(
            Opts::new("railway_current_usage_usd", "Total current usage in USD"),
            &["project"],
        )
        .expect("Failed to create current_usage metric");

        let estimated_monthly = GaugeVec::new(
            Opts::new(
                "railway_estimated_monthly_usd",
                "Estimated monthly total in USD",
            ),
            &["project"],
        )
        .expect("Failed to create estimated_monthly metric");

        let daily_average = GaugeVec::new(
            Opts::new("railway_daily_average_usd", "Average daily spending in USD"),
            &["project"],
        )
        .expect("Failed to create daily_average metric");

        // Exporter metrics
        let scrape_duration = Gauge::new(
            "railway_exporter_scrape_duration_seconds",
            "Time spent scraping the Railway API",
        )
        .expect("Failed to create scrape_duration metric");

        let scrape_success = Gauge::new(
            "railway_exporter_scrape_success",
            "Whether the last scrape was successful (1 = success, 0 = failure)",
        )
        .expect("Failed to create scrape_success metric");

        // Register all metrics
        registry
            .register(Box::new(cpu_usage.clone()))
            .expect("Failed to register cpu_usage");
        registry
            .register(Box::new(memory_usage.clone()))
            .expect("Failed to register memory_usage");
        registry
            .register(Box::new(disk_usage.clone()))
            .expect("Failed to register disk_usage");
        registry
            .register(Box::new(network_tx.clone()))
            .expect("Failed to register network_tx");
        registry
            .register(Box::new(network_rx.clone()))
            .expect("Failed to register network_rx");
        registry
            .register(Box::new(service_cost.clone()))
            .expect("Failed to register service_cost");
        registry
            .register(Box::new(service_estimated_monthly.clone()))
            .expect("Failed to register service_estimated_monthly");
        registry
            .register(Box::new(current_usage.clone()))
            .expect("Failed to register current_usage");
        registry
            .register(Box::new(estimated_monthly.clone()))
            .expect("Failed to register estimated_monthly");
        registry
            .register(Box::new(daily_average.clone()))
            .expect("Failed to register daily_average");
        registry
            .register(Box::new(scrape_duration.clone()))
            .expect("Failed to register scrape_duration");
        registry
            .register(Box::new(scrape_success.clone()))
            .expect("Failed to register scrape_success");

        Self {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_tx,
            network_rx,
            service_cost,
            service_estimated_monthly,
            current_usage,
            estimated_monthly,
            daily_average,
            scrape_duration,
            scrape_success,
            registry,
        }
    }

    /// Encodes all metrics in Prometheus text format.
    ///
    /// # Returns
    ///
    /// A string containing all metrics in Prometheus exposition format.
    ///
    /// # Example
    ///
    /// ```rust
    /// use railway_exporter::Metrics;
    ///
    /// let metrics = Metrics::new();
    /// metrics.cpu_usage.with_label_values(&["api", "prod"]).set(1000.0);
    ///
    /// let output = metrics.encode();
    /// assert!(output.contains("railway_cpu_usage_vcpu_minutes"));
    /// assert!(output.contains("1000"));
    /// ```
    pub fn encode(&self) -> String {
        let encoder = TextEncoder::new();
        let mut buffer = Vec::new();
        encoder
            .encode(&self.registry.gather(), &mut buffer)
            .expect("Failed to encode metrics");
        String::from_utf8(buffer).expect("Metrics are not valid UTF-8")
    }

    /// Clears all metric values (useful for testing or reset scenarios).
    ///
    /// Note: This resets all gauge values to 0, but the metrics remain registered.
    pub fn reset(&self) {
        self.cpu_usage.reset();
        self.memory_usage.reset();
        self.disk_usage.reset();
        self.network_tx.reset();
        self.network_rx.reset();
        self.service_cost.reset();
        self.service_estimated_monthly.reset();
        self.current_usage.reset();
        self.estimated_monthly.reset();
        self.daily_average.reset();
        self.scrape_duration.set(0.0);
        self.scrape_success.set(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new();
        // Should be able to encode without panicking
        let output = metrics.encode();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_metrics_default() {
        let metrics = Metrics::default();
        let output = metrics.encode();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_metrics_encode_contains_exporter_metrics() {
        let metrics = Metrics::new();
        // Set values so metrics appear in output
        metrics.scrape_duration.set(0.0);
        metrics.scrape_success.set(0.0);
        let output = metrics.encode();

        // Exporter metrics should always be present (they're simple Gauges, not GaugeVec)
        assert!(output.contains("railway_exporter_scrape_duration_seconds"));
        assert!(output.contains("railway_exporter_scrape_success"));
    }

    #[test]
    fn test_metrics_set_and_encode() {
        let metrics = Metrics::new();

        metrics
            .cpu_usage
            .with_label_values(&["web", "myproject"])
            .set(1234.5);
        metrics
            .memory_usage
            .with_label_values(&["web", "myproject"])
            .set(567.8);
        metrics
            .current_usage
            .with_label_values(&["myproject"])
            .set(10.50);

        let output = metrics.encode();

        // Check metrics are present with correct labels and values
        assert!(output.contains("railway_cpu_usage_vcpu_minutes{"));
        assert!(output.contains("service=\"web\""));
        assert!(output.contains("project=\"myproject\""));
        assert!(output.contains("1234.5"));
    }

    #[test]
    fn test_metrics_multiple_services() {
        let metrics = Metrics::new();

        metrics
            .cpu_usage
            .with_label_values(&["api", "prod"])
            .set(100.0);
        metrics
            .cpu_usage
            .with_label_values(&["web", "prod"])
            .set(200.0);
        metrics
            .cpu_usage
            .with_label_values(&["db", "prod"])
            .set(300.0);

        let output = metrics.encode();

        assert!(output.contains("service=\"api\""));
        assert!(output.contains("service=\"web\""));
        assert!(output.contains("service=\"db\""));
    }

    #[test]
    fn test_metrics_reset() {
        let metrics = Metrics::new();

        metrics
            .cpu_usage
            .with_label_values(&["api", "prod"])
            .set(1000.0);
        metrics.scrape_duration.set(1.5);
        metrics.scrape_success.set(1.0);

        metrics.reset();

        // After reset, gauges should not contain the old values
        let output = metrics.encode();
        assert!(!output.contains("1000"));
        assert!(!output.contains("1.5"));
    }

    #[test]
    fn test_metrics_scrape_duration() {
        let metrics = Metrics::new();

        metrics.scrape_duration.set(0.5);
        let output = metrics.encode();

        assert!(output.contains("railway_exporter_scrape_duration_seconds 0.5"));
    }

    #[test]
    fn test_metrics_scrape_success() {
        let metrics = Metrics::new();

        metrics.scrape_success.set(1.0);
        let output = metrics.encode();
        assert!(output.contains("railway_exporter_scrape_success 1"));

        metrics.scrape_success.set(0.0);
        let output = metrics.encode();
        assert!(output.contains("railway_exporter_scrape_success 0"));
    }

    #[test]
    fn test_metrics_help_text() {
        let metrics = Metrics::new();
        // Set a value so the metric appears
        metrics
            .cpu_usage
            .with_label_values(&["test", "test"])
            .set(1.0);
        let output = metrics.encode();

        // Check HELP comments are present
        assert!(output.contains("# HELP railway_cpu_usage_vcpu_minutes"));
    }

    #[test]
    fn test_metrics_type_gauge() {
        let metrics = Metrics::new();
        // Set a value so the metric appears
        metrics
            .cpu_usage
            .with_label_values(&["test", "test"])
            .set(1.0);
        let output = metrics.encode();

        // Check TYPE comments indicate gauge
        assert!(output.contains("# TYPE railway_cpu_usage_vcpu_minutes gauge"));
    }
}
