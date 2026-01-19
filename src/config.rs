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
//! # Gzip compression settings
//! gzip:
//!   enabled: true       # Enable gzip compression (default: true)
//!   min_size: 256       # Minimum response size in bytes (default: 256)
//!   level: 1            # Compression level 1-9 (default: 1 = fast)
//!
//! # Icon cache settings (LRU cache with raw bytes storage)
//! icon_cache:
//!   enabled: true       # Enable icon caching (default: true)
//!   max_count: 200      # Max icons to cache (default: 200)
//!   mode: link          # "base64" = embed in JSON, "link" = serve from /static/icons/services/{name}
//!   max_age: 86400      # Browser cache TTL in seconds (for mode: link, default: 86400 = 1 day)
//!
//! pricing:
//!   - name: hobby
//!     price:
//!       cpu: 0.000463
//!       memory: 0.000231
//!   - name: pro
//!     price:
//!       cpu: 0.000231
//!       memory: 0.000116
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
pub(crate) struct YamlConfig {
    pub(crate) railway_api_token: Option<String>,
    pub(crate) railway_project_id: Option<String>,
    pub(crate) railway_plan: Option<Plan>,
    pub(crate) railway_api_url: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) scrape_interval: Option<u16>,
    pub(crate) pricing: Option<PricingSection>,
    pub(crate) service_groups: Option<HashMap<String, Vec<String>>>,
    /// Project display name (for /status endpoint).
    pub(crate) project_name: Option<String>,
    /// Enable CORS headers on all responses.
    pub(crate) cors_enabled: Option<bool>,
    /// Enable WebSocket endpoint.
    pub(crate) websocket_enabled: Option<bool>,
    /// Gzip compression settings.
    pub(crate) gzip: Option<GzipConfig>,
    /// Icon cache settings.
    pub(crate) icon_cache: Option<IconCacheConfig>,
}

use serde::Serialize;

/// Gzip compression configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GzipConfig {
    /// Enable gzip compression for HTTP responses.
    #[serde(default = "default_gzip_enabled")]
    pub enabled: bool,
    /// Minimum response size in bytes to trigger compression.
    #[serde(default = "default_gzip_min_size")]
    pub min_size: usize,
    /// Compression level (1-9). 1 = fast, 9 = best compression.
    #[serde(default = "default_gzip_level")]
    pub level: u32,
}

fn default_gzip_enabled() -> bool {
    true
}
fn default_gzip_min_size() -> usize {
    256
}
fn default_gzip_level() -> u32 {
    1
}

impl Default for GzipConfig {
    fn default() -> Self {
        Self {
            enabled: default_gzip_enabled(),
            min_size: default_gzip_min_size(),
            level: default_gzip_level(),
        }
    }
}

/// Icon delivery mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IconMode {
    /// Embed icons as base64 data URLs in JSON responses.
    /// Larger payload but no additional requests needed.
    #[default]
    Base64,
    /// Return URLs to /icons/{service} endpoint.
    /// Smaller JSON payload, browser caches icons separately.
    Link,
}

impl IconMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            IconMode::Base64 => "base64",
            IconMode::Link => "link",
        }
    }
}

impl fmt::Display for IconMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Icon cache configuration.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IconCacheConfig {
    /// Enable icon caching.
    #[serde(default = "default_icon_cache_enabled")]
    pub enabled: bool,
    /// Maximum number of icons to cache (LRU eviction).
    #[serde(default = "default_icon_cache_max_count")]
    pub max_count: usize,
    /// Icon delivery mode: "base64" or "link".
    #[serde(default)]
    pub mode: IconMode,
    /// Browser cache TTL in seconds (for mode: link).
    /// Default: 86400 (1 day).
    #[serde(default = "default_icon_cache_max_age")]
    pub max_age: u32,
}

fn default_icon_cache_enabled() -> bool {
    true
}
fn default_icon_cache_max_count() -> usize {
    200
}
fn default_icon_cache_max_age() -> u32 {
    86400 // 1 day
}

impl Default for IconCacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_icon_cache_enabled(),
            max_count: default_icon_cache_max_count(),
            mode: IconMode::default(),
            max_age: default_icon_cache_max_age(),
        }
    }
}

/// Network pricing (tx/rx).
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct NetworkPricing {
    pub tx: Option<f64>,
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

    /// Gzip compression settings.
    pub gzip: GzipConfig,

    /// Icon cache settings.
    pub icon_cache: IconCacheConfig,
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

        // Gzip configuration with validation
        let gzip = yaml_config.gzip.unwrap_or_default();
        if gzip.level < 1 || gzip.level > 9 {
            return Err(ConfigError::InvalidValue(
                "gzip.level must be between 1 and 9".to_string(),
            ));
        }

        let icon_cache = yaml_config.icon_cache.unwrap_or_default();

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
            gzip,
            icon_cache,
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
            gzip: GzipConfig::default(),
            icon_cache: IconCacheConfig::default(),
        }
    }
}