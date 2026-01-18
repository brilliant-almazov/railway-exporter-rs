//! Configuration management for Railway Exporter.
//!
//! Configuration is loaded from YAML:
//! 1. Base64-encoded YAML in `CONFIG_BASE64` env var (for Docker/Railway)
//! 2. YAML config file (`config.yaml` or path in `CONFIG_FILE`)
//!
//! ## Environment Variables
//!
//! | Variable | Required | Default | Description |
//! |----------|:--------:|---------|-------------|
//! | `CONFIG_BASE64` | No | — | Base64-encoded YAML config |
//! | `CONFIG_FILE` | No | `config.yaml` | Path to YAML config file |
//!
//! ## YAML Config Format
//!
//! ```yaml
//! railway_api_token: "your-token"
//! railway_project_id: "your-project-id"
//! railway_plan: pro
//! port: 9090
//! scrape_interval: 300
//!
//! pricing:
//!   cpu: 0.000231
//!   memory: 0.000116
//!   disk: 0.000021
//!   network: 0.10
//!
//! service_groups:
//!   monitoring:
//!     - prometheus
//!     - grafana
//!   database:
//!     - postgres
//!     - redis
//! ```

use crate::pricing::PricingConfig;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Railway pricing plan.
///
/// Determines the pricing rates for resource usage.
///
/// # Example
///
/// ```rust
/// use railway_exporter::config::Plan;
/// use std::str::FromStr;
///
/// let plan = Plan::from_str("pro").unwrap();
/// assert_eq!(plan, Plan::Pro);
///
/// let plan: Plan = "hobby".parse().unwrap();
/// assert_eq!(plan, Plan::Hobby);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Plan {
    /// Hobby plan with higher per-unit prices.
    #[default]
    Hobby,
    /// Pro plan with lower per-unit prices (better for production).
    Pro,
}

impl Plan {
    /// Returns the plan name as a lowercase string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Plan::Hobby => "hobby",
            Plan::Pro => "pro",
        }
    }
}

impl fmt::Display for Plan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Plan {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hobby" => Ok(Plan::Hobby),
            "pro" => Ok(Plan::Pro),
            _ => Err(ConfigError::InvalidPlan(s.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for Plan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Plan::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// YAML configuration structure.
#[derive(Debug, Deserialize, Default)]
struct YamlConfig {
    railway_api_token: Option<String>,
    railway_project_id: Option<String>,
    railway_plan: Option<Plan>,
    railway_api_url: Option<String>,
    port: Option<u16>,
    scrape_interval: Option<u16>,
    pricing: Option<PricingSection>,
    service_groups: Option<HashMap<String, Vec<String>>>,
    /// Project display name (for /status endpoint).
    project_name: Option<String>,
    /// Enable CORS headers on all responses.
    cors_enabled: Option<bool>,
    /// Enable WebSocket endpoint.
    websocket_enabled: Option<bool>,
}

use serde::Serialize;

/// Network pricing (tx/rx).
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct NetworkPricing {
    pub tx: Option<f64>,
    pub rx: Option<f64>,
}

/// Price values for a plan.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct PriceValues {
    pub cpu: Option<f64>,
    pub memory: Option<f64>,
    pub disk: Option<f64>,
    pub network: Option<NetworkPricing>,
}

/// Named pricing entry (for API response).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PricingEntry {
    pub name: String,
    pub price: PriceValues,
}

/// Pricing section as array of named entries.
type PricingSection = Vec<PricingEntry>;

/// Default Railway GraphQL API URL.
pub const DEFAULT_API_URL: &str = "https://backboard.railway.app/graphql/v2";

/// Configuration for the Railway Exporter.
///
/// # Example
///
/// ```rust,no_run
/// use railway_exporter::Config;
///
/// // Load from config.yaml or CONFIG_BASE64
/// let config = Config::load().expect("Missing required config");
///
/// println!("Monitoring project: {}", config.project_id);
/// println!("Using {} plan pricing", config.plan);
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    /// Railway API token for authentication.
    pub api_token: String,

