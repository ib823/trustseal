//! TM-1: Signer Domain Types
//!
//! Types for managing signers within a signing ceremony.
//! Each signer has a slot with their own sub-state machine:
//!
//! PENDING -> INVITED -> AUTHENTICATED -> SIGNED | DECLINED

#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::match_same_arms)]

use serde::{Deserialize, Serialize};

/// Signer slot ID with SLT_ prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SignerSlotId(pub String);

impl SignerSlotId {
    /// Create a new signer slot ID.
    pub fn new(id: impl Into<String>) -> Option<Self> {
        let id = id.into();
        if id.starts_with("SLT_") && id.len() == 30 {
            Some(Self(id))
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SignerSlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Signer role in the ceremony.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignerRole {
    /// Primary signatory
    Signatory,
    /// Witness (observes but may or may not sign)
    Witness,
    /// Approver (approves but signature not embedded)
    Approver,
    /// Notary (certified witness)
    Notary,
}

impl SignerRole {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Signatory => "signatory",
            Self::Witness => "witness",
            Self::Approver => "approver",
            Self::Notary => "notary",
        }
    }

    /// Parse from database string.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Wildcard fallback is intentional for safety
    pub fn from_str(s: &str) -> Self {
        match s {
            "signatory" => Self::Signatory,
            "witness" => Self::Witness,
            "approver" => Self::Approver,
            "notary" => Self::Notary,
            _ => Self::Signatory, // DB constraints prevent this
        }
    }
}

/// Signer status within a ceremony slot.
///
/// Sub-state machine: PENDING -> INVITED -> AUTHENTICATED -> SIGNED | DECLINED
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignerStatus {
    /// Initial state, awaiting invitation
    Pending,
    /// Invitation sent, awaiting authentication
    Invited,
    /// Authenticated via `WebAuthn`, ready to sign
    Authenticated,
    /// Signature completed
    Signed,
    /// Signer declined to sign
    Declined,
    /// Invitation expired
    Expired,
}

impl SignerStatus {
    /// Check if this is a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Signed | Self::Declined | Self::Expired)
    }

    /// Check if the signer has signed.
    #[must_use]
    pub const fn is_signed(&self) -> bool {
        matches!(self, Self::Signed)
    }

    /// Check if the signer can still act.
    #[must_use]
    pub const fn can_act(&self) -> bool {
        matches!(self, Self::Invited | Self::Authenticated)
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Invited => "INVITED",
            Self::Authenticated => "AUTHENTICATED",
            Self::Signed => "SIGNED",
            Self::Declined => "DECLINED",
            Self::Expired => "EXPIRED",
        }
    }

    /// Valid next states from this state.
    #[must_use]
    pub fn valid_transitions(&self) -> &'static [SignerStatus] {
        match self {
            Self::Pending => &[Self::Invited],
            Self::Invited => &[Self::Authenticated, Self::Declined, Self::Expired],
            Self::Authenticated => &[Self::Signed, Self::Declined],
            Self::Signed => &[],
            Self::Declined => &[],
            Self::Expired => &[],
        }
    }

    /// Check if transition to target is valid.
    #[must_use]
    pub fn can_transition_to(&self, target: &Self) -> bool {
        self.valid_transitions().contains(target)
    }

    /// Parse from database string.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Wildcard fallback is intentional for safety
    pub fn from_str(s: &str) -> Self {
        match s {
            "PENDING" => Self::Pending,
            "INVITED" => Self::Invited,
            "AUTHENTICATED" => Self::Authenticated,
            "SIGNED" => Self::Signed,
            "DECLINED" => Self::Declined,
            "EXPIRED" => Self::Expired,
            _ => Self::Pending, // DB constraints prevent this
        }
    }
}

impl std::fmt::Display for SignerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Invitation details for a signer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInvitation {
    /// Invitation token (for URL)
    pub token: String,
    /// When the invitation was sent (RFC 3339)
    pub sent_at: Option<String>,
    /// When the invitation expires (RFC 3339)
    pub expires_at: String,
    /// Number of reminder emails sent
    pub reminders_sent: u32,
}

