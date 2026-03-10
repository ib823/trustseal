//! NFC reader using PC/SC interface.

use std::sync::Arc;

use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use super::apdu::{ApduCommand, ApduResponse, NdefParser, NdefRecord};

/// NFC reader errors.
#[derive(Debug, Error)]
pub enum NfcError {
    #[error("No reader found")]
    NoReader,

    #[error("Card not present")]
    NoCard,

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("APDU error: {0}")]
    ApduError(#[from] super::apdu::ApduError),

    #[error("PC/SC error: {0}")]
    PcscError(String),
}

/// NFC card event.
#[derive(Debug, Clone)]
pub enum NfcEvent {
    /// Card inserted.
    CardInserted { atr: Vec<u8> },

    /// Card removed.
    CardRemoved,

    /// NDEF record read.
    NdefRead { records: Vec<NdefRecord> },

    /// VaultPass credential detected.
    CredentialDetected { sd_jwt: String },

    /// Error occurred.
    Error { message: String },
}

/// NFC reader interface.
pub struct NfcReader {
    /// Reader name.
    reader_name: Option<String>,

    /// Event sender.
    event_tx: mpsc::Sender<NfcEvent>,

    /// Running state.
    is_running: Arc<RwLock<bool>>,

    /// Polling interval in milliseconds.
    poll_interval_ms: u64,
}

impl NfcReader {
    /// Create a new NFC reader.
    pub fn new() -> (Self, mpsc::Receiver<NfcEvent>) {
        let (event_tx, event_rx) = mpsc::channel(32);

        let reader = Self {
            reader_name: None,
            event_tx,
            is_running: Arc::new(RwLock::new(false)),
            poll_interval_ms: 250,
        };

        (reader, event_rx)
    }

    /// Set the reader name.
    pub fn with_reader(mut self, name: &str) -> Self {
        self.reader_name = Some(name.to_string());
        self
    }

    /// Set polling interval.
    pub fn with_poll_interval(mut self, ms: u64) -> Self {
        self.poll_interval_ms = ms;
        self
    }

    /// Start the NFC reader.
    pub async fn start(&self) -> Result<(), NfcError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }

        info!("Starting NFC reader");

        // In production, this would:
        // 1. Establish PC/SC context
        // 2. Find readers
        // 3. Start polling for cards

        #[cfg(feature = "simulation")]
        {
            info!("Simulation mode: NFC reader simulated");
            *is_running = true;
            return Ok(());
        }

        #[cfg(not(feature = "simulation"))]
        {
            warn!("NFC reader not yet implemented - using stub");
            *is_running = true;
            Ok(())
        }
    }

    /// Stop the NFC reader.
    pub async fn stop(&self) -> Result<(), NfcError> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }

        info!("Stopping NFC reader");
        *is_running = false;
        Ok(())
    }

    /// Check if reader is running.
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Read NDEF from current card (stub).
    pub async fn read_ndef(&self) -> Result<Vec<NdefRecord>, NfcError> {
        // In production, this would:
        // 1. Select NDEF application
        // 2. Read capability container
        // 3. Select NDEF file
        // 4. Read NDEF data
        // 5. Parse NDEF records

        warn!("NDEF read not implemented - returning empty");
        Ok(Vec::new())
    }

    /// Transmit APDU command (stub).
    pub async fn transmit(&self, command: &ApduCommand) -> Result<ApduResponse, NfcError> {
        debug!("Transmitting APDU: {:02X?}", command.to_bytes());

        // Stub: return success with no data
        Ok(ApduResponse {
            data: Vec::new(),
            sw1: 0x90,
            sw2: 0x00,
        })
    }

    /// Process a detected card.
    pub async fn process_card(&self, atr: &[u8]) -> Result<Option<String>, NfcError> {
        info!("Processing NFC card, ATR: {:02X?}", atr);

        // Send card inserted event
        let _ = self
            .event_tx
            .send(NfcEvent::CardInserted { atr: atr.to_vec() })
            .await;

        // Try to read NDEF
        let records = self.read_ndef().await?;

        if records.is_empty() {
            debug!("No NDEF records found");
            return Ok(None);
        }

        let _ = self
            .event_tx
            .send(NfcEvent::NdefRead {
                records: records.clone(),
            })
            .await;

        // Look for VaultPass record
        for record in records {
            if NdefParser::is_vaultpass_record(&record) {
                let sd_jwt = NdefParser::extract_sdjwt(&record)?;
                let _ = self
                    .event_tx
                    .send(NfcEvent::CredentialDetected {
                        sd_jwt: sd_jwt.clone(),
                    })
                    .await;
                return Ok(Some(sd_jwt));
            }
        }

        debug!("No VaultPass credential found in NDEF records");
        Ok(None)
    }

    /// Run polling loop (for standalone use).
    pub async fn run_polling(&self) {
        info!(
            "Starting NFC polling loop (interval: {}ms)",
            self.poll_interval_ms
        );

        let is_running = self.is_running.clone();
        let poll_interval = tokio::time::Duration::from_millis(self.poll_interval_ms);

        while *is_running.read().await {
            // In production, this would check for card presence
            // and trigger process_card when detected

            tokio::time::sleep(poll_interval).await;
        }

        info!("NFC polling loop stopped");
    }
}

impl Default for NfcReader {
    fn default() -> Self {
        Self::new().0
    }
}

/// Simulated card for testing.
#[cfg(any(test, feature = "simulation"))]
pub struct SimulatedCard {
    /// ATR bytes.
    pub atr: Vec<u8>,

    /// NDEF records.
    pub records: Vec<NdefRecord>,
}

#[cfg(any(test, feature = "simulation"))]
impl SimulatedCard {
    /// Create a VaultPass card with an SD-JWT.
    pub fn vaultpass(sd_jwt: &str) -> Self {
        use super::apdu::VAULTPASS_NDEF_TYPE;

        Self {
            atr: vec![0x3B, 0x8F, 0x80, 0x01], // Sample ATR
            records: vec![NdefRecord {
                tnf: 0x04,
                record_type: VAULTPASS_NDEF_TYPE.to_vec(),
                id: Vec::new(),
                payload: sd_jwt.as_bytes().to_vec(),
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reader_creation() {
        let (reader, _rx) = NfcReader::new();
        assert!(!reader.is_running().await);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let (reader, _rx) = NfcReader::new();

        reader.start().await.unwrap();
        assert!(reader.is_running().await);

        reader.stop().await.unwrap();
        assert!(!reader.is_running().await);
    }

    #[tokio::test]
    async fn test_simulated_card() {
        let card = SimulatedCard::vaultpass("test.sdjwt");
        assert!(!card.atr.is_empty());
        assert_eq!(card.records.len(), 1);
        assert!(NdefParser::is_vaultpass_record(&card.records[0]));
    }

    #[tokio::test]
    async fn test_transmit_stub() {
        let (reader, _rx) = NfcReader::new();
        reader.start().await.unwrap();

        let cmd = ApduCommand::select_ndef_application();
        let response = reader.transmit(&cmd).await.unwrap();

        assert!(response.is_success());
    }
}
