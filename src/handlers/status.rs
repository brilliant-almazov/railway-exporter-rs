//! Status endpoint handler.

use super::HandlerResponse;
use crate::state::AppState;
use crate::types::{ApiStatus, ConfigStatus, EndpointStatus, ServerStatus};
use hyper::body::Bytes;
use hyper::Response;

/// GET /status (or /) - Server status and configuration.
///
/// Returns JSON with:
/// - version, project_name, uptime
/// - endpoints (what's enabled from config)
/// - config (plan, scrape_interval, groups list)
/// - process (CPU, memory from ProcessInfoProvider in AppState)
/// - api (last success/error, scrape counts)
pub async fn handle(state: &AppState) -> HandlerResponse {
    let api_status = state.api_status.read().await;
    let process = state.process_info.status();

    // EndpointStatus from config (not hardcoded!)
    let endpoints = EndpointStatus {
        prometheus: true, // Always enabled
        json: true,       // Always enabled
        websocket: state.config.websocket_enabled,
        health: true, // Always enabled
    };

    // Get group names from config
    let service_groups: Vec<String> = state.config.service_groups.keys().cloned().collect();

    let status = ServerStatus {
        version: env!("CARGO_PKG_VERSION"),
        project_name: state.config.project_name.clone(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        endpoints,
        config: ConfigStatus {
            plan: state.config.plan.to_string(),
            scrape_interval_seconds: state.config.scrape_interval,
            api_url: state.config.api_url.clone(),
            service_groups,
            prices: state.config.pricing_values.clone(),
        },
        process,
        api: ApiStatus {
            last_success: api_status.last_success,
            last_error: api_status.last_error.clone(),
            total_scrapes: api_status.total_scrapes,
            failed_scrapes: api_status.failed_scrapes,
        },
    };

    (
        Response::builder().header("Content-Type", "application/json"),
        Bytes::from(serde_json::to_string_pretty(&status).unwrap()),
    )
}
