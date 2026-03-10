//! VaultPass BLE protocol implementation.
//!
//! Protocol flow:
//! 1. Wallet discovers verifier via service UUID
//! 2. Wallet connects and reads challenge characteristic
//! 3. Verifier generates a fresh challenge
//! 4. Wallet writes presentation (SD-JWT + key binding) to presentation characteristic
//! 5. Verifier verifies and writes result to result characteristic
//! 6. Wallet reads result and disconnects

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Protocol errors.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),

    #[error("Challenge expired")]
    ChallengeExpired,

    #[error("Invalid challenge")]
    InvalidChallenge,

    #[error("Presentation too large: {size} > {max}")]
    PresentationTooLarge { size: usize, max: usize },

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Maximum presentation size (64KB).
pub const MAX_PRESENTATION_SIZE: usize = 65536;

/// Challenge validity duration in seconds.
pub const CHALLENGE_VALIDITY_SECS: u64 = 30;

/// BLE protocol handler.
pub struct BleProtocol {
    /// Current challenge.
    current_challenge: Option<Challenge>,

    /// Verifier site ID.
    site_id: String,
}

/// Challenge issued to wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// Random challenge bytes (32 bytes, base64url encoded).
    pub nonce: String,

    /// Timestamp when challenge was issued.
    pub issued_at: u64,

    /// Verifier site ID.
    pub site_id: String,
}

/// Presentation request from wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationRequest {
    /// SD-JWT compact serialization.
    pub sd_jwt: String,

    /// Challenge nonce being responded to.
    pub challenge_nonce: String,

    /// Key binding JWT (proves holder controls the key).
    pub key_binding_jwt: Option<String>,

    /// Requested zone/floor (optional).
    pub requested_zone: Option<String>,
}

/// Presentation response to wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationResponse {
    /// Access result.
    pub result: AccessResult,

    /// Optional message.
    pub message: Option<String>,

    /// Audit log ID.
    pub audit_id: Option<String>,
}

/// Access result codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessResult {
    /// Access granted.
    Granted,

    /// Access denied.
    Denied,

    /// Credential revoked.
    Revoked,

    /// Credential expired.
    Expired,

    /// Invalid presentation.
    Invalid,

    /// System error.
    Error,
}

impl BleProtocol {
    /// Create a new protocol handler.
    pub fn new(site_id: &str) -> Self {
        Self {
            current_challenge: None,
            site_id: site_id.to_string(),
        }
    }

    /// Generate a new challenge.
    pub fn generate_challenge(&mut self) -> Challenge {
        let mut nonce_bytes = [0u8; 32];
        getrandom::getrandom(&mut nonce_bytes).expect("Failed to generate random bytes");

        let challenge = Challenge {
            nonce: URL_SAFE_NO_PAD.encode(nonce_bytes),
            issued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs(),
            site_id: self.site_id.clone(),
        };

        self.current_challenge = Some(challenge.clone());
        challenge
    }

    /// Encode challenge for BLE transmission.
    pub fn encode_challenge(&self, challenge: &Challenge) -> Result<Vec<u8>, ProtocolError> {
        serde_json::to_vec(challenge)
            .map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }

    /// Decode presentation request from BLE.
    pub fn decode_presentation(&self, data: &[u8]) -> Result<PresentationRequest, ProtocolError> {
        if data.len() > MAX_PRESENTATION_SIZE {
            return Err(ProtocolError::PresentationTooLarge {
                size: data.len(),
                max: MAX_PRESENTATION_SIZE,
            });
        }

        serde_json::from_slice(data)
            .map_err(|e| ProtocolError::InvalidFormat(e.to_string()))
    }

    /// Validate that the presentation challenge matches.
    pub fn validate_challenge(&self, request: &PresentationRequest) -> Result<(), ProtocolError> {
        let challenge = self
            .current_challenge
            .as_ref()
            .ok_or(ProtocolError::InvalidChallenge)?;

        // Check nonce matches
        if request.challenge_nonce != challenge.nonce {
            return Err(ProtocolError::InvalidChallenge);
        }

        // Check not expired
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        if now - challenge.issued_at > CHALLENGE_VALIDITY_SECS {
            return Err(ProtocolError::ChallengeExpired);
        }

        Ok(())
    }

