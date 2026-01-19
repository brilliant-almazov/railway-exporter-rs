//! Railway GraphQL API client.
//!
//! This module provides functionality to query the Railway API for:
//! - Project information and services
//! - Usage metrics (CPU, memory, disk, network)
//! - Estimated monthly usage
//!
//! ## API Documentation
//!
//! - [Railway GraphQL API Reference](https://docs.railway.app/reference/graphql-api)
//! - [GraphQL Playground](https://railway.app/graphql) - Interactive API explorer (requires auth)
//! - [Railway API Token](https://docs.railway.app/reference/public-api#creating-a-token) - How to create tokens
//!
//! ## Current Metrics
//!
//! | Metric | GraphQL Field | Description |
//! |--------|---------------|-------------|
//! | CPU | `CPU_USAGE` | vCPU-minutes consumed |
//! | Memory | `MEMORY_USAGE_GB` | GB-minutes consumed |
//! | Disk | `DISK_USAGE_GB` | GB-minutes of persistent storage |
//! | Network TX | `NETWORK_TX_GB` | Egress traffic in GB |
//!
//! ## Potential Future Metrics
//!
//! These could be added in future versions:
//!
//! - **Replica count** - Number of running instances per service
//! - **Deployment status** - Current deployment state (building/deploying/running/failed)
//! - **Region** - Geographic region where service runs (us-east, us-west, eu-west)
//! - **Build time** - Time spent building container images
//! - **Restart count** - Number of service restarts (health indicator)
//! - **Environment count** - Number of environments in project
//! - **Volume usage** - Detailed persistent volume metrics
//!
//! ## Example GraphQL Queries
//!
//! Get project with services:
//! ```graphql
//! {
//!   project(id: "PROJECT_ID") {
//!     name
//!     services { edges { node { id name } } }
//!   }
//! }
//! ```
//!
//! Get usage metrics:
//! ```graphql
//! {
//!   usage(
//!     projectId: "PROJECT_ID",
//!     measurements: [CPU_USAGE, MEMORY_USAGE_GB, DISK_USAGE_GB, NETWORK_TX_GB],
//!     groupBy: [SERVICE_ID]
//!   ) {
//!     measurement
//!     value
//!     tags { serviceId }
//!   }
//! }
//! ```

use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::DEFAULT_API_URL;

/// GraphQL request body.
#[derive(Debug, Serialize)]
pub struct GraphQLRequest {
    /// The GraphQL query string.
    pub query: String,
}

/// GraphQL response wrapper.
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    /// Response data (if successful).
    pub data: Option<T>,
    /// Errors (if any).
    pub errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error.
#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    /// Error message.
    pub message: String,
}

/// Project data response.
#[derive(Debug, Deserialize)]
pub struct ProjectData {
    /// The project.
    pub project: Project,
}

/// Railway project.
#[derive(Debug, Deserialize)]
pub struct Project {
    /// Project name.
    pub name: String,
    /// Project services.
    pub services: ServiceEdges,
}

/// Service edges wrapper.
#[derive(Debug, Deserialize)]
pub struct ServiceEdges {
    /// List of service edges.
    pub edges: Vec<ServiceEdge>,
}

/// Service edge.
#[derive(Debug, Deserialize)]
pub struct ServiceEdge {
    /// The service node.
    pub node: ServiceNode,
}

/// Service node.
#[derive(Debug, Deserialize)]
pub struct ServiceNode {
    /// Service ID.
    pub id: String,
    /// Service name.
    pub name: String,
    /// Service icon (emoji or URL).
    pub icon: Option<String>,
}

/// Usage data response.
#[derive(Debug, Deserialize)]
pub struct UsageData {
    /// List of usage items.
    pub usage: Vec<UsageItem>,
}

/// Usage measurement item.
#[derive(Debug, Deserialize)]
pub struct UsageItem {
    /// Measurement type (CPU_USAGE, MEMORY_USAGE_GB, etc.).
    pub measurement: String,
    /// Usage value.
    pub value: f64,
    /// Tags including service ID.
    pub tags: UsageTags,
}

/// Usage tags.
#[derive(Debug, Deserialize)]
pub struct UsageTags {
    /// Service ID.
    #[serde(rename = "serviceId")]
    pub service_id: String,
}

/// Estimated usage data response.
#[derive(Debug, Deserialize)]
pub struct EstimatedData {
    /// List of estimated usage items.
    #[serde(rename = "estimatedUsage")]
    pub estimated_usage: Vec<EstimatedItem>,
}

