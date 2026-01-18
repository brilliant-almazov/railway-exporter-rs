//! Application state management.

use crate::metrics::Metrics;
use crate::types::MetricsJson;
use crate::utils::ProcessInfoProvider;
use crate::Config;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;
use tokio::sync::{broadcast, RwLock};

/// API status tracking data.
#[derive(Debug, Default)]
pub struct ApiStatusData {
    /// Timestamp of last successful scrape.
    pub last_success: Option<i64>,
    /// Last error message (if any).
    pub last_error: Option<String>,
    /// Total number of scrapes attempted.
    pub total_scrapes: u64,
    /// Number of failed scrapes.
    pub failed_scrapes: u64,
}

/// Shared application state.
pub struct AppState {
    /// Application configuration.
    pub config: Config,
    /// Prometheus metrics.
    pub metrics: Metrics,
    /// Cached JSON metrics for /metrics.json endpoint.
    pub metrics_json: RwLock<Option<MetricsJson>>,
    /// Server start time.
    pub start_time: Instant,
    /// API status tracking.
    pub api_status: RwLock<ApiStatusData>,
    /// WebSocket broadcast channel.
    pub ws_broadcast: broadcast::Sender<String>,
    /// Process info provider (CPU, memory, PID).
    pub process_info: ProcessInfoProvider,
    /// Number of active WebSocket clients.
    pub ws_clients: AtomicU32,
}

impl AppState {
    /// Creates new application state.
    pub fn new(config: Config) -> Self {
        let (ws_tx, _) = broadcast::channel::<String>(16);

        Self {
            config,
            metrics: Metrics::new(),
            metrics_json: RwLock::new(None),
            start_time: Instant::now(),
            api_status: RwLock::new(ApiStatusData::default()),
            ws_broadcast: ws_tx,
            process_info: ProcessInfoProvider::new(),
            ws_clients: AtomicU32::new(0),
        }
    }

    /// Increment WebSocket client count.
    pub fn ws_client_connect(&self) -> u32 {
        self.ws_clients.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Decrement WebSocket client count.
    pub fn ws_client_disconnect(&self) -> u32 {
        self.ws_clients.fetch_sub(1, Ordering::SeqCst) - 1
    }

    /// Get current WebSocket client count.
    pub fn ws_client_count(&self) -> u32 {
        self.ws_clients.load(Ordering::SeqCst)
    }
}
