use chrono::{DateTime, Utc};
use serde::Serialize;
use std::net::IpAddr;

use super::types::{KeyAlgorithm, KeyHandle};

/// KMS operations that emit audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum KmsOperation {
    GenerateKey,
    Sign,
    Verify,
    ExportPublicKey,
    RotateKey,
    DestroyKey,
    ListKeys,
}

impl std::fmt::Display for KmsOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenerateKey => write!(f, "generate_key"),
            Self::Sign => write!(f, "sign"),
            Self::Verify => write!(f, "verify"),
            Self::ExportPublicKey => write!(f, "export_public_key"),
            Self::RotateKey => write!(f, "rotate_key"),
            Self::DestroyKey => write!(f, "destroy_key"),
            Self::ListKeys => write!(f, "list_keys"),
        }
    }
}

/// Audit event emitted by every KMS operation.
/// Designed for Merkle log ingestion (F2).
#[derive(Debug, Clone, Serialize)]
pub struct KmsAuditEvent {
    /// ULID with EVT_ prefix
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub operation: KmsOperation,
    pub key_handle: Option<KeyHandle>,
    pub algorithm: Option<KeyAlgorithm>,
    pub tenant_id: Option<String>,
    pub actor_id: String,
    pub success: bool,
    pub error_code: Option<String>,
    /// Encrypted at rest — never logged in plaintext
    pub ip_address: Option<IpAddr>,
}

impl KmsAuditEvent {
    /// Create a successful audit event.
    pub fn success(
        operation: KmsOperation,
        key_handle: Option<&KeyHandle>,
        actor_id: &str,
        tenant_id: Option<&str>,
    ) -> Self {
        Self {
            event_id: format!("EVT_{}", ulid::Ulid::new()),
            timestamp: Utc::now(),
            operation,
            key_handle: key_handle.cloned(),
            algorithm: None,
            tenant_id: tenant_id.map(String::from),
            actor_id: actor_id.to_string(),
            success: true,
            error_code: None,
            ip_address: None,
        }
    }

    /// Create a failure audit event.
    pub fn failure(
        operation: KmsOperation,
        key_handle: Option<&KeyHandle>,
        actor_id: &str,
        tenant_id: Option<&str>,
        error_code: &str,
    ) -> Self {
        Self {
            event_id: format!("EVT_{}", ulid::Ulid::new()),
            timestamp: Utc::now(),
            operation,
            key_handle: key_handle.cloned(),
            algorithm: None,
            tenant_id: tenant_id.map(String::from),
            actor_id: actor_id.to_string(),
            success: false,
            error_code: Some(error_code.to_string()),
            ip_address: None,
        }
    }

    #[must_use]
    pub fn with_algorithm(mut self, algorithm: KeyAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    #[must_use]
    pub fn with_ip(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }
}
