//! BLE advertising for verifier discovery.

use std::sync::Arc;

use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// BLE advertising errors.
#[derive(Debug, Error)]
pub enum AdvertiseError {
    #[error("Bluetooth adapter not found")]
    AdapterNotFound,

    #[error("Bluetooth not enabled")]
    BluetoothDisabled,

    #[error("Advertising failed: {0}")]
    AdvertiseFailed(String),

    #[error("Bluer error: {0}")]
    BluerError(String),
}

/// VaultPass BLE service UUID.
pub const VAULTPASS_SERVICE_UUID: uuid::Uuid =
    uuid::Uuid::from_u128(0x53414849_0001_1000_8000_00805F9B34FB);

/// BLE advertiser for verifier device.
pub struct BleAdvertiser {
    /// Verifier site ID for advertising.
    site_id: String,

    /// Property ID.
    property_id: String,

    /// Advertising state.
    is_advertising: Arc<RwLock<bool>>,

    /// Adapter name.
    adapter_name: String,
}

impl BleAdvertiser {
    /// Create a new BLE advertiser.
    pub fn new(site_id: &str, property_id: &str) -> Self {
        Self {
            site_id: site_id.to_string(),
            property_id: property_id.to_string(),
            is_advertising: Arc::new(RwLock::new(false)),
            adapter_name: "hci0".to_string(),
        }
    }

    /// Set the Bluetooth adapter name.
    pub fn with_adapter(mut self, adapter: &str) -> Self {
        self.adapter_name = adapter.to_string();
        self
    }

    /// Start advertising.
    pub async fn start(&self) -> Result<(), AdvertiseError> {
        let mut is_advertising = self.is_advertising.write().await;
        if *is_advertising {
            debug!("Already advertising");
            return Ok(());
        }

        info!(
            "Starting BLE advertising for site {} on adapter {}",
            self.site_id, self.adapter_name
        );

        // In production, this would use bluer to:
        // 1. Get the adapter
        // 2. Set the local name to site_id
        // 3. Register the GATT service
        // 4. Start advertising with the service UUID

        // For now, stub implementation
        #[cfg(feature = "simulation")]
        {
            info!("Simulation mode: BLE advertising simulated");
            *is_advertising = true;
            return Ok(());
        }

        #[cfg(not(feature = "simulation"))]
        {
            // Real BlueZ implementation would go here
            // let session = bluer::Session::new().await
            //     .map_err(|e| AdvertiseError::BluerError(e.to_string()))?;
            // let adapter = session.adapter(&self.adapter_name).await
            //     .map_err(|_| AdvertiseError::AdapterNotFound)?;
            // ...

            warn!("BLE advertising not yet implemented - using stub");
            *is_advertising = true;
            Ok(())
        }
    }

    /// Stop advertising.
    pub async fn stop(&self) -> Result<(), AdvertiseError> {
        let mut is_advertising = self.is_advertising.write().await;
        if !*is_advertising {
            debug!("Not advertising");
            return Ok(());
        }

        info!("Stopping BLE advertising");
        *is_advertising = false;
        Ok(())
    }

    /// Check if advertising.
    pub async fn is_advertising(&self) -> bool {
        *self.is_advertising.read().await
    }

    /// Get the service UUID.
    pub fn service_uuid(&self) -> uuid::Uuid {
        VAULTPASS_SERVICE_UUID
    }

    /// Get advertising data for the verifier.
    pub fn advertising_data(&self) -> AdvertisingData {
        AdvertisingData {
            service_uuid: VAULTPASS_SERVICE_UUID,
            local_name: self.site_id.clone(),
            manufacturer_data: self.encode_manufacturer_data(),
        }
    }

    /// Encode manufacturer data with property info.
    fn encode_manufacturer_data(&self) -> Vec<u8> {
        // Custom manufacturer data format:
        // [0-1]: Company ID (0xFFFF for development)
        // [2]: Protocol version (0x01)
        // [3-10]: Property ID prefix (8 chars)
        let mut data = vec![0xFF, 0xFF, 0x01];
        let property_bytes = self.property_id.as_bytes();
        let len = std::cmp::min(8, property_bytes.len());
        data.extend_from_slice(&property_bytes[..len]);
        // Pad to 8 bytes
        data.resize(11, 0);
        data
    }
}

/// Advertising data structure.
#[derive(Debug, Clone)]
pub struct AdvertisingData {
    /// Service UUID.
    pub service_uuid: uuid::Uuid,

    /// Local name (site ID).
    pub local_name: String,

    /// Manufacturer-specific data.
    pub manufacturer_data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_advertiser_creation() {
        let advertiser = BleAdvertiser::new("VRF_01HXK", "PRY_01HXK");
        assert!(!advertiser.is_advertising().await);
        assert_eq!(advertiser.service_uuid(), VAULTPASS_SERVICE_UUID);
    }

    #[tokio::test]
    async fn test_advertising_data() {
        let advertiser = BleAdvertiser::new("VRF_01HXK", "PRY_01HXK");
        let data = advertiser.advertising_data();

        assert_eq!(data.local_name, "VRF_01HXK");
        assert_eq!(data.service_uuid, VAULTPASS_SERVICE_UUID);
        // Manufacturer data: 2 bytes company + 1 byte version + 8 bytes property
        assert_eq!(data.manufacturer_data.len(), 11);
        assert_eq!(data.manufacturer_data[2], 0x01); // Version
    }

    #[tokio::test]
    async fn test_start_stop_advertising() {
        let advertiser = BleAdvertiser::new("VRF_01HXK", "PRY_01HXK");

        advertiser.start().await.unwrap();
        assert!(advertiser.is_advertising().await);

        advertiser.stop().await.unwrap();
        assert!(!advertiser.is_advertising().await);
    }
}
