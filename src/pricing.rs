//! Pricing calculations for Railway resources.
//!
//! Railway uses different pricing tiers for Hobby and Pro plans.
//! See [Railway Pricing](https://railway.app/pricing) for current rates.
//!
//! ## Pricing Table (as of 2024)
//!
//! | Resource | Hobby | Pro |
//! |----------|------:|----:|
//! | CPU | $0.000463/vCPU-min | $0.000231/vCPU-min |
//! | Memory | $0.000231/GB-min | $0.000116/GB-min |
//! | Disk | $0.000021/GB-min | $0.000021/GB-min |
//! | Egress | $0.10/GB | $0.10/GB |

use serde::Serialize;
use std::collections::HashMap;

/// Default pricing for Hobby plan (per unit).
pub const HOBBY_PRICES: &[(&str, f64)] = &[
    ("CPU_USAGE", 0.000463),
    ("MEMORY_USAGE_GB", 0.000231),
    ("DISK_USAGE_GB", 0.000021),
    ("NETWORK_TX_GB", 0.10),
];

/// Default pricing for Pro plan (per unit).
pub const PRO_PRICES: &[(&str, f64)] = &[
    ("CPU_USAGE", 0.000231),
    ("MEMORY_USAGE_GB", 0.000116),
    ("DISK_USAGE_GB", 0.000021),
    ("NETWORK_TX_GB", 0.10),
];

/// Custom pricing configuration.
///
/// Allows overriding default prices for specific measurements.
///
/// # Example
///
/// ```rust
/// use railway_exporter::pricing::PricingConfig;
///
/// let mut config = PricingConfig::new("pro");
/// config.set_price("CPU_USAGE", 0.0003); // Custom CPU price
///
/// assert_eq!(config.get_price("CPU_USAGE"), 0.0003);
/// assert_eq!(config.get_price("MEMORY_USAGE_GB"), 0.000116); // Default Pro price
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct PricingConfig {
    plan: String,
    overrides: HashMap<String, f64>,
}

impl PricingConfig {
    /// Creates a new pricing configuration for the given plan.
    ///
    /// # Arguments
    ///
    /// * `plan` - Either "hobby" or "pro"
    ///
    /// # Example
    ///
    /// ```rust
    /// use railway_exporter::pricing::PricingConfig;
    ///
    /// let config = PricingConfig::new("pro");
    /// assert_eq!(config.get_price("CPU_USAGE"), 0.000231);
    /// ```
    pub fn new(plan: &str) -> Self {
        Self {
            plan: plan.to_lowercase(),
            overrides: HashMap::new(),
        }
    }

    /// Sets a custom price for a measurement, overriding the default.
    ///
    /// # Arguments
    ///
    /// * `measurement` - The measurement type (e.g., "CPU_USAGE")
    /// * `price` - The price per unit
    pub fn set_price(&mut self, measurement: &str, price: f64) {
        self.overrides.insert(measurement.to_string(), price);
    }

    /// Gets the price for a measurement.
    ///
    /// Returns the custom price if set, otherwise the default for the plan.
    ///
    /// # Arguments
    ///
    /// * `measurement` - The measurement type (e.g., "CPU_USAGE", "MEMORY_USAGE_GB")
    ///
    /// # Returns
    ///
    /// The price per unit, or 0.0 if the measurement is unknown.
    pub fn get_price(&self, measurement: &str) -> f64 {
        if let Some(&price) = self.overrides.get(measurement) {
            return price;
        }
        get_price(&self.plan, measurement)
    }

    /// Returns the plan name.
    pub fn plan(&self) -> &str {
        &self.plan
    }
}

/// Gets the default price for a measurement based on the plan.
///
/// # Arguments
///
/// * `plan` - The pricing plan ("hobby" or "pro")
/// * `measurement` - The measurement type
///
/// # Returns
///
/// The price per unit. Returns 0.0 for unknown measurements.
///
/// # Example
///
/// ```rust
/// use railway_exporter::pricing::get_price;
///
/// // Pro plan CPU is cheaper
/// assert_eq!(get_price("pro", "CPU_USAGE"), 0.000231);
/// assert_eq!(get_price("hobby", "CPU_USAGE"), 0.000463);
///
/// // Disk and network are the same on both plans
/// assert_eq!(get_price("pro", "DISK_USAGE_GB"), 0.000021);
/// assert_eq!(get_price("hobby", "DISK_USAGE_GB"), 0.000021);
/// ```
pub fn get_price(plan: &str, measurement: &str) -> f64 {
    match (plan.to_lowercase().as_str(), measurement) {
        ("pro", "CPU_USAGE") => 0.000231,
        ("pro", "MEMORY_USAGE_GB") => 0.000116,
        (_, "CPU_USAGE") => 0.000463,
        (_, "MEMORY_USAGE_GB") => 0.000231,
        (_, "DISK_USAGE_GB") => 0.000021,
        (_, "NETWORK_TX_GB") => 0.10,
        _ => 0.0,
    }
}

/// Calculates the total cost for a set of measurements.
///
/// # Arguments
///
/// * `plan` - The pricing plan ("hobby" or "pro")
/// * `measurements` - A map of measurement names to values
///
/// # Returns
///
/// The total cost in USD.
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use railway_exporter::pricing::calculate_cost;
///
/// let mut usage = HashMap::new();
/// usage.insert("CPU_USAGE".to_string(), 1000.0); // 1000 vCPU-minutes
/// usage.insert("MEMORY_USAGE_GB".to_string(), 500.0); // 500 GB-minutes
///
/// let cost = calculate_cost("pro", &usage);
/// // 1000 * 0.000231 + 500 * 0.000116 = 0.231 + 0.058 = 0.289
/// assert!((cost - 0.289).abs() < 0.001);
/// ```
pub fn calculate_cost(plan: &str, measurements: &HashMap<String, f64>) -> f64 {
    measurements
        .iter()
        .map(|(name, value)| value * get_price(plan, name))
        .sum()
}
