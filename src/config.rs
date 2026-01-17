//! Configuration management for Railway Exporter.
//!
//! Configuration can be loaded from:
//! 1. Environment variables (highest priority)
//! 2. Base64-encoded TOML in `CONFIG_BASE64` env var
//! 3. TOML config file
//!
//! ## Environment Variables
//!
//! | Variable | Required | Default | Description |
//! |----------|:--------:|---------|-------------|
//! | `RAILWAY_API_TOKEN` | Yes | — | Railway API token |
//! | `RAILWAY_PROJECT_ID` | Yes | — | Project ID to monitor |
//! | `RAILWAY_PLAN` | No | `hobby` | Pricing plan (`hobby` or `pro`) |
//! | `SCRAPE_INTERVAL` | No | `300` | API query interval in seconds |
//! | `PORT` | No | `9333` | HTTP server port |
//! | `CONFIG_BASE64` | No | — | Base64-encoded TOML config |
//! | `CONFIG_FILE` | No | — | Path to TOML config file |
//!
//! ## TOML Config Format
//!
//! ```toml
//! [railway]
//! api_token = "your-token"
//! project_id = "your-project-id"
//! plan = "pro"
//!
//! [pricing]
//! cpu = 0.000231
//! memory = 0.000116
//! disk = 0.000021
//! network = 0.10
//!
//! [server]
//! port = 9333
//! scrape_interval = 300
//! ```

use crate::pricing::PricingConfig;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;
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

/// TOML configuration structure.
#[derive(Debug, Deserialize, Default)]
struct TomlConfig {
    railway: Option<RailwaySection>,
    pricing: Option<PricingSection>,
    server: Option<ServerSection>,
}

#[derive(Debug, Deserialize, Default)]
struct RailwaySection {
    api_token: Option<String>,
    project_id: Option<String>,
    plan: Option<Plan>,
    api_url: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct PricingSection {
    cpu: Option<f64>,
    memory: Option<f64>,
    disk: Option<f64>,
    network: Option<f64>,
}

#[derive(Debug, Deserialize, Default)]
struct ServerSection {
    port: Option<u16>,
    scrape_interval: Option<u64>,
}

/// Configuration for the Railway Exporter.
///
/// # Example
///
/// ```rust,no_run
/// use railway_exporter::Config;
///
/// // Load from environment variables
/// let config = Config::load().expect("Missing required config");
///
/// println!("Monitoring project: {}", config.project_id);
/// println!("Using {} plan pricing", config.plan);
/// ```
/// Default Railway GraphQL API URL.
pub const DEFAULT_API_URL: &str = "https://backboard.railway.app/graphql/v2";

#[derive(Debug, Clone)]
pub struct Config {
    /// Railway API token for authentication.
    pub api_token: String,

    /// Railway project ID to monitor.
    pub project_id: String,

    /// Pricing plan (Hobby or Pro).
    pub plan: Plan,

    /// Interval between API queries in seconds.
    pub scrape_interval: u64,

    /// HTTP server port for metrics endpoint.
    pub port: u16,

    /// Railway GraphQL API URL.
    pub api_url: String,

