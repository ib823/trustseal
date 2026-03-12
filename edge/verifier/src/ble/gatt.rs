//! GATT server for VaultPass BLE service.
//!
//! Characteristics:
//! - Challenge (read): Returns current challenge
//! - Presentation (write): Receives SD-JWT presentation
//! - Result (notify): Sends access decision

use std::sync::Arc;

use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

use super::protocol::{BleProtocol, PresentationRequest, PresentationResponse};

/// GATT characteristic UUIDs.
pub mod uuids {
    use uuid::Uuid;

    /// Challenge characteristic (read).
    pub const CHALLENGE: Uuid = Uuid::from_u128(0x53414849_0002_1000_8000_00805F9B34FB);

    /// Presentation characteristic (write).
    pub const PRESENTATION: Uuid = Uuid::from_u128(0x53414849_0003_1000_8000_00805F9B34FB);

    /// Result characteristic (notify).
    pub const RESULT: Uuid = Uuid::from_u128(0x53414849_0004_1000_8000_00805F9B34FB);
}

/// GATT server errors.
#[derive(Debug, Error)]
pub enum GattError {
    #[error("Service registration failed: {0}")]
    RegistrationFailed(String),

    #[error("Characteristic error: {0}")]
    CharacteristicError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] super::protocol::ProtocolError),

    #[error("Channel closed")]
    ChannelClosed,
}

/// Incoming presentation event.
#[derive(Debug)]
pub struct PresentationEvent {
    /// The presentation request.
    pub request: PresentationRequest,

    /// Response channel.
    pub response_tx: mpsc::Sender<PresentationResponse>,
}

/// GATT server for BLE credential verification.
pub struct GattServer {
    /// Protocol handler.
    protocol: Arc<RwLock<BleProtocol>>,

    /// Presentation event sender.
    event_tx: mpsc::Sender<PresentationEvent>,

    /// Whether server is running.
    is_running: Arc<RwLock<bool>>,
}

impl GattServer {
    /// Create a new GATT server.
    pub fn new(site_id: &str) -> (Self, mpsc::Receiver<PresentationEvent>) {
        let (event_tx, event_rx) = mpsc::channel(16);

        let server = Self {
            protocol: Arc::new(RwLock::new(BleProtocol::new(site_id))),
            event_tx,
            is_running: Arc::new(RwLock::new(false)),
        };

        (server, event_rx)
    }

    /// Start the GATT server.
    pub async fn start(&self) -> Result<(), GattError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            debug!("GATT server already running");
            return Ok(());
        }

        info!("Starting GATT server");

        // In production, this would register GATT services with BlueZ
        // For now, stub implementation

        #[cfg(feature = "simulation")]
        {
            info!("Simulation mode: GATT server simulated");
            *is_running = true;
            return Ok(());
        }

        #[cfg(not(feature = "simulation"))]
        {
            warn!("GATT server not yet implemented - using stub");
            *is_running = true;
            Ok(())
        }
    }

    /// Stop the GATT server.
    pub async fn stop(&self) -> Result<(), GattError> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping GATT server");
        *is_running = false;
        Ok(())
    }

    /// Handle a challenge read request.
    pub async fn handle_challenge_read(&self) -> Result<Vec<u8>, GattError> {
        let mut protocol = self.protocol.write().await;
        let challenge = protocol.generate_challenge();

        debug!("Generated challenge for BLE client");
        protocol.encode_challenge(&challenge).map_err(Into::into)
    }

    /// Handle a presentation write.
    pub async fn handle_presentation_write(&self, data: &[u8]) -> Result<Vec<u8>, GattError> {
        let protocol = self.protocol.read().await;

        // Decode the presentation
        let request = protocol.decode_presentation(data)?;
        debug!("Received presentation from BLE client");

        // Validate challenge
        protocol.validate_challenge(&request)?;

        // Create response channel
        let (response_tx, mut response_rx) = mpsc::channel(1);

        // Send event for processing
        self.event_tx
            .send(PresentationEvent {
                request,
                response_tx,
            })
            .await
            .map_err(|_| GattError::ChannelClosed)?;

        // Wait for response
        let response = response_rx.recv().await.ok_or(GattError::ChannelClosed)?;

        // Encode response before releasing read lock
        let encoded = protocol.encode_response(&response)?;

        // Clear challenge after use
        drop(protocol);
        self.protocol.write().await.clear_challenge();

        Ok(encoded)
    }

    /// Check if server is running.
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Get characteristic UUIDs.
    pub fn characteristic_uuids() -> CharacteristicUuids {
        CharacteristicUuids {
            challenge: uuids::CHALLENGE,
            presentation: uuids::PRESENTATION,
            result: uuids::RESULT,
        }
    }
}

/// Characteristic UUID set.
#[derive(Debug, Clone)]
pub struct CharacteristicUuids {
    /// Challenge characteristic.
    pub challenge: uuid::Uuid,

    /// Presentation characteristic.
    pub presentation: uuid::Uuid,

    /// Result characteristic.
    pub result: uuid::Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gatt_server_creation() {
        let (server, _rx) = GattServer::new("VRF_01HXK");
        assert!(!server.is_running().await);
    }

    #[tokio::test]
    async fn test_challenge_read() {
        let (server, _rx) = GattServer::new("VRF_01HXK");
        server.start().await.unwrap();

        let challenge_data = server.handle_challenge_read().await.unwrap();
        assert!(!challenge_data.is_empty());

        // Should be valid JSON
        let challenge: super::super::protocol::Challenge =
            serde_json::from_slice(&challenge_data).unwrap();
        assert_eq!(challenge.site_id, "VRF_01HXK");
    }

    #[tokio::test]
    async fn test_characteristic_uuids() {
        let uuids = GattServer::characteristic_uuids();
        assert_ne!(uuids.challenge, uuids.presentation);
        assert_ne!(uuids.presentation, uuids.result);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let (server, _rx) = GattServer::new("VRF_01HXK");

        server.start().await.unwrap();
        assert!(server.is_running().await);

        server.stop().await.unwrap();
        assert!(!server.is_running().await);
    }
}
