//! Metrics collection from Railway API.

use crate::client::{ApiError, Client};
use crate::config::IconMode;
use crate::state::AppState;
use crate::types::{MetricsJson, ProjectSummary, ServiceData, WsMessage};
use chrono::{Datelike, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

/// Collects metrics from Railway API and updates Prometheus gauges.
pub async fn collect_metrics(
    client: &Client,
    state: &Arc<AppState>,
) -> Result<(), ApiError> {
    let start = Instant::now();
    let config = &state.config;
    let metrics = &state.metrics;

    // Update process metrics
    metrics.update_process_metrics();

    // Update scrape counter
    {
        let mut status = state.api_status.write().await;
        status.total_scrapes += 1;
    }

    // Get project info
    let project = match client.get_project(&config.project_id).await {
        Ok(p) => {
            metrics
                .api_up
                .with_label_values(&[&config.project_id])
                .set(1.0);
            p
        }
        Err(e) => {
            metrics
                .api_up
                .with_label_values(&[&config.project_id])
                .set(0.0);
            let mut status = state.api_status.write().await;
            status.failed_scrapes += 1;
            status.last_error = Some(e.to_string());
            return Err(e);
        }
    };

    let project_name = &project.name;

    // Build service map: id -> (name, icon_url, group)
    // First pass: collect service info with original icon URLs
    let services_raw: Vec<(String, String, String, String)> = project
        .services
        .edges
        .iter()
        .map(|e| {
            let name = e.node.name.clone();
            let icon_url = e.node.icon.clone().unwrap_or_default();
            // Find group for this service
            let group = config
                .service_groups
                .iter()
                .find(|(_, patterns)| patterns.iter().any(|p| name.contains(p) || p == &name))
                .map(|(g, _)| g.clone())
                .unwrap_or_else(|| "ungrouped".to_string());
            (e.node.id.clone(), name, icon_url, group)
        })
        .collect();

    // Second pass: process icons based on mode
    debug!("Processing icons for {} services (mode: {})", services_raw.len(), config.icon_cache.mode);
    let mut services: HashMap<String, (String, String, String)> = HashMap::new();
    for (id, name, icon_url, group) in services_raw {
        let icon_data = if !config.icon_cache.enabled {
            // Caching disabled - use original URL as-is
            icon_url.clone()
        } else {
            match config.icon_cache.mode {
                IconMode::Base64 => {
                    // Fetch and return as base64 data URL
                    state.icon_cache.get_icon(&name, &icon_url).await
                }
                IconMode::Link => {
                    // Ensure icon is cached, return full URL to our endpoint
                    if !icon_url.is_empty() && !icon_url.starts_with("data:") {
                        state.icon_cache.ensure_cached(&name, &icon_url).await;
                        let path = format!("/static/icons/services/{}", urlencoding::encode(&name));
                        if config.icon_cache.base_url.is_empty() {
                            path
                        } else {
                            format!("{}{}", config.icon_cache.base_url, path)
                        }
                    } else if icon_url.starts_with("data:") {
                        // Already a data URL - use as-is
                        icon_url.clone()
                    } else {
                        // Empty URL
                        String::new()
                    }
                }
            }
        };
        services.insert(id, (name, icon_data, group));
    }

    // Get usage metrics
    let usage = client.get_usage(&config.project_id).await?;

    let mut total_cost = 0.0;
    let mut services_data: Vec<ServiceData> = Vec::new();

    for (sid, measurements) in &usage {
        let default_svc = (sid.clone(), String::new(), "ungrouped".to_string());
        let (name, icon, group) = services.get(sid).unwrap_or(&default_svc);

        let cpu = *measurements.get("CPU_USAGE").unwrap_or(&0.0);
        let mem = *measurements.get("MEMORY_USAGE_GB").unwrap_or(&0.0);
        let disk = *measurements.get("DISK_USAGE_GB").unwrap_or(&0.0);
        let tx = *measurements.get("NETWORK_TX_GB").unwrap_or(&0.0);

        let labels = &[
            name.as_str(),
            project_name.as_str(),
            icon.as_str(),
            group.as_str(),
        ];

        metrics.cpu_usage.with_label_values(labels).set(cpu);
        metrics.memory_usage.with_label_values(labels).set(mem);
        metrics.disk_usage.with_label_values(labels).set(disk);
        metrics.network_tx.with_label_values(labels).set(tx);

        let cost = cpu * config.pricing.get_price("CPU_USAGE")
            + mem * config.pricing.get_price("MEMORY_USAGE_GB")
            + disk * config.pricing.get_price("DISK_USAGE_GB")
            + tx * config.pricing.get_price("NETWORK_TX_GB");

        metrics.service_cost.with_label_values(labels).set(cost);
        total_cost += cost;

        // Check if service is deleted (exists in usage but not in services list)
        let is_deleted = !services.contains_key(sid);

        services_data.push(ServiceData {
            id: sid.clone(),
            name: name.clone(),
            icon: icon.clone(),
            group: group.clone(),
            cpu_usage: cpu,
            memory_usage: mem,
            disk_usage: disk,
            network_tx: tx,
            cost_usd: cost,
            estimated_monthly_usd: 0.0, // Updated below
            is_deleted,
        });
    }

    // Get estimated usage
    let estimated = client.get_estimated_usage(&config.project_id).await?;
    let est_monthly: f64 = estimated
        .iter()
        .map(|(measurement, value)| value * config.pricing.get_price(measurement))
        .sum();

    // Update estimated monthly per service (proportional to current cost)
    if total_cost > 0.0 {
        for service in &mut services_data {
            let ratio = service.cost_usd / total_cost;
            service.estimated_monthly_usd = est_monthly * ratio;

            let labels = &[
                service.name.as_str(),
                project_name,
                service.icon.as_str(),
                service.group.as_str(),
            ];
            metrics
                .service_estimated_monthly
                .with_label_values(labels)
                .set(service.estimated_monthly_usd);
        }
    }

    // Project-level metrics
    metrics
        .current_usage
        .with_label_values(&[project_name])
        .set(total_cost);
    metrics
        .estimated_monthly
        .with_label_values(&[project_name])
        .set(est_monthly);

    // Calculate billing period
    let now = Utc::now();
    let days_elapsed = now.day();
    let days_in_month = days_in_current_month(now.year(), now.month());
    let days_remaining = days_in_month - days_elapsed;

    metrics
        .daily_average
        .with_label_values(&[project_name])
        .set(total_cost / days_elapsed as f64);
    metrics
        .days_in_billing_period
        .with_label_values(&[project_name])
        .set(days_elapsed as f64);
    metrics
        .days_remaining_in_month
        .with_label_values(&[project_name])
        .set(days_remaining as f64);

    let scrape_duration = start.elapsed().as_secs_f64();
    let timestamp = now.timestamp();

    metrics
        .last_scrape_timestamp
        .with_label_values(&[project_name])
        .set(timestamp as f64);
    metrics
        .scrape_duration_seconds
        .with_label_values(&[project_name])
        .set(scrape_duration);

    // Build JSON response
    let metrics_json = MetricsJson {
        project: ProjectSummary {
            name: project_name.clone(),
            current_usage_usd: total_cost,
            estimated_monthly_usd: est_monthly,
            daily_average_usd: total_cost / days_elapsed as f64,
            days_elapsed,
            days_remaining,
        },
        services: services_data,
        scrape_timestamp: timestamp,
        scrape_duration_seconds: scrape_duration,
    };

    // Store for HTTP endpoint
    {
        let mut json_lock = state.metrics_json.write().await;
        *json_lock = Some(metrics_json.clone());
    }

    // Broadcast to WebSocket clients (as WsMessage::Metrics)
    let ws_msg = WsMessage::Metrics(metrics_json.clone());
    if let Ok(json_str) = serde_json::to_string(&ws_msg) {
        let _ = state.ws_broadcast.send(json_str);
    }

    // Update API status
    {
        let mut status = state.api_status.write().await;
        status.last_success = Some(timestamp);
        status.last_error = None;
    }

    info!(
        "Collected in {:.2}s. Current: ${:.2}, Estimated: ${:.2}/month",
        scrape_duration, total_cost, est_monthly
    );

    Ok(())
}

/// Calculates days in a given month.
pub(crate) fn days_in_current_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}