    /// Custom pricing configuration.
    pub pricing: PricingConfig,
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
    /// Failed to parse TOML.
    TomlError(String),
    /// Failed to decode Base64.
    Base64Error(String),
    /// Invalid plan value (must be "hobby" or "pro").
    InvalidPlan(String),
    /// Invalid configuration value (out of range, wrong format, etc.).
    InvalidValue(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingValue(key) => write!(f, "Missing required config: {}", key),
            ConfigError::ParseError(key, msg) => write!(f, "Failed to parse {}: {}", key, msg),
            ConfigError::FileError(msg) => write!(f, "Config file error: {}", msg),
            ConfigError::TomlError(msg) => write!(f, "TOML parse error: {}", msg),
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
    /// Loads configuration from all sources.
    ///
    /// Priority (highest to lowest):
    /// 1. Environment variables
    /// 2. Base64-encoded TOML in `CONFIG_BASE64`
    /// 3. TOML file specified in `CONFIG_FILE`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use railway_exporter::Config;
    ///
    /// let config = Config::load().expect("Failed to load config");
    /// ```
    pub fn load() -> Result<Self, ConfigError> {
        // Try to load TOML config first (lowest priority, will be overridden)
        let toml_config = Self::load_toml_config()?;

        // Get values with priority: env > toml
        let api_token = env::var("RAILWAY_API_TOKEN")
            .ok()
            .or_else(|| toml_config.railway.as_ref()?.api_token.clone())
            .ok_or_else(|| ConfigError::MissingValue("RAILWAY_API_TOKEN".to_string()))?;

        let project_id = env::var("RAILWAY_PROJECT_ID")
            .ok()
            .or_else(|| toml_config.railway.as_ref()?.project_id.clone())
            .ok_or_else(|| ConfigError::MissingValue("RAILWAY_PROJECT_ID".to_string()))?;

        let plan = if let Ok(plan_str) = env::var("RAILWAY_PLAN") {
            Plan::from_str(&plan_str)?
        } else {
            toml_config
                .railway
                .as_ref()
                .and_then(|r| r.plan)
                .unwrap_or_default()
        };

        let scrape_interval = env::var("SCRAPE_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .or_else(|| toml_config.server.as_ref()?.scrape_interval)
            .unwrap_or(300);

        // Validate scrape_interval range
        if scrape_interval < 60 {
            return Err(ConfigError::InvalidValue(
                "SCRAPE_INTERVAL must be at least 60 seconds".to_string(),
            ));
        }
        if scrape_interval > 3600 {
            return Err(ConfigError::InvalidValue(
                "SCRAPE_INTERVAL must be at most 3600 seconds".to_string(),
            ));
        }

        let port = env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .or_else(|| toml_config.server.as_ref()?.port)
            .unwrap_or(9333);

        let api_url = env::var("RAILWAY_API_URL")
            .ok()
            .or_else(|| toml_config.railway.as_ref()?.api_url.clone())
            .unwrap_or_else(|| DEFAULT_API_URL.to_string());

        // Build pricing config
        let mut pricing = PricingConfig::new(plan.as_str());

        // Apply TOML pricing overrides
        if let Some(ref p) = toml_config.pricing {
            if let Some(cpu) = p.cpu {
                pricing.set_price("CPU_USAGE", cpu);
            }
            if let Some(memory) = p.memory {
                pricing.set_price("MEMORY_USAGE_GB", memory);
            }
            if let Some(disk) = p.disk {
                pricing.set_price("DISK_USAGE_GB", disk);
            }
            if let Some(network) = p.network {
                pricing.set_price("NETWORK_TX_GB", network);
            }
        }

        // Apply env var pricing overrides (highest priority)
        if let Ok(price) = env::var("CUSTOM_CPU_PRICE")
            .and_then(|s| s.parse().map_err(|_| env::VarError::NotPresent))
        {
            pricing.set_price("CPU_USAGE", price);
        }
        if let Ok(price) = env::var("CUSTOM_MEMORY_PRICE")
            .and_then(|s| s.parse().map_err(|_| env::VarError::NotPresent))
        {
            pricing.set_price("MEMORY_USAGE_GB", price);
        }
        if let Ok(price) = env::var("CUSTOM_DISK_PRICE")
            .and_then(|s| s.parse().map_err(|_| env::VarError::NotPresent))
        {
            pricing.set_price("DISK_USAGE_GB", price);
        }
        if let Ok(price) = env::var("CUSTOM_NETWORK_PRICE")
            .and_then(|s| s.parse().map_err(|_| env::VarError::NotPresent))
        {
            pricing.set_price("NETWORK_TX_GB", price);
        }

        Ok(Self {
            api_token,
            project_id,
            plan,
            scrape_interval,
            port,
            api_url,
            pricing,
        })
    }

    /// Loads TOML configuration from Base64 env var or file.
    fn load_toml_config() -> Result<TomlConfig, ConfigError> {
        // Try CONFIG_BASE64 first
        if let Ok(b64) = env::var("CONFIG_BASE64") {
            let bytes = STANDARD
                .decode(&b64)
                .map_err(|e| ConfigError::Base64Error(e.to_string()))?;
            let content =
                String::from_utf8(bytes).map_err(|e| ConfigError::Base64Error(e.to_string()))?;
            return toml::from_str(&content).map_err(|e| ConfigError::TomlError(e.to_string()));
        }

        // Try CONFIG_FILE
        if let Ok(path) = env::var("CONFIG_FILE") {
            if Path::new(&path).exists() {
                let content =
                    fs::read_to_string(&path).map_err(|e| ConfigError::FileError(e.to_string()))?;
                return toml::from_str(&content).map_err(|e| ConfigError::TomlError(e.to_string()));
            }
        }

        // Try default config.toml
        if Path::new("config.toml").exists() {
            let content = fs::read_to_string("config.toml")
                .map_err(|e| ConfigError::FileError(e.to_string()))?;
            return toml::from_str(&content).map_err(|e| ConfigError::TomlError(e.to_string()));
        }

        // No TOML config, return empty
        Ok(TomlConfig::default())
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
        scrape_interval: u64,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests that manipulate env vars are skipped as they can't run
    // reliably in parallel. Use integration tests for Config::load() testing.

    #[test]
    fn test_config_new() {
        let config = Config::new("token", "project", Plan::Pro, 60, 8080);
        assert_eq!(config.api_token, "token");
        assert_eq!(config.project_id, "project");
        assert_eq!(config.plan, Plan::Pro);
        assert_eq!(config.scrape_interval, 60);
        assert_eq!(config.port, 8080);
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

        let err = ConfigError::TomlError("invalid".to_string());
        assert_eq!(format!("{}", err), "TOML parse error: invalid");

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
}
