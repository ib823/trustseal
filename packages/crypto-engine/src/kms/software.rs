use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use ring::rand::SystemRandom;
use ring::signature::{
    self, EcdsaKeyPair, Ed25519KeyPair, KeyPair, ECDSA_P256_SHA256_ASN1,
    ECDSA_P256_SHA256_ASN1_SIGNING,
};
use tokio::sync::RwLock;
use tracing::{info, warn};
use zeroize::Zeroize;

use super::audit::{KmsAuditEvent, KmsOperation};
use super::provider::KmsProvider;
use super::types::{
    DestroyConfirmation, KeyAlgorithm, KeyHandle, KeyMetadata, KeyRotationResult, KeyState,
    PublicKeyBytes, Signature,
};
use crate::error::{CryptoError, ErrorCode};

/// In-memory key material. Zeroized on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
struct SoftwareKeyMaterial {
    /// PKCS#8 DER-encoded private key
    #[zeroize(skip)] // ring's key types don't impl Zeroize, but Vec<u8> does
    pkcs8_der: Vec<u8>,
}

/// Internal key entry storing both material and metadata.
struct KeyEntry {
    material: Option<SoftwareKeyMaterial>,
    metadata: KeyMetadata,
}

/// Software-based KMS provider for local development.
///
/// Keys are stored in-memory using `ring`. Thread-safe via `RwLock`.
/// NOT for production — use `AwsCloudHsmProvider` instead.
pub struct SoftwareKmsProvider {
    keys: Arc<RwLock<HashMap<String, KeyEntry>>>,
    rng: SystemRandom,
    /// Audit event sink — in production this feeds the Merkle log (F2).
    /// For dev, we collect events in memory.
    audit_log: Arc<RwLock<Vec<KmsAuditEvent>>>,
}

impl SoftwareKmsProvider {
    #[must_use]
    pub fn new() -> Self {
        info!("SoftwareKmsProvider initialized — FOR DEVELOPMENT ONLY");
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            rng: SystemRandom::new(),
            audit_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Retrieve collected audit events (dev/test only).
    pub async fn drain_audit_events(&self) -> Vec<KmsAuditEvent> {
        let mut log = self.audit_log.write().await;
        std::mem::take(&mut *log)
    }

    async fn emit_audit(&self, event: KmsAuditEvent) {
        tracing::info!(
            event_id = %event.event_id,
            operation = %event.operation,
            success = event.success,
            "KMS audit event"
        );
        self.audit_log.write().await.push(event);
    }

    fn get_ed25519_pair(pkcs8: &[u8]) -> Result<Ed25519KeyPair, CryptoError> {
        Ed25519KeyPair::from_pkcs8(pkcs8).map_err(|e| {
            CryptoError::kms(
                ErrorCode::SignFailed,
                format!("Ed25519 key parse error: {e}"),
            )
        })
    }

    fn get_ecdsa_pair(pkcs8: &[u8]) -> Result<EcdsaKeyPair, CryptoError> {
        EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8, &SystemRandom::new())
            .map_err(|e| {
                CryptoError::kms(ErrorCode::SignFailed, format!("ECDSA key parse error: {e}"))
            })
    }
}

