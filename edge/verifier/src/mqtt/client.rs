//! MQTT client for platform communication.
//!
//! Handles:
//! - Status list updates
//! - Configuration changes
//! - Heartbeat/presence
//! - Audit log sync triggers

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

/// MQTT errors.
#[derive(Debug, Error)]
pub enum MqttError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Not connected")]
    NotConnected,

    #[error("Publish failed: {0}")]
    PublishFailed(String),

    #[error("Subscribe failed: {0}")]
    SubscribeFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// MQTT configuration.
#[derive(Debug, Clone)]
pub struct MqttConfig {
    /// Broker URL (mqtt://host:port).
    pub broker_url: String,

    /// Client ID.
    pub client_id: String,

    /// Username (optional).
    pub username: Option<String>,

    /// Password (optional).
    pub password: Option<String>,

    /// Keep-alive interval.
    pub keep_alive: Duration,

    /// QoS level (0, 1, or 2).
    pub qos: u8,

    /// Auto-reconnect.
    pub auto_reconnect: bool,

    /// Reconnect delay.
    pub reconnect_delay: Duration,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_url: "mqtt://localhost:1883".to_string(),
            client_id: "verifier".to_string(),
            username: None,
            password: None,
            keep_alive: Duration::from_secs(30),
            qos: 1,
            auto_reconnect: true,
            reconnect_delay: Duration::from_secs(5),
        }
    }
}

/// MQTT message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttMessage {
    /// Topic.
    pub topic: String,
    /// Payload.
    pub payload: Vec<u8>,
    /// QoS.
    pub qos: u8,
    /// Retain flag.
    pub retain: bool,
}

/// MQTT topic patterns.
pub mod topics {
    /// Status list update for a property.
    pub fn status_list_update(property_id: &str) -> String {
        format!("sahi/{}/status-list/update", property_id)
    }

    /// Configuration update for a verifier.
    pub fn config_update(site_id: &str) -> String {
        format!("sahi/verifier/{}/config", site_id)
    }

    /// Policy update for a property.
    pub fn policy_update(property_id: &str) -> String {
        format!("sahi/{}/policy/update", property_id)
    }

    /// Audit sync trigger.
    pub fn audit_sync(site_id: &str) -> String {
        format!("sahi/verifier/{}/audit/sync", site_id)
    }

    /// Verifier heartbeat.
    pub fn heartbeat(site_id: &str) -> String {
        format!("sahi/verifier/{}/heartbeat", site_id)
    }

    /// Access event (published by verifier).
    pub fn access_event(site_id: &str) -> String {
        format!("sahi/verifier/{}/access", site_id)
    }
}

/// Connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// MQTT client.
pub struct MqttClient {
    /// Configuration.
    config: MqttConfig,

    /// Connection state.
    state: Arc<RwLock<ConnectionState>>,

    /// Message receiver (for incoming messages).
    message_tx: mpsc::Sender<MqttMessage>,

    /// Subscribed topics.
    subscriptions: Arc<RwLock<Vec<String>>>,
}

impl MqttClient {
    /// Create a new MQTT client.
    pub fn new(config: MqttConfig) -> (Self, mpsc::Receiver<MqttMessage>) {
        let (message_tx, message_rx) = mpsc::channel(64);

        let client = Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            message_tx,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        };

