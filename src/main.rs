//! Railway Exporter - Prometheus metrics for Railway.app billing.
//!
//! Endpoints:
//! - GET /metrics - Prometheus format
//! - GET /metrics.json - JSON format
//! - GET /status - Server status
//! - GET /health - Health check
//! - GET /ws - WebSocket (real-time updates)

use railway_exporter::railway::RailwayClient;
use railway_exporter::{collector, server, AppState, Config};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    info!("Railway Exporter v{}", env!("CARGO_PKG_VERSION"));
    info!("Plan: {}", config.plan);
    info!("API URL: {}", config.api_url);
    info!("Scrape interval: {}s", config.scrape_interval);
    info!(
        "Service groups: {:?}",
        config.service_groups.keys().collect::<Vec<_>>()
    );

    // Create application state
    let state = Arc::new(AppState::new(config.clone()));

    // Create Railway API client
    let client = RailwayClient::new(&config.api_token, Some(&config.api_url));

    // Initial collection
    if let Err(e) = collector::collect_metrics(&client, &state).await {
        error!("Initial collection failed: {}", e);
    }

    // Background collection loop
    let state_bg = state.clone();
    let scrape_interval = config.scrape_interval;
    let api_token = config.api_token.clone();
    let api_url = config.api_url.clone();

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(scrape_interval as u64));
        let client = RailwayClient::new(&api_token, Some(&api_url));

        loop {
            ticker.tick().await;
            let _ = collector::collect_metrics(&client, &state_bg).await;
        }
    });

    // Start HTTP server (blocks)
    server::start(state).await;
}
