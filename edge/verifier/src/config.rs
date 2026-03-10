//! Site-specific configuration for edge verifier.

use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to load config file: {0}")]
    LoadError(#[from] config::ConfigError),

    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
}

/// Edge verifier configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct VerifierConfig {
    /// Unique site identifier (ULID with VRF_ prefix).
    pub site_id: String,

    /// Property ID this verifier belongs to.
    pub property_id: String,

    /// Tenant ID for multi-tenant isolation.
    pub tenant_id: String,

    /// Platform API base URL.
    #[serde(default = "default_api_url")]
    pub api_base_url: String,

    /// MQTT broker URL.
    #[serde(default = "default_mqtt_url")]
    pub mqtt_broker_url: String,

    /// MQTT client ID (defaults to site_id).
    pub mqtt_client_id: Option<String>,

    /// Path to audit database.
    #[serde(default = "default_audit_db_path")]
    pub audit_db_path: PathBuf,

    /// Status list cache TTL in seconds.
    #[serde(default = "default_status_list_ttl", with = "duration_secs")]
    pub status_list_ttl: Duration,

    /// Maximum age for stale status list before deny-all.
    #[serde(default = "default_stale_threshold", with = "duration_secs")]
    pub stale_threshold: Duration,

    /// Trust registry cache TTL.
    #[serde(default = "default_trust_registry_ttl", with = "duration_secs")]
    pub trust_registry_ttl: Duration,

    /// BLE configuration.
    #[serde(default)]
    pub ble: BleConfig,

    /// NFC configuration.
    #[serde(default)]
    pub nfc: NfcConfig,

    /// Hardware GPIO configuration.
    #[serde(default)]
    pub hardware: HardwareConfig,

    /// Offline mode settings.
    #[serde(default)]
    pub offline: OfflineConfig,
}

/// BLE-specific configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct BleConfig {
    /// BLE adapter name (e.g., "hci0").
    #[serde(default = "default_ble_adapter")]
    pub adapter: String,

    /// Whether BLE is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Challenge nonce validity duration.
    #[serde(default = "default_challenge_ttl", with = "duration_secs")]
    pub challenge_ttl: Duration,
}

impl Default for BleConfig {
    fn default() -> Self {
        Self {
            adapter: default_ble_adapter(),
            enabled: true,
            challenge_ttl: default_challenge_ttl(),
        }
    }
}

/// NFC-specific configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct NfcConfig {
    /// NFC reader name pattern.
    #[serde(default = "default_nfc_reader")]
    pub reader: String,

    /// Whether NFC is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Poll interval for card detection.
    #[serde(default = "default_nfc_poll_interval", with = "duration_millis")]
    pub poll_interval: Duration,
}

impl Default for NfcConfig {
    fn default() -> Self {
        Self {
            reader: default_nfc_reader(),
            enabled: true,
            poll_interval: default_nfc_poll_interval(),
        }
    }
}

/// Hardware GPIO configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct HardwareConfig {
    /// GPIO pin for door lock relay.
    #[serde(default = "default_relay_pin")]
    pub relay_pin: u8,

    /// GPIO pin for green LED (granted).
    #[serde(default = "default_green_led_pin")]
    pub green_led_pin: u8,

    /// GPIO pin for red LED (denied).
    #[serde(default = "default_red_led_pin")]
    pub red_led_pin: u8,

    /// GPIO pin for amber LED (processing).
    #[serde(default = "default_amber_led_pin")]
    pub amber_led_pin: u8,

    /// GPIO pin for buzzer.
    #[serde(default = "default_buzzer_pin")]
    pub buzzer_pin: u8,

    /// GPIO pin for tamper switch.
    #[serde(default = "default_tamper_pin")]
    pub tamper_pin: u8,

    /// Door unlock duration.
    #[serde(default = "default_unlock_duration", with = "duration_secs")]
    pub unlock_duration: Duration,

    /// Whether hardware GPIO is enabled (false for simulation).
    #[serde(default)]
    pub enabled: bool,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            relay_pin: default_relay_pin(),
            green_led_pin: default_green_led_pin(),
            red_led_pin: default_red_led_pin(),
            amber_led_pin: default_amber_led_pin(),
            buzzer_pin: default_buzzer_pin(),
            tamper_pin: default_tamper_pin(),
            unlock_duration: default_unlock_duration(),
            enabled: false, // Disabled by default for dev
        }
    }
}

/// Offline operation configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct OfflineConfig {
    /// Allow cached credentials in offline mode.
    #[serde(default = "default_true")]
    pub allow_cached: bool,

    /// Maximum offline duration before lockdown.
    #[serde(default = "default_max_offline_duration", with = "duration_secs")]
    pub max_offline_duration: Duration,

    /// Pre-approved holder DIDs for offline mode.
    #[serde(default)]
    pub allow_list: Vec<String>,
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            allow_cached: true,
            max_offline_duration: default_max_offline_duration(),
            allow_list: Vec::new(),
        }
    }
}

impl VerifierConfig {
    /// Load configuration from file and environment.
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if present
        let _ = dotenvy::dotenv();

