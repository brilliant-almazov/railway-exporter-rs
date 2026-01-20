//! Unit tests for Railway Exporter metrics.

use crate::metrics::Metrics;

// =============================================================================
// Metrics Creation Tests
// =============================================================================

#[test]
fn test_metrics_default() {
    let metrics = Metrics::default();
    // Default should work the same as new()
    metrics
        .cpu_usage
        .with_label_values(&["test", "project", "📦", "default"])
        .set(0.0);
    let output = metrics.encode();
    assert!(!output.is_empty());
    assert!(output.contains("railway_cpu_usage_vcpu_minutes"));
}

#[test]
fn test_metrics_new() {
    let metrics = Metrics::new();
    // Set at least one metric value (registry only outputs metrics with values)
    metrics
        .cpu_usage
        .with_label_values(&["test", "project", "📦", "default"])
        .set(0.0);
    let output = metrics.encode();
    assert!(!output.is_empty());
    assert!(output.contains("railway_cpu_usage_vcpu_minutes"));
}

#[test]
fn test_metrics_with_service_labels() {
    let metrics = Metrics::new();
    metrics
        .cpu_usage
        .with_label_values(&["web", "myproject", "🌐", "frontend"])
        .set(1234.5);

    let output = metrics.encode();
    assert!(output.contains("railway_cpu_usage_vcpu_minutes{"));
    assert!(output.contains("service=\"web\""));
    assert!(output.contains("group=\"frontend\""));
}

#[test]
fn test_metrics_reset() {
    let metrics = Metrics::new();
    metrics
        .cpu_usage
        .with_label_values(&["api", "prod", "", "backend"])
        .set(1000.0);

    metrics.reset();

    let output = metrics.encode();
    assert!(!output.contains("1000"));
}

#[test]
fn test_process_metrics() {
    let metrics = Metrics::new();
    metrics.update_process_metrics();

    let output = metrics.encode();
    assert!(output.contains("railway_exporter_memory_bytes"));
    assert!(output.contains("railway_exporter_cpu_percent"));
}

// =============================================================================
// Per-Service Metrics Tests
// =============================================================================

#[test]
fn test_all_service_metrics() {
    let metrics = Metrics::new();
    let labels = &["api", "my-project", "🚀", "backend"];

    // Set all per-service metrics
    metrics.cpu_usage.with_label_values(labels).set(100.0);
    metrics.memory_usage.with_label_values(labels).set(200.0);
    metrics.disk_usage.with_label_values(labels).set(50.0);
    metrics.network_tx.with_label_values(labels).set(1.5);
    metrics.service_cost.with_label_values(labels).set(10.50);
    metrics
        .service_estimated_monthly
        .with_label_values(labels)
        .set(31.50);

    let output = metrics.encode();

    // Verify all metrics appear in output
    assert!(output.contains("railway_cpu_usage_vcpu_minutes"));
    assert!(output.contains("railway_memory_usage_gb_minutes"));
    assert!(output.contains("railway_disk_usage_gb_minutes"));
    assert!(output.contains("railway_network_tx_gb"));
    assert!(output.contains("railway_service_cost_usd"));
    assert!(output.contains("railway_service_estimated_monthly_usd"));

    // Verify values
    assert!(output.contains("100"));
    assert!(output.contains("200"));
    assert!(output.contains("10.5"));
}

// =============================================================================
// Per-Project Metrics Tests
// =============================================================================

#[test]
fn test_all_project_metrics() {
    let metrics = Metrics::new();
    let labels = &["my-project"];

    // Set all per-project metrics
    metrics.current_usage.with_label_values(labels).set(25.00);
    metrics
        .estimated_monthly
        .with_label_values(labels)
        .set(75.00);
    metrics.daily_average.with_label_values(labels).set(2.50);
    metrics
        .days_in_billing_period
        .with_label_values(labels)
        .set(10.0);
    metrics
        .days_remaining_in_month
        .with_label_values(labels)
        .set(20.0);
    metrics
        .last_scrape_timestamp
        .with_label_values(labels)
        .set(1700000000.0);
    metrics
        .scrape_duration_seconds
        .with_label_values(labels)
        .set(0.15);
    metrics.api_up.with_label_values(labels).set(1.0);

    let output = metrics.encode();

    // Verify all metrics appear in output
    assert!(output.contains("railway_current_usage_usd"));
    assert!(output.contains("railway_estimated_monthly_usd"));
    assert!(output.contains("railway_daily_average_usd"));
    assert!(output.contains("railway_days_in_billing_period"));
    assert!(output.contains("railway_days_remaining_in_month"));
    assert!(output.contains("railway_exporter_last_scrape_timestamp"));
    assert!(output.contains("railway_exporter_scrape_duration_seconds"));
    assert!(output.contains("railway_api_up"));
}

// =============================================================================
// Encode Tests
// =============================================================================

#[test]
fn test_encode_empty_registry() {
    let metrics = Metrics::new();
    let output = metrics.encode();
    // Empty registry produces empty output (no metrics set)
    // This is valid Prometheus format
    assert!(output.is_empty() || output.contains("#"));
}

#[test]
fn test_encode_prometheus_format() {
    let metrics = Metrics::new();
    metrics
        .cpu_usage
        .with_label_values(&["svc", "proj", "", "grp"])
        .set(42.0);

    let output = metrics.encode();

    // Verify Prometheus text format
    assert!(output.contains("# HELP railway_cpu_usage_vcpu_minutes"));
    assert!(output.contains("# TYPE railway_cpu_usage_vcpu_minutes gauge"));
    assert!(output.contains("railway_cpu_usage_vcpu_minutes{"));
    assert!(output.contains("} 42"));
}
