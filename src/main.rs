//! Railway Exporter - Prometheus metrics for Railway.app billing.
//!
//! Endpoints:
//! - GET /metrics - Prometheus format
//! - GET /metrics.json - JSON format
//! - GET /status - Server status
//! - GET /health - Health check
//! - GET /ws - WebSocket (real-time updates)

// === Memory Allocator Configuration ===
//
// Linux: jemalloc with aggressive memory release (dirty_decay_ms:0, muzzy_decay_ms:0)
// macOS: System allocator (malloc is already efficient on macOS)
// Windows: System allocator (MSVC default)

#[cfg(all(target_os = "linux", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

/// Configure jemalloc for aggressive memory release on Linux.
#[cfg(all(target_os = "linux", not(target_env = "msvc")))]
fn configure_allocator() {
    use tikv_jemalloc_ctl::{arenas, epoch, opt};

    // Advance epoch to get fresh stats
    epoch::advance().ok();

    // Log jemalloc version
    if let Ok(version) = opt::version::read() {
        tracing::info!("jemalloc version: {}", version);
    }

    // Set aggressive memory release: return memory to OS immediately
    // dirty_decay_ms: time before dirty pages are released (0 = immediate)
    // muzzy_decay_ms: time before muzzy pages are released (0 = immediate)
    if let Err(e) = arenas::dirty_decay_ms::write(0) {
        tracing::warn!("Failed to set dirty_decay_ms: {}", e);
    }
    if let Err(e) = arenas::muzzy_decay_ms::write(0) {
        tracing::warn!("Failed to set muzzy_decay_ms: {}", e);
    }

    tracing::info!("jemalloc configured with aggressive memory release");
}

/// No-op allocator configuration for non-Linux platforms.
#[cfg(not(all(target_os = "linux", not(target_env = "msvc"))))]
fn configure_allocator() {
    #[cfg(target_os = "macos")]
    tracing::info!("Using macOS system allocator");

    #[cfg(target_os = "windows")]
    tracing::info!("Using Windows system allocator");
}

use railway_exporter::client::Client;
use railway_exporter::{collector, server, AppState, Config};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Configure memory allocator for aggressive memory release
    configure_allocator();

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
    info!(
        "Gzip: enabled={}, min_size={}, level={}",
        config.gzip.enabled, config.gzip.min_size, config.gzip.level
    );
    info!(
        "Icon cache: enabled={}, max_count={}",
        config.icon_cache.enabled, config.icon_cache.max_count
    );

    // Create application state
    let state = Arc::new(AppState::new(config.clone()));

    // Create Railway API client
    let client = Client::new(&config.api_token, Some(&config.api_url));

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
        let client = Client::new(&api_token, Some(&api_url));

        loop {
            ticker.tick().await;
            let _ = collector::collect_metrics(&client, &state_bg).await;
        }
    });

    // Start HTTP server (blocks)
    server::start(state).await;
}