/// A signer slot within a ceremony.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerSlot {
    /// Unique slot ID
    pub id: SignerSlotId,
    /// Order in signing sequence (for sequential ceremonies)
    pub order: u32,
    /// Signer's name
    pub name: String,
    /// Signer's email
    pub email: String,
    /// Signer's role
    pub role: SignerRole,
    /// Whether this signer is required
    pub is_required: bool,
    /// Current status
    pub status: SignerStatus,
    /// Invitation details
    pub invitation: Option<SignerInvitation>,
    /// `WebAuthn` credential ID (after authentication)
    pub webauthn_credential_id: Option<String>,
    /// Assurance level achieved (P1, P2, P3)
    pub assurance_level: Option<String>,
    /// When signature was completed
    pub signed_at: Option<String>,
    /// Signature data (for embedding in PDF)
    pub signature_data: Option<SignatureData>,
    /// Reason for declining (if declined)
    pub decline_reason: Option<String>,
}

/// Signature data after signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureData {
    /// CMS/PKCS#7 signature (base64-encoded)
    pub cms_signature: String,
    /// Timestamp token (RFC 3161, base64-encoded)
    pub timestamp_token: Option<String>,
    /// Signing certificate chain (PEM)
    pub certificate_chain: Vec<String>,
    /// Hash of the signed content
    pub content_hash: String,
    /// Signing algorithm used
    pub algorithm: String,
}

impl SignerSlot {
    /// Check if this slot can be signed now.
    #[must_use]
    pub fn can_sign(&self) -> bool {
        self.status == SignerStatus::Authenticated
    }

    /// Check if the invitation has expired.
    #[must_use]
    pub fn invitation_expired(&self, now: &str) -> bool {
        let Ok(current) = chrono::DateTime::parse_from_rfc3339(now)
            .map(|value| value.with_timezone(&chrono::Utc))
        else {
            return false;
        };

        self.invitation.as_ref().is_some_and(|inv| {
            chrono::DateTime::parse_from_rfc3339(&inv.expires_at)
                .map(|value| value.with_timezone(&chrono::Utc))
                .is_ok_and(|expires_at| expires_at <= current)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_slot_id_validation() {
        let valid = SignerSlotId::new("SLT_01HXK5M4N8JXJXJXJXJXJXJXJX");
        assert!(valid.is_some());

        let invalid = SignerSlotId::new("USR_01HXK5M4N8JXJXJXJXJXJXJXJX");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_signer_status_transitions() {
        let pending = SignerStatus::Pending;
        assert!(pending.can_transition_to(&SignerStatus::Invited));
        assert!(!pending.can_transition_to(&SignerStatus::Signed));

        let invited = SignerStatus::Invited;
        assert!(invited.can_transition_to(&SignerStatus::Authenticated));
        assert!(invited.can_transition_to(&SignerStatus::Declined));
        assert!(invited.can_transition_to(&SignerStatus::Expired));
        assert!(!invited.can_transition_to(&SignerStatus::Signed));

        let authenticated = SignerStatus::Authenticated;
        assert!(authenticated.can_transition_to(&SignerStatus::Signed));
        assert!(authenticated.can_transition_to(&SignerStatus::Declined));
        assert!(!authenticated.can_transition_to(&SignerStatus::Invited));
    }

    #[test]
    fn test_terminal_states() {
        assert!(SignerStatus::Signed.is_terminal());
        assert!(SignerStatus::Declined.is_terminal());
        assert!(SignerStatus::Expired.is_terminal());
        assert!(!SignerStatus::Pending.is_terminal());
        assert!(!SignerStatus::Invited.is_terminal());
        assert!(!SignerStatus::Authenticated.is_terminal());
    }

    #[test]
    fn test_signer_role_serialization() {
        let role = SignerRole::Signatory;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"signatory\"");
    }

    #[test]
    fn test_signer_status_serialization() {
        let status = SignerStatus::Authenticated;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"AUTHENTICATED\"");
    }
}