        (client, message_rx)
    }

    /// Connect to the broker.
    pub async fn connect(&self) -> Result<(), MqttError> {
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connecting;
        }

        info!("Connecting to MQTT broker: {}", self.config.broker_url);

        // In production with rumqttc:
        // let mut options = MqttOptions::new(&self.config.client_id, host, port);
        // options.set_keep_alive(self.config.keep_alive);
        // if let (Some(user), Some(pass)) = (&self.config.username, &self.config.password) {
        //     options.set_credentials(user, pass);
        // }
        // let (client, eventloop) = AsyncClient::new(options, 64);
        // tokio::spawn(async move { ... handle eventloop ... });

        // Stub implementation
        #[cfg(feature = "simulation")]
        {
            info!("Simulation mode: MQTT connection simulated");
            let mut state = self.state.write().await;
            *state = ConnectionState::Connected;
            return Ok(());
        }

        #[cfg(not(feature = "simulation"))]
        {
            warn!("MQTT client not yet implemented - using stub");
            let mut state = self.state.write().await;
            *state = ConnectionState::Connected;
            Ok(())
        }
    }

    /// Disconnect from the broker.
    pub async fn disconnect(&self) -> Result<(), MqttError> {
        info!("Disconnecting from MQTT broker");

        let mut state = self.state.write().await;
        *state = ConnectionState::Disconnected;

        Ok(())
    }

    /// Subscribe to a topic.
    pub async fn subscribe(&self, topic: &str) -> Result<(), MqttError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(MqttError::NotConnected);
        }

        debug!("Subscribing to: {}", topic);

        let mut subs = self.subscriptions.write().await;
        if !subs.contains(&topic.to_string()) {
            subs.push(topic.to_string());
        }

        // In production with rumqttc:
        // client.subscribe(topic, QoS::from_u8(self.config.qos)).await
        //     .map_err(|e| MqttError::SubscribeFailed(e.to_string()))?;

        Ok(())
    }

    /// Unsubscribe from a topic.
    pub async fn unsubscribe(&self, topic: &str) -> Result<(), MqttError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(MqttError::NotConnected);
        }

        debug!("Unsubscribing from: {}", topic);

        let mut subs = self.subscriptions.write().await;
        subs.retain(|t| t != topic);

        Ok(())
    }

    /// Publish a message.
    pub async fn publish(
        &self,
        topic: &str,
        payload: &[u8],
        _retain: bool,
    ) -> Result<(), MqttError> {
        if *self.state.read().await != ConnectionState::Connected {
            return Err(MqttError::NotConnected);
        }

        debug!("Publishing to {}: {} bytes", topic, payload.len());

        // In production with rumqttc:
        // client.publish(topic, QoS::from_u8(self.config.qos), retain, payload).await
        //     .map_err(|e| MqttError::PublishFailed(e.to_string()))?;

        Ok(())
    }

    /// Publish a JSON message.
    pub async fn publish_json<T: Serialize>(
        &self,
        topic: &str,
        data: &T,
        retain: bool,
    ) -> Result<(), MqttError> {
        let payload =
            serde_json::to_vec(data).map_err(|e| MqttError::SerializationError(e.to_string()))?;
        self.publish(topic, &payload, retain).await
    }

    /// Send heartbeat.
    pub async fn send_heartbeat(&self, site_id: &str) -> Result<(), MqttError> {
        let heartbeat = Heartbeat {
            site_id: site_id.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            status: "online".to_string(),
        };

        self.publish_json(&topics::heartbeat(site_id), &heartbeat, false)
            .await
    }

    /// Get connection state.
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Check if connected.
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == ConnectionState::Connected
    }

    /// Get subscribed topics.
    pub async fn subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().await.clone()
    }

    /// Subscribe to standard verifier topics.
    pub async fn subscribe_verifier_topics(
        &self,
        site_id: &str,
        property_id: &str,
    ) -> Result<(), MqttError> {
        self.subscribe(&topics::status_list_update(property_id))
            .await?;
        self.subscribe(&topics::config_update(site_id)).await?;
        self.subscribe(&topics::policy_update(property_id)).await?;
        self.subscribe(&topics::audit_sync(site_id)).await?;
        Ok(())
    }
}

/// Heartbeat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Site ID.
    pub site_id: String,
    /// Timestamp.
    pub timestamp: String,
    /// Status.
    pub status: String,
}

/// Status list update message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusListUpdate {
    /// Status list URL.
    pub url: String,
    /// Credential ID.
    pub credential_id: String,
    /// Encoded list (base64).
    pub encoded_list: String,
}

/// Policy update message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyUpdate {
    /// Policy version.
    pub version: u32,
    /// Rules.
    pub rules: Vec<serde_json::Value>,
}

/// Access event message (published by verifier).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessEvent {
    /// Event ID.
    pub event_id: String,
    /// Timestamp.
    pub timestamp: String,
    /// Site ID.
    pub site_id: String,
    /// Holder DID (hashed or masked).
    pub holder_did_hash: String,
    /// Outcome.
    pub outcome: String,
    /// Method (BLE, NFC).
    pub method: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let config = MqttConfig::default();
        let (client, _rx) = MqttClient::new(config);

        assert_eq!(client.state().await, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let config = MqttConfig::default();
        let (client, _rx) = MqttClient::new(config);

        client.connect().await.unwrap();
        assert!(client.is_connected().await);

        client.disconnect().await.unwrap();
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_subscribe() {
        let config = MqttConfig::default();
        let (client, _rx) = MqttClient::new(config);

        client.connect().await.unwrap();
        client.subscribe("test/topic").await.unwrap();

        let subs = client.subscriptions().await;
        assert!(subs.contains(&"test/topic".to_string()));
    }

    #[tokio::test]
    async fn test_not_connected() {
        let config = MqttConfig::default();
        let (client, _rx) = MqttClient::new(config);

        let result = client.subscribe("test/topic").await;
        assert!(matches!(result, Err(MqttError::NotConnected)));
    }

    #[tokio::test]
    async fn test_topics() {
        assert_eq!(
            topics::status_list_update("PRY_01"),
            "sahi/PRY_01/status-list/update"
        );
        assert_eq!(
            topics::config_update("VRF_01"),
            "sahi/verifier/VRF_01/config"
        );
        assert_eq!(
            topics::heartbeat("VRF_01"),
            "sahi/verifier/VRF_01/heartbeat"
        );
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let config = MqttConfig::default();
        let (client, _rx) = MqttClient::new(config);

        client.connect().await.unwrap();
        client.send_heartbeat("VRF_01HXK").await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_verifier_topics() {
        let config = MqttConfig::default();
        let (client, _rx) = MqttClient::new(config);

        client.connect().await.unwrap();
        client
            .subscribe_verifier_topics("VRF_01HXK", "PRY_01HXK")
            .await
            .unwrap();

        let subs = client.subscriptions().await;
        assert_eq!(subs.len(), 4);
    }
}