    /// Railway project ID to monitor.
    pub project_id: String,

    /// Pricing plan (Hobby or Pro).
    pub plan: Plan,

    /// Interval between API queries in seconds.
    pub scrape_interval: u16,

    /// HTTP server port for metrics endpoint.
    pub port: u16,

    /// Railway GraphQL API URL.
    pub api_url: String,

    /// Custom pricing configuration (for calculations).
    pub pricing: PricingConfig,

    /// Pricing values for current plan (for API response).
    pub pricing_values: PriceValues,

    /// Service groups mapping (group name -> list of service patterns).
    pub service_groups: HashMap<String, Vec<String>>,

    /// Project display name (for /status endpoint and frontend).
    pub project_name: String,

    /// Enable CORS headers on all responses.
    pub cors_enabled: bool,

    /// Enable WebSocket endpoint.
    pub websocket_enabled: bool,
}

/// Error type for configuration loading.
#[derive(Debug)]
pub enum ConfigError {
    /// Required configuration value is missing.
    MissingValue(String),
    /// Failed to parse configuration value.
    ParseError(String, String),
    /// Failed to read config file.
    FileError(String),
    /// Failed to parse YAML.
    YamlError(String),
    /// Failed to decode Base64.
    Base64Error(String),
    /// Invalid plan value (must be "hobby" or "pro").
    InvalidPlan(String),
    /// Invalid configuration value (out of range, wrong format, etc.).
    InvalidValue(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingValue(key) => write!(f, "Missing required config: {}", key),
            ConfigError::ParseError(key, msg) => write!(f, "Failed to parse {}: {}", key, msg),
            ConfigError::FileError(msg) => write!(f, "Config file error: {}", msg),
            ConfigError::YamlError(msg) => write!(f, "YAML parse error: {}", msg),
            ConfigError::Base64Error(msg) => write!(f, "Base64 decode error: {}", msg),
            ConfigError::InvalidPlan(value) => {
                write!(f, "Invalid plan '{}': must be 'hobby' or 'pro'", value)
            }
            ConfigError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// Loads configuration from YAML sources.
    ///
    /// Priority (highest to lowest):
    /// 1. Base64-encoded YAML in `CONFIG_BASE64` env var
    /// 2. YAML file specified in `CONFIG_FILE` env var
    /// 3. Default `config.yaml` file
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use railway_exporter::Config;
    ///
    /// let config = Config::load().expect("Failed to load config");
    /// ```
    pub fn load() -> Result<Self, ConfigError> {
        let yaml_config = Self::load_yaml_config()?;

        let api_token = yaml_config
            .railway_api_token
            .ok_or_else(|| ConfigError::MissingValue("railway_api_token".to_string()))?;

        let project_id = yaml_config
            .railway_project_id
            .ok_or_else(|| ConfigError::MissingValue("railway_project_id".to_string()))?;

        let plan = yaml_config.railway_plan.unwrap_or_default();

        let scrape_interval = yaml_config.scrape_interval.unwrap_or(300);

        // Validate scrape_interval range
        if scrape_interval < 60 {
            return Err(ConfigError::InvalidValue(
                "scrape_interval must be at least 60 seconds".to_string(),
            ));
        }
        if scrape_interval > 3600 {
            return Err(ConfigError::InvalidValue(
                "scrape_interval must be at most 3600 seconds".to_string(),
            ));
        }

        let port = yaml_config.port.unwrap_or(9090);

        let api_url = yaml_config
            .railway_api_url
            .unwrap_or_else(|| DEFAULT_API_URL.to_string());

        // Build pricing config
        let mut pricing = PricingConfig::new(plan.as_str());

        // Apply YAML pricing overrides for the selected plan
        if let Some(ref entries) = yaml_config.pricing {
            // Find pricing entry matching current plan
            if let Some(entry) = entries
                .iter()
                .find(|e| e.name.to_lowercase() == plan.as_str())
            {
                if let Some(cpu) = entry.price.cpu {
                    pricing.set_price("CPU_USAGE", cpu);
                }
                if let Some(memory) = entry.price.memory {
                    pricing.set_price("MEMORY_USAGE_GB", memory);
                }
                if let Some(disk) = entry.price.disk {
                    pricing.set_price("DISK_USAGE_GB", disk);
                }
                if let Some(ref network) = entry.price.network {
                    if let Some(tx) = network.tx {
                        pricing.set_price("NETWORK_TX_GB", tx);
                    }
                    if let Some(rx) = network.rx {
                        pricing.set_price("NETWORK_RX_GB", rx);
                    }
                }
            }
        }

        // Get pricing values for current plan (for API response)
        let pricing_values = yaml_config
            .pricing
            .as_ref()
            .and_then(|entries| entries.iter().find(|e| e.name.to_lowercase() == plan.as_str()))
            .map(|e| e.price.clone())
            .unwrap_or_default();

        let service_groups = yaml_config.service_groups.unwrap_or_default();

        // Default project_name to project_id if not specified
        let project_name = yaml_config
            .project_name
            .unwrap_or_else(|| project_id.clone());

        let cors_enabled = yaml_config.cors_enabled.unwrap_or(true);
        let websocket_enabled = yaml_config.websocket_enabled.unwrap_or(true);

        Ok(Self {
            api_token,
            project_id,
            plan,
            scrape_interval,
            port,
            api_url,
            pricing,
            pricing_values,
            service_groups,
            project_name,
            cors_enabled,
            websocket_enabled,
        })
    }

    /// Loads YAML configuration from Base64 env var or file.
    fn load_yaml_config() -> Result<YamlConfig, ConfigError> {
        // Try CONFIG_BASE64 first (for Docker/Railway)
        if let Ok(b64) = env::var("CONFIG_BASE64") {
            let bytes = STANDARD
                .decode(&b64)
                .map_err(|e| ConfigError::Base64Error(e.to_string()))?;

            let content =
                String::from_utf8(bytes).map_err(|e| ConfigError::Base64Error(e.to_string()))?;

            return serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::YamlError(e.to_string()));
        }

        // Try CONFIG_FILE env var
        if let Ok(path) = env::var("CONFIG_FILE") {
            if Path::new(&path).exists() {
                let content =
                    fs::read_to_string(&path).map_err(|e| ConfigError::FileError(e.to_string()))?;

                return serde_yaml::from_str(&content)
                    .map_err(|e| ConfigError::YamlError(e.to_string()));
            }
        }

        // Try default config.yaml
        if Path::new("config.yaml").exists() {
            let content = fs::read_to_string("config.yaml")
                .map_err(|e| ConfigError::FileError(e.to_string()))?;

            return serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::YamlError(e.to_string()));
        }

