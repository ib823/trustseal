use async_trait::async_trait;

use crate::error::CryptoError;
use super::types::{
    DestroyConfirmation, KeyAlgorithm, KeyHandle, KeyMetadata, KeyRotationResult,
    PublicKeyBytes, Signature,
};

/// Core KMS provider abstraction — the heart of F1.
///
/// All signing key operations go through this trait. Implementations:
/// - `SoftwareKmsProvider` — local dev (in-memory keys via `ring`)
/// - `AwsKmsProvider` — staging (AWS KMS API)
/// - `AwsCloudHsmProvider` — production (PKCS#11 to CloudHSM)
///
/// Selected by `SAHI_KMS_PROVIDER` environment variable.
/// Injected as `Arc<dyn KmsProvider>` — provider swap requires zero code changes.
#[async_trait]
pub trait KmsProvider: Send + Sync {
    /// Generate a new key pair.
    /// In HSM mode, the private key never leaves the HSM.
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: &str,
        tenant_id: Option<&str>,
    ) -> Result<KeyHandle, CryptoError>;

    /// Sign data using a key stored in the KMS.
    /// Returns an error if the key is not in Active state.
    async fn sign(
        &self,
        key_handle: &KeyHandle,
        data: &[u8],
    ) -> Result<Signature, CryptoError>;

    /// Verify a signature.
    /// Can be done in software even for HSM-backed keys (only needs public key).
    async fn verify(
        &self,
        key_handle: &KeyHandle,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, CryptoError>;

    /// Export the public key (private key NEVER leaves the KMS).
    async fn export_public_key(
        &self,
        key_handle: &KeyHandle,
    ) -> Result<PublicKeyBytes, CryptoError>;

    /// Rotate a key: generate new key, mark old key as VerifyOnly.
    /// Old key remains valid for verification during the grace period.
    async fn rotate_key(
        &self,
        old_handle: &KeyHandle,
    ) -> Result<KeyRotationResult, CryptoError>;

    /// List all keys with metadata. Never returns private key material.
    async fn list_keys(
        &self,
        tenant_id: Option<&str>,
    ) -> Result<Vec<KeyMetadata>, CryptoError>;

    /// Destroy a key (irreversible).
    /// Requires confirmation matching the key handle.
    /// Metadata is retained for audit — only key material is destroyed.
    async fn destroy_key(
        &self,
        key_handle: &KeyHandle,
        confirmation: DestroyConfirmation,
    ) -> Result<(), CryptoError>;

    /// Get metadata for a specific key.
    async fn get_key_metadata(
        &self,
        key_handle: &KeyHandle,
    ) -> Result<KeyMetadata, CryptoError>;
}