    /// Encode response for BLE transmission.
    pub fn encode_response(
        &self,
        response: &PresentationResponse,
    ) -> Result<Vec<u8>, ProtocolError> {
        serde_json::to_vec(response)
            .map_err(|e| ProtocolError::SerializationError(e.to_string()))
    }

    /// Clear the current challenge (after use or timeout).
    pub fn clear_challenge(&mut self) {
        self.current_challenge = None;
    }

    /// Get the current challenge if valid.
    pub fn current_challenge(&self) -> Option<&Challenge> {
        self.current_challenge.as_ref()
    }
}

impl PresentationResponse {
    /// Create a granted response.
    pub fn granted(audit_id: &str) -> Self {
        Self {
            result: AccessResult::Granted,
            message: None,
            audit_id: Some(audit_id.to_string()),
        }
    }

    /// Create a denied response.
    pub fn denied(message: &str, audit_id: &str) -> Self {
        Self {
            result: AccessResult::Denied,
            message: Some(message.to_string()),
            audit_id: Some(audit_id.to_string()),
        }
    }

    /// Create a revoked response.
    pub fn revoked(audit_id: &str) -> Self {
        Self {
            result: AccessResult::Revoked,
            message: Some("Credential has been revoked".to_string()),
            audit_id: Some(audit_id.to_string()),
        }
    }

    /// Create an expired response.
    pub fn expired(audit_id: &str) -> Self {
        Self {
            result: AccessResult::Expired,
            message: Some("Credential has expired".to_string()),
            audit_id: Some(audit_id.to_string()),
        }
    }

    /// Create an invalid response.
    pub fn invalid(reason: &str, audit_id: &str) -> Self {
        Self {
            result: AccessResult::Invalid,
            message: Some(reason.to_string()),
            audit_id: Some(audit_id.to_string()),
        }
    }

    /// Create an error response.
    pub fn error(message: &str) -> Self {
        Self {
            result: AccessResult::Error,
            message: Some(message.to_string()),
            audit_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_challenge() {
        let mut protocol = BleProtocol::new("VRF_01HXK");
        let challenge = protocol.generate_challenge();

        assert_eq!(challenge.site_id, "VRF_01HXK");
        assert!(!challenge.nonce.is_empty());
        assert!(challenge.issued_at > 0);
    }

    #[test]
    fn test_encode_decode_challenge() {
        let mut protocol = BleProtocol::new("VRF_01HXK");
        let challenge = protocol.generate_challenge();

        let encoded = protocol.encode_challenge(&challenge).unwrap();
        let decoded: Challenge = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(decoded.nonce, challenge.nonce);
        assert_eq!(decoded.site_id, challenge.site_id);
    }

    #[test]
    fn test_validate_challenge_success() {
        let mut protocol = BleProtocol::new("VRF_01HXK");
        let challenge = protocol.generate_challenge();

        let request = PresentationRequest {
            sd_jwt: "test.jwt".to_string(),
            challenge_nonce: challenge.nonce.clone(),
            key_binding_jwt: None,
            requested_zone: None,
        };

        assert!(protocol.validate_challenge(&request).is_ok());
    }

    #[test]
    fn test_validate_challenge_wrong_nonce() {
        let mut protocol = BleProtocol::new("VRF_01HXK");
        let _challenge = protocol.generate_challenge();

        let request = PresentationRequest {
            sd_jwt: "test.jwt".to_string(),
            challenge_nonce: "wrong-nonce".to_string(),
            key_binding_jwt: None,
            requested_zone: None,
        };

        assert!(matches!(
            protocol.validate_challenge(&request),
            Err(ProtocolError::InvalidChallenge)
        ));
    }

    #[test]
    fn test_presentation_too_large() {
        let protocol = BleProtocol::new("VRF_01HXK");
        let large_data = vec![0u8; MAX_PRESENTATION_SIZE + 1];

        assert!(matches!(
            protocol.decode_presentation(&large_data),
            Err(ProtocolError::PresentationTooLarge { .. })
        ));
    }

    #[test]
    fn test_response_constructors() {
        let granted = PresentationResponse::granted("AUD_123");
        assert_eq!(granted.result, AccessResult::Granted);

        let denied = PresentationResponse::denied("Policy violation", "AUD_456");
        assert_eq!(denied.result, AccessResult::Denied);
        assert!(denied.message.unwrap().contains("Policy"));

        let revoked = PresentationResponse::revoked("AUD_789");
        assert_eq!(revoked.result, AccessResult::Revoked);
    }
}