        let config_path = std::env::var("VERIFIER_CONFIG")
            .unwrap_or_else(|_| "/etc/vaultpass/verifier.toml".to_string());

        let settings = config::Config::builder()
            // Start with defaults
            .set_default("api_base_url", default_api_url())?
            .set_default("mqtt_broker_url", default_mqtt_url())?
            // Load from file
            .add_source(config::File::with_name(&config_path).required(false))
            // Override with environment variables
            .add_source(
                config::Environment::with_prefix("VERIFIER")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let config: VerifierConfig = settings.try_deserialize()?;
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.site_id.is_empty() {
            return Err(ConfigError::ValidationError(
                "site_id is required".to_string(),
            ));
        }

        if !self.site_id.starts_with("VRF_") {
            return Err(ConfigError::ValidationError(
                "site_id must have VRF_ prefix".to_string(),
            ));
        }

        if self.property_id.is_empty() {
            return Err(ConfigError::ValidationError(
                "property_id is required".to_string(),
            ));
        }

        if self.tenant_id.is_empty() {
            return Err(ConfigError::ValidationError(
                "tenant_id is required".to_string(),
            ));
        }

        Ok(())
    }

    /// Get MQTT client ID.
    pub fn mqtt_client_id(&self) -> String {
        self.mqtt_client_id
            .clone()
            .unwrap_or_else(|| self.site_id.clone())
    }
}

// Default value functions
fn default_api_url() -> String {
    "https://api.sahi.my".to_string()
}

fn default_mqtt_url() -> String {
    "mqtt://mqtt.sahi.my:8883".to_string()
}

fn default_audit_db_path() -> PathBuf {
    PathBuf::from("/var/lib/vaultpass/audit.db")
}

fn default_status_list_ttl() -> Duration {
    Duration::from_secs(900) // 15 minutes
}

fn default_stale_threshold() -> Duration {
    Duration::from_secs(4 * 3600) // 4 hours
}

fn default_trust_registry_ttl() -> Duration {
    Duration::from_secs(4 * 3600) // 4 hours
}

fn default_ble_adapter() -> String {
    "hci0".to_string()
}

fn default_challenge_ttl() -> Duration {
    Duration::from_secs(30)
}

fn default_nfc_reader() -> String {
    "ACS ACR122U".to_string()
}

fn default_nfc_poll_interval() -> Duration {
    Duration::from_millis(100)
}

fn default_relay_pin() -> u8 {
    17
}
fn default_green_led_pin() -> u8 {
    27
}
fn default_red_led_pin() -> u8 {
    22
}
fn default_amber_led_pin() -> u8 {
    23
}
fn default_buzzer_pin() -> u8 {
    24
}
fn default_tamper_pin() -> u8 {
    25
}

fn default_unlock_duration() -> Duration {
    Duration::from_secs(5)
}

fn default_max_offline_duration() -> Duration {
    Duration::from_secs(24 * 3600) // 24 hours
}

fn default_true() -> bool {
    true
}

/// Serde helper for Duration in seconds.
mod duration_secs {
    use serde::{Deserialize, Deserializer};
    use std::time::Duration;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Serde helper for Duration in milliseconds.
mod duration_millis {
    use serde::{Deserialize, Deserializer};
    use std::time::Duration;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        assert_eq!(default_status_list_ttl(), Duration::from_secs(900));
        assert_eq!(default_stale_threshold(), Duration::from_secs(4 * 3600));
        assert_eq!(default_relay_pin(), 17);
    }

    #[test]
    fn test_validate_site_id() {
        let config = VerifierConfig {
            site_id: "VRF_01HXK".to_string(),
            property_id: "PRY_01HXK".to_string(),
            tenant_id: "TNT_01HXK".to_string(),
            api_base_url: default_api_url(),
            mqtt_broker_url: default_mqtt_url(),
            mqtt_client_id: None,
            audit_db_path: default_audit_db_path(),
            status_list_ttl: default_status_list_ttl(),
            stale_threshold: default_stale_threshold(),
            trust_registry_ttl: default_trust_registry_ttl(),
            ble: BleConfig::default(),
            nfc: NfcConfig::default(),
            hardware: HardwareConfig::default(),
            offline: OfflineConfig::default(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_site_id_prefix() {
        let config = VerifierConfig {
            site_id: "INVALID_01HXK".to_string(),
            property_id: "PRY_01HXK".to_string(),
            tenant_id: "TNT_01HXK".to_string(),
            api_base_url: default_api_url(),
            mqtt_broker_url: default_mqtt_url(),
            mqtt_client_id: None,
            audit_db_path: default_audit_db_path(),
            status_list_ttl: default_status_list_ttl(),
            stale_threshold: default_stale_threshold(),
            trust_registry_ttl: default_trust_registry_ttl(),
            ble: BleConfig::default(),
            nfc: NfcConfig::default(),
            hardware: HardwareConfig::default(),
            offline: OfflineConfig::default(),
        };

        assert!(config.validate().is_err());
    }
}