/// Estimated usage item.
#[derive(Debug, Deserialize)]
pub struct EstimatedItem {
    /// Measurement type.
    pub measurement: String,
    /// Estimated value for the month.
    #[serde(rename = "estimatedValue")]
    pub estimated_value: f64,
}

/// Railway API client error.
#[derive(Debug)]
pub enum ApiError {
    /// HTTP request failed.
    RequestError(String),
    /// GraphQL returned an error.
    GraphQLError(String),
    /// Response parsing failed.
    ParseError(String),
    /// No data in response.
    NoData,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::RequestError(msg) => write!(f, "Request error: {}", msg),
            ApiError::GraphQLError(msg) => write!(f, "GraphQL error: {}", msg),
            ApiError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ApiError::NoData => write!(f, "No data in response"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Railway API client.
///
/// # Example
///
/// ```rust,no_run
/// use railway_exporter::client::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new("your-api-token", None);
///
///     let project = client.get_project("project-id").await.unwrap();
///     println!("Project: {}", project.name);
/// }
/// ```
pub struct Client {
    http: HttpClient,
    token: String,
    api_url: String,
}

impl Client {
    /// Creates a new Railway API client.
    ///
    /// # Arguments
    ///
    /// * `token` - Railway API token
    /// * `api_url` - Optional API URL (defaults to Railway's GraphQL endpoint)
    pub fn new(token: &str, api_url: Option<&str>) -> Self {
        Self {
            http: HttpClient::new(),
            token: token.to_string(),
            api_url: api_url.unwrap_or(DEFAULT_API_URL).to_string(),
        }
    }

    /// Executes a GraphQL query.
    ///
    /// # Arguments
    ///
    /// * `query` - GraphQL query string
    ///
    /// # Returns
    ///
    /// Parsed response data or an error.
    pub async fn query<T: for<'de> Deserialize<'de>>(&self, query: &str) -> Result<T, ApiError> {
        let resp = self
            .http
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&GraphQLRequest {
                query: query.to_string(),
            })
            .send()
            .await
            .map_err(|e| ApiError::RequestError(e.to_string()))?;

        let gql_resp: GraphQLResponse<T> = resp
            .json()
            .await
            .map_err(|e| ApiError::ParseError(e.to_string()))?;

        if let Some(errors) = gql_resp.errors {
            if !errors.is_empty() {
                return Err(ApiError::GraphQLError(errors[0].message.clone()));
            }
        }

        gql_resp.data.ok_or(ApiError::NoData)
    }

    /// Gets project information including services.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Railway project ID
    ///
    /// # Returns
    ///
    /// Project data with services list.
    pub async fn get_project(&self, project_id: &str) -> Result<Project, ApiError> {
        let query = format!(
            r#"{{ project(id: "{}") {{ name services {{ edges {{ node {{ id name icon }} }} }} }} }}"#,
            project_id
        );
        let data: ProjectData = self.query(&query).await?;
        Ok(data.project)
    }

    /// Gets current usage metrics for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Railway project ID
    ///
    /// # Returns
    ///
    /// Map of service ID to measurements (measurement name -> value).
    pub async fn get_usage(
        &self,
        project_id: &str,
    ) -> Result<HashMap<String, HashMap<String, f64>>, ApiError> {
        let query = format!(
            r#"{{ usage(projectId: "{}", measurements: [CPU_USAGE, MEMORY_USAGE_GB, DISK_USAGE_GB, NETWORK_TX_GB], groupBy: [SERVICE_ID]) {{ measurement value tags {{ serviceId }} }} }}"#,
            project_id
        );
        let data: UsageData = self.query(&query).await?;

        let mut result: HashMap<String, HashMap<String, f64>> = HashMap::new();
        for item in data.usage {
            result
                .entry(item.tags.service_id)
                .or_default()
                .insert(item.measurement, item.value);
        }
        Ok(result)
    }

    /// Gets estimated monthly usage for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Railway project ID
    ///
    /// # Returns
    ///
    /// Map of measurement name to estimated monthly value.
    pub async fn get_estimated_usage(
        &self,
        project_id: &str,
    ) -> Result<HashMap<String, f64>, ApiError> {
        let query = format!(
            r#"{{ estimatedUsage(projectId: "{}", measurements: [CPU_USAGE, MEMORY_USAGE_GB, DISK_USAGE_GB, NETWORK_TX_GB]) {{ measurement estimatedValue }} }}"#,
            project_id
        );
        let data: EstimatedData = self.query(&query).await?;

        let result: HashMap<String, f64> = data
            .estimated_usage
            .into_iter()
            .map(|i| (i.measurement, i.estimated_value))
            .collect();
        Ok(result)
    }
}