        // No config found
        Err(ConfigError::FileError(
            "No config found. Provide CONFIG_BASE64 env var or config.yaml file".to_string(),
        ))
    }

    /// Creates a configuration with explicit values (for testing).
    ///
    /// # Arguments
    ///
    /// * `api_token` - Railway API token
    /// * `project_id` - Railway project ID
    /// * `plan` - Pricing plan (Hobby or Pro)
    /// * `scrape_interval` - Seconds between API queries
    /// * `port` - HTTP server port
    ///
    /// # Example
    ///
    /// ```rust
    /// use railway_exporter::{Config, config::Plan};
    ///
    /// let config = Config::new("token", "project-id", Plan::Pro, 60, 8080);
    /// assert_eq!(config.plan, Plan::Pro);
    /// ```
    pub fn new(
        api_token: &str,
        project_id: &str,
        plan: Plan,
        scrape_interval: u16,
        port: u16,
    ) -> Self {
        Self {
            api_token: api_token.to_string(),
            project_id: project_id.to_string(),
            plan,
            scrape_interval,
            port,
            api_url: DEFAULT_API_URL.to_string(),
            pricing: PricingConfig::new(plan.as_str()),
            pricing_values: PriceValues::default(),
            service_groups: HashMap::new(),
            project_name: project_id.to_string(),
            cors_enabled: true,
            websocket_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new("token", "project", Plan::Pro, 60, 8080);
        assert_eq!(config.api_token, "token");
        assert_eq!(config.project_id, "project");
        assert_eq!(config.plan, Plan::Pro);
        assert_eq!(config.scrape_interval, 60);
        assert_eq!(config.port, 8080);
        assert!(config.service_groups.is_empty());
    }

    #[test]
    fn test_config_api_url() {
        let config = Config::new("t", "p", Plan::Hobby, 60, 8080);
        assert_eq!(config.api_url, DEFAULT_API_URL);
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::MissingValue("TEST".to_string());
        assert_eq!(format!("{}", err), "Missing required config: TEST");

        let err = ConfigError::YamlError("invalid".to_string());
        assert_eq!(format!("{}", err), "YAML parse error: invalid");

        let err = ConfigError::ParseError("PORT".to_string(), "not a number".to_string());
        assert_eq!(format!("{}", err), "Failed to parse PORT: not a number");

        let err = ConfigError::FileError("not found".to_string());
        assert_eq!(format!("{}", err), "Config file error: not found");

        let err = ConfigError::Base64Error("invalid".to_string());
        assert_eq!(format!("{}", err), "Base64 decode error: invalid");

        let err = ConfigError::InvalidPlan("enterprise".to_string());
        assert_eq!(
            format!("{}", err),
            "Invalid plan 'enterprise': must be 'hobby' or 'pro'"
        );
    }

    #[test]
    fn test_config_pricing_uses_plan() {
        let hobby = Config::new("t", "p", Plan::Hobby, 60, 8080);
        let pro = Config::new("t", "p", Plan::Pro, 60, 8080);

        // Pro should have lower CPU price
        assert!(pro.pricing.get_price("CPU_USAGE") < hobby.pricing.get_price("CPU_USAGE"));
    }

    #[test]
    fn test_plan_from_str() {
        assert_eq!(Plan::from_str("hobby").unwrap(), Plan::Hobby);
        assert_eq!(Plan::from_str("HOBBY").unwrap(), Plan::Hobby);
        assert_eq!(Plan::from_str("pro").unwrap(), Plan::Pro);
        assert_eq!(Plan::from_str("PRO").unwrap(), Plan::Pro);
        assert_eq!(Plan::from_str("Pro").unwrap(), Plan::Pro);
    }

    #[test]
    fn test_plan_from_str_invalid() {
        let result = Plan::from_str("enterprise");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidPlan(v) => assert_eq!(v, "enterprise"),
            _ => panic!("Expected InvalidPlan error"),
        }
    }

    #[test]
    fn test_plan_display() {
        assert_eq!(format!("{}", Plan::Hobby), "hobby");
        assert_eq!(format!("{}", Plan::Pro), "pro");
    }

    #[test]
    fn test_plan_as_str() {
        assert_eq!(Plan::Hobby.as_str(), "hobby");
        assert_eq!(Plan::Pro.as_str(), "pro");
    }

    #[test]
    fn test_plan_default() {
        assert_eq!(Plan::default(), Plan::Hobby);
    }

    #[test]
    fn test_yaml_config_deserialize() {
        let yaml = r#"
railway_api_token: test-token
railway_project_id: test-project
railway_plan: pro
port: 9090
scrape_interval: 120
service_groups:
  monitoring:
    - prometheus
    - grafana
  database:
    - postgres
"#;
        let config: YamlConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.railway_api_token.unwrap(), "test-token");
        assert_eq!(config.railway_project_id.unwrap(), "test-project");
        assert_eq!(config.railway_plan.unwrap(), Plan::Pro);
        assert_eq!(config.port.unwrap(), 9090);
        assert_eq!(config.scrape_interval.unwrap(), 120);

        let groups = config.service_groups.unwrap();
        assert_eq!(
            groups.get("monitoring").unwrap(),
            &vec!["prometheus", "grafana"]
        );
        assert_eq!(groups.get("database").unwrap(), &vec!["postgres"]);
    }
}
