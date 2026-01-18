//! Shared types for Railway Exporter.

use crate::config::PriceValues;
use serde::{Deserialize, Serialize};
// ============================================================================
// JSON Response Types
// ============================================================================

/// Service data for JSON output.
#[derive(Clone, Serialize, Debug)]
pub struct ServiceData {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub group: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_tx: f64,
    pub network_rx: f64,
    pub cost_usd: f64,
    pub estimated_monthly_usd: f64,
    #[serde(rename = "isDeleted")]
    pub is_deleted: bool,
}

/// Project summary for JSON output.
#[derive(Clone, Serialize, Debug)]
pub struct ProjectSummary {
    pub name: String,
    pub current_usage_usd: f64,
    pub estimated_monthly_usd: f64,
    pub daily_average_usd: f64,
    pub days_elapsed: u32,
    pub days_remaining: u32,
}

/// Full metrics JSON response.
#[derive(Clone, Serialize, Debug)]
pub struct MetricsJson {
    pub project: ProjectSummary,
    pub services: Vec<ServiceData>,
    pub scrape_timestamp: i64,
    pub scrape_duration_seconds: f64,
}

// ============================================================================
// WebSocket Message Types
// ============================================================================

/// WebSocket message wrapper with type discrimination.
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    #[serde(rename = "metrics")]
    Metrics(MetricsJson),
    #[serde(rename = "status")]
    Status(WsStatus),
}

/// Lightweight status for WebSocket (subset of ServerStatus).
#[derive(Serialize, Debug, Clone)]
pub struct WsStatus {
    pub uptime_seconds: u64,
    pub api: ApiStatus,
    pub ws_clients: u32,
}

// ============================================================================
// Status Response Types
// ============================================================================

/// Server status response.
#[derive(Serialize, Debug)]
pub struct ServerStatus {
    pub version: &'static str,
    pub project_name: String,
    pub uptime_seconds: u64,
    pub endpoints: EndpointStatus,
    pub config: ConfigStatus,
    pub process: ProcessStatus,
    pub api: ApiStatus,
}

/// Endpoint availability status (from config).
#[derive(Serialize, Debug, Clone)]
pub struct EndpointStatus {
    pub prometheus: bool,
    pub json: bool,
    pub websocket: bool,
    pub health: bool,
}

/// Config status exposed to frontend.
#[derive(Serialize, Debug)]
pub struct ConfigStatus {
    pub plan: String,
    pub scrape_interval_seconds: u16,
    pub api_url: String,
    /// List of group names (for frontend dropdown).
    pub service_groups: Vec<String>,
    pub prices: PriceValues,
}

#[derive(Serialize, Debug)]
pub struct ProcessStatus {
    pub pid: u32,
    pub memory_mb: f64,
    pub cpu_percent: f32,
}

#[derive(Serialize, Debug, Clone)]
pub struct ApiStatus {
    pub last_success: Option<i64>,
    pub last_error: Option<String>,
    pub total_scrapes: u64,
    pub failed_scrapes: u64,
}

// ============================================================================
// GraphQL Types (for railway.rs)
// ============================================================================

/// GraphQL request body.
#[derive(Debug, Serialize)]
pub struct GraphQLRequest {
    pub query: String,
}

/// GraphQL response wrapper.
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ProjectData {
    pub project: Project,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub services: ServiceEdges,
}

#[derive(Debug, Deserialize)]
pub struct ServiceEdges {
    pub edges: Vec<ServiceEdge>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceEdge {
    pub node: ServiceNode,
}

#[derive(Debug, Deserialize)]
pub struct ServiceNode {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UsageData {
    pub usage: Vec<UsageItem>,
}

#[derive(Debug, Deserialize)]
pub struct UsageItem {
    pub measurement: String,
    pub value: f64,
    pub tags: UsageTags,
}

#[derive(Debug, Deserialize)]
pub struct UsageTags {
    #[serde(rename = "serviceId")]
    pub service_id: String,
}

#[derive(Debug, Deserialize)]
pub struct EstimatedData {
    #[serde(rename = "estimatedUsage")]
    pub estimated_usage: Vec<EstimatedItem>,
}

#[derive(Debug, Deserialize)]
pub struct EstimatedItem {
    pub measurement: String,
    #[serde(rename = "estimatedValue")]
    pub estimated_value: f64,
}
