//! eKYC types for VP-9 Onboarding & KYC flows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Verification provider.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationProvider {
    /// MyDigital ID (Malaysian government identity platform).
    #[default]
    MydigitalId,
    /// Manual verification (admin-approved).
    Manual,
}

impl std::fmt::Display for VerificationProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MydigitalId => write!(f, "mydigital_id"),
            Self::Manual => write!(f, "manual"),
        }
    }
}

/// Verification status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    /// Verification initiated but not started.
    #[default]
    Pending,
    /// OAuth flow in progress.
    InProgress,
    /// Successfully verified.
    Verified,
    /// Verification failed.
    Failed,
    /// Verification expired (needs re-verification).
    Expired,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Verified => write!(f, "verified"),
            Self::Failed => write!(f, "failed"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

/// Assurance level per TrustMark spec.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssuranceLevel {
    /// Basic: email/phone verified, synced passkey OK.
    #[default]
    P1,
    /// Enhanced: eKYC-verified identity, biometric passkey.
    P2,
    /// Qualified: CA-issued certificate, hardware security key, in-person.
    P3,
}

impl std::fmt::Display for AssuranceLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P1 => write!(f, "P1"),
            Self::P2 => write!(f, "P2"),
            Self::P3 => write!(f, "P3"),
        }
    }
}

/// Identity verification record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityVerification {
    pub id: String,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub status: VerificationStatus,
    pub provider: VerificationProvider,
    pub assurance_level: AssuranceLevel,
    pub name_hash: Option<String>,
    pub ic_hash: Option<String>,
    pub did: Option<String>,
    pub did_bound_at: Option<DateTime<Utc>>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// OAuth session for PKCE flow.
#[derive(Debug, Clone)]
pub struct OAuthSession {
    pub id: String,
    pub tenant_id: String,
    pub verification_id: String,
    pub state: String,
    pub nonce: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub redirect_uri: String,
    pub scope: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// MyDigital ID verified claims.
///
/// IMPORTANT: These are hashed on receipt, never stored raw.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedClaims {
    /// SHA-256 hash of normalized name (lowercase, trim).
    pub name_hash: String,
    /// SHA-256 hash of IC number (digits only).
    pub ic_hash: String,
    /// Assurance level from provider.
    pub assurance_level: AssuranceLevel,
}

/// OAuth token response from MyDigital ID.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

/// User info response from MyDigital ID.
#[derive(Debug, Clone, Deserialize)]
pub struct UserInfoResponse {
    pub sub: String,
    pub name: Option<String>,
    pub ic_number: Option<String>,
    pub verified: Option<bool>,
}