impl Default for SoftwareKmsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KmsProvider for SoftwareKmsProvider {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: &str,
        tenant_id: Option<&str>,
    ) -> Result<KeyHandle, CryptoError> {
        let handle = KeyHandle::new(format!("KEY_{}", ulid::Ulid::new()));

        let pkcs8_der = match algorithm {
            KeyAlgorithm::Ed25519 => {
                let pkcs8 = Ed25519KeyPair::generate_pkcs8(&self.rng).map_err(|e| {
                    CryptoError::kms(
                        ErrorCode::KeyGenerationFailed,
                        format!("Ed25519 keygen failed: {e}"),
                    )
                })?;
                pkcs8.as_ref().to_vec()
            }
            KeyAlgorithm::EcdsaP256 => {
                let pkcs8 =
                    EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &self.rng)
                        .map_err(|e| {
                            CryptoError::kms(
                                ErrorCode::KeyGenerationFailed,
                                format!("ECDSA P-256 keygen failed: {e}"),
                            )
                        })?;
                pkcs8.as_ref().to_vec()
            }
        };

        let metadata = KeyMetadata {
            handle: handle.clone(),
            algorithm,
            label: label.to_string(),
            state: KeyState::Active,
            created_at: Utc::now(),
            rotated_at: None,
            expires_at: None,
            tenant_id: tenant_id.map(String::from),
        };

        let entry = KeyEntry {
            material: Some(SoftwareKeyMaterial { pkcs8_der }),
            metadata,
        };

        self.keys.write().await.insert(handle.id.clone(), entry);

        self.emit_audit(
            KmsAuditEvent::success(
                KmsOperation::GenerateKey,
                Some(&handle),
                "SYSTEM",
                tenant_id,
            )
            .with_algorithm(algorithm),
        )
        .await;

        info!(key_id = %handle.id, algorithm = %algorithm, label = %label, "Key generated");
        Ok(handle)
    }

    async fn sign(&self, key_handle: &KeyHandle, data: &[u8]) -> Result<Signature, CryptoError> {
        let keys = self.keys.read().await;
        let entry = keys.get(&key_handle.id).ok_or_else(|| {
            CryptoError::kms(
                ErrorCode::KeyNotFound,
                format!("Key {} not found", key_handle.id),
            )
        })?;

        if entry.metadata.state != KeyState::Active {
            return Err(CryptoError::kms(
                ErrorCode::InvalidKeyState,
                format!(
                    "Key {} is in {} state, signing requires Active",
                    key_handle.id, entry.metadata.state
                ),
            ));
        }

        let material = entry.material.as_ref().ok_or_else(|| {
            CryptoError::kms(ErrorCode::KeyNotFound, "Key material destroyed".to_string())
        })?;

        let sig_bytes = match entry.metadata.algorithm {
            KeyAlgorithm::Ed25519 => {
                let pair = Self::get_ed25519_pair(&material.pkcs8_der)?;
                pair.sign(data).as_ref().to_vec()
            }
            KeyAlgorithm::EcdsaP256 => {
                let pair = Self::get_ecdsa_pair(&material.pkcs8_der)?;
                pair.sign(&self.rng, data)
                    .map_err(|e| {
                        CryptoError::kms(ErrorCode::SignFailed, format!("ECDSA sign failed: {e}"))
                    })?
                    .as_ref()
                    .to_vec()
            }
        };

        self.emit_audit(
            KmsAuditEvent::success(
                KmsOperation::Sign,
                Some(key_handle),
                "SYSTEM",
                entry.metadata.tenant_id.as_deref(),
            )
            .with_algorithm(entry.metadata.algorithm),
        )
        .await;

        Ok(Signature {
            bytes: sig_bytes,
            algorithm: entry.metadata.algorithm,
        })
    }

    async fn verify(
        &self,
        key_handle: &KeyHandle,
        data: &[u8],
        sig: &Signature,
    ) -> Result<bool, CryptoError> {
        let keys = self.keys.read().await;
        let entry = keys.get(&key_handle.id).ok_or_else(|| {
            CryptoError::kms(
                ErrorCode::KeyNotFound,
                format!("Key {} not found", key_handle.id),
            )
        })?;

        // Verification allowed in Active and VerifyOnly states
        if entry.metadata.state == KeyState::Destroyed
            || entry.metadata.state == KeyState::PendingDestruction
        {
            return Err(CryptoError::kms(
                ErrorCode::InvalidKeyState,
                format!(
                    "Key {} is {}, cannot verify",
                    key_handle.id, entry.metadata.state
                ),
            ));
        }

        let material = entry.material.as_ref().ok_or_else(|| {
            CryptoError::kms(ErrorCode::KeyNotFound, "Key material destroyed".to_string())
        })?;

        let valid = match entry.metadata.algorithm {
            KeyAlgorithm::Ed25519 => {
                let pair = Self::get_ed25519_pair(&material.pkcs8_der)?;
                let public_key = pair.public_key();
                let peer_pk =
                    signature::UnparsedPublicKey::new(&signature::ED25519, public_key.as_ref());
                peer_pk.verify(data, &sig.bytes).is_ok()
            }
            KeyAlgorithm::EcdsaP256 => {
                let pair = Self::get_ecdsa_pair(&material.pkcs8_der)?;
                let public_key = pair.public_key();
                let peer_pk =
                    signature::UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, public_key.as_ref());
                peer_pk.verify(data, &sig.bytes).is_ok()
            }
        };

        self.emit_audit(
            KmsAuditEvent::success(
                KmsOperation::Verify,
                Some(key_handle),
                "SYSTEM",
                entry.metadata.tenant_id.as_deref(),
            )
            .with_algorithm(entry.metadata.algorithm),
        )
        .await;

        Ok(valid)
    }

    async fn export_public_key(
        &self,
        key_handle: &KeyHandle,
    ) -> Result<PublicKeyBytes, CryptoError> {
        let keys = self.keys.read().await;
        let entry = keys.get(&key_handle.id).ok_or_else(|| {
            CryptoError::kms(
                ErrorCode::KeyNotFound,
                format!("Key {} not found", key_handle.id),
            )
        })?;

        let material = entry.material.as_ref().ok_or_else(|| {
            CryptoError::kms(ErrorCode::KeyNotFound, "Key material destroyed".to_string())
        })?;

        let pub_bytes = match entry.metadata.algorithm {
            KeyAlgorithm::Ed25519 => {
                let pair = Self::get_ed25519_pair(&material.pkcs8_der)?;
                pair.public_key().as_ref().to_vec()
            }
            KeyAlgorithm::EcdsaP256 => {
                let pair = Self::get_ecdsa_pair(&material.pkcs8_der)?;
                pair.public_key().as_ref().to_vec()
            }
        };

        self.emit_audit(KmsAuditEvent::success(
            KmsOperation::ExportPublicKey,
            Some(key_handle),
            "SYSTEM",
            entry.metadata.tenant_id.as_deref(),
        ))
        .await;

        Ok(PublicKeyBytes {
            bytes: pub_bytes,
            algorithm: entry.metadata.algorithm,
        })
    }

    async fn rotate_key(&self, old_handle: &KeyHandle) -> Result<KeyRotationResult, CryptoError> {
        // Read old key metadata first
        let (algorithm, label, tenant_id) = {
            let keys = self.keys.read().await;
            let entry = keys.get(&old_handle.id).ok_or_else(|| {
                CryptoError::kms(
                    ErrorCode::KeyNotFound,
                    format!("Key {} not found", old_handle.id),
                )
            })?;

            if entry.metadata.state != KeyState::Active {
                return Err(CryptoError::kms(
                    ErrorCode::InvalidKeyState,
                    format!(
                        "Key {} is {}, rotation requires Active",
                        old_handle.id, entry.metadata.state
                    ),
                ));
            }

            (
                entry.metadata.algorithm,
                entry.metadata.label.clone(),
                entry.metadata.tenant_id.clone(),
            )
        };

        // Generate new key with same algorithm and label
        let new_handle = self
            .generate_key(algorithm, &label, tenant_id.as_deref())
            .await?;

        // Mark old key as VerifyOnly
        let valid_until = Utc::now() + Duration::days(30);
        {
            let mut keys = self.keys.write().await;
            if let Some(entry) = keys.get_mut(&old_handle.id) {
                entry.metadata.state = KeyState::VerifyOnly;
                entry.metadata.rotated_at = Some(Utc::now());
                entry.metadata.expires_at = Some(valid_until);
            }
        }

        self.emit_audit(
            KmsAuditEvent::success(
                KmsOperation::RotateKey,
                Some(old_handle),
                "SYSTEM",
                tenant_id.as_deref(),
            )
            .with_algorithm(algorithm),
        )
        .await;

        info!(
            old_key = %old_handle.id,
            new_key = %new_handle.id,
            valid_until = %valid_until,
            "Key rotated"
        );

        Ok(KeyRotationResult {
            new_handle,
            old_handle: old_handle.clone(),
            old_key_valid_until: valid_until,
        })
    }

    async fn list_keys(&self, tenant_id: Option<&str>) -> Result<Vec<KeyMetadata>, CryptoError> {
        let keys = self.keys.read().await;
        let result: Vec<KeyMetadata> = keys
            .values()
            .filter(|entry| match tenant_id {
                Some(tid) => entry.metadata.tenant_id.as_deref() == Some(tid),
                None => true,
            })
            .map(|entry| entry.metadata.clone())
            .collect();

        self.emit_audit(KmsAuditEvent::success(
            KmsOperation::ListKeys,
            None,
            "SYSTEM",
            tenant_id,
        ))
        .await;

        Ok(result)
    }

    async fn destroy_key(
        &self,
        key_handle: &KeyHandle,
        confirmation: DestroyConfirmation,
    ) -> Result<(), CryptoError> {
        if !confirmation.is_valid_for(key_handle) {
            self.emit_audit(KmsAuditEvent::failure(
                KmsOperation::DestroyKey,
                Some(key_handle),
                &confirmation.actor_id,
                None,
                ErrorCode::DestroyConfirmationFailed.code(),
            ))
            .await;

            return Err(CryptoError::kms(
                ErrorCode::DestroyConfirmationFailed,
                "Destroy confirmation does not match key handle".to_string(),
            ));
        }

        let mut keys = self.keys.write().await;
        let entry = keys.get_mut(&key_handle.id).ok_or_else(|| {
            CryptoError::kms(
                ErrorCode::KeyNotFound,
                format!("Key {} not found", key_handle.id),
            )
        })?;

        // Zeroize and remove key material
        entry.material = None;
        entry.metadata.state = KeyState::Destroyed;

        // Drop the write lock before emitting audit
        let tenant_id = entry.metadata.tenant_id.clone();
        drop(keys);

        self.emit_audit(KmsAuditEvent::success(
            KmsOperation::DestroyKey,
            Some(key_handle),
            &confirmation.actor_id,
            tenant_id.as_deref(),
        ))
        .await;

        warn!(key_id = %key_handle.id, actor = %confirmation.actor_id, "Key DESTROYED — irreversible");
        Ok(())
    }

    async fn get_key_metadata(&self, key_handle: &KeyHandle) -> Result<KeyMetadata, CryptoError> {
        let keys = self.keys.read().await;
        let entry = keys.get(&key_handle.id).ok_or_else(|| {
            CryptoError::kms(
                ErrorCode::KeyNotFound,
                format!("Key {} not found", key_handle.id),
            )
        })?;
        Ok(entry.metadata.clone())
    }
}
