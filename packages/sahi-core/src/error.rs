use std::fmt;

use serde::{Deserialize, Serialize};

/// Platform-wide error code registry (MASTER_PLAN Appendix E).
///
/// All errors follow the pattern `SAHI_XXXX` where X is a 4-digit code.
/// Each domain has a dedicated range to avoid collisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    // ── Authentication (1000-1099) ──────────────────────────────────────
    /// SAHI_1001: JWT expired
    JwtExpired,
    /// SAHI_1002: Invalid signature on token
    InvalidSignature,
    /// SAHI_1003: WebAuthn authentication failed
    WebAuthnFailed,

    // ── Authorization (1100-1199) ───────────────────────────────────────
    /// SAHI_1100: Tenant mismatch (cross-tenant access attempt)
    TenantMismatch,
    /// SAHI_1101: Insufficient role for operation
    InsufficientRole,
    /// SAHI_1102: Row-level security violation
    RlsViolation,

    // ── KMS (2000-2099) ─────────────────────────────────────────────────
    /// SAHI_2001: Key not found in KMS
    KeyNotFound,
    /// SAHI_2002: Signing operation failed
    SignFailed,
    /// SAHI_2003: HSM/KMS provider unreachable
    ProviderUnavailable,
    /// SAHI_2004: Key generation failed
    KeyGenerationFailed,
    /// SAHI_2005: Verification failed (invalid signature)
    VerificationFailed,
    /// SAHI_2006: Key rotation failed
    RotationFailed,
    /// SAHI_2007: Key destruction failed
    DestructionFailed,
    /// SAHI_2008: Unsupported algorithm
    UnsupportedAlgorithm,
    /// SAHI_2009: Invalid key state for requested operation
    InvalidKeyState,
    /// SAHI_2010: Destroy confirmation mismatch
    DestroyConfirmationFailed,

    // ── Credential (2100-2199) ──────────────────────────────────────────
    /// SAHI_2100: Invalid SD-JWT format
    InvalidSdJwt,
    /// SAHI_2101: Credential has been revoked
    CredentialRevoked,
    /// SAHI_2102: Credential has expired
    CredentialExpired,

    // ── Merkle Log (2200-2299) ──────────────────────────────────────────
    /// SAHI_2200: Sequence number gap detected
    SequenceGap,
    /// SAHI_2201: Merkle root mismatch
    RootMismatch,
    /// SAHI_2202: Append to Merkle log failed
    AppendFailed,

    // ── Signing Ceremony (3000-3099) ────────────────────────────────────
    /// SAHI_3001: Signing ceremony expired
    CeremonyExpired,
    /// SAHI_3002: Invalid ceremony state transition
    InvalidTransition,
    /// SAHI_3003: Document hash mismatch
    DocumentHashMismatch,

    // ── PAdES (3100-3199) ───────────────────────────────────────────────
    /// SAHI_3100: PDF parsing error
    PdfParseError,
    /// SAHI_3101: Timestamp authority unreachable
    TsaUnreachable,
    /// SAHI_3102: PAdES augmentation failed
    AugmentationFailed,

    // ── COSE / Labels (4000-4099) ───────────────────────────────────────
    /// SAHI_4001: COSE token expired
    TokenExpired,
    /// SAHI_4002: SUN (Secure Unique NFC) validation failed
    SunValidationFailed,
    /// SAHI_4003: NFC tag tamper detected
    TagTampered,

    // ── Gate / Verifier (5000-5099) ─────────────────────────────────────
    /// SAHI_5001: Credential denied at gate
    CredentialDenied,
    /// SAHI_5002: Offline cache is stale
    OfflineStaleCache,
    /// SAHI_5003: Tamper detected on verifier device
    TamperDetected,

    // ── Internal (9000-9099) ────────────────────────────────────────────
    /// SAHI_9001: Database error
    DatabaseError,
    /// SAHI_9002: Redis error
    RedisError,
    /// SAHI_9003: Configuration error
    ConfigurationError,
}

impl ErrorCode {
    /// Returns the SAHI_XXXX code string.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            // Authentication
            Self::JwtExpired => "SAHI_1001",
            Self::InvalidSignature => "SAHI_1002",
            Self::WebAuthnFailed => "SAHI_1003",
            // Authorization
            Self::TenantMismatch => "SAHI_1100",
            Self::InsufficientRole => "SAHI_1101",
            Self::RlsViolation => "SAHI_1102",
            // KMS
            Self::KeyNotFound => "SAHI_2001",
            Self::SignFailed => "SAHI_2002",
            Self::ProviderUnavailable => "SAHI_2003",
            Self::KeyGenerationFailed => "SAHI_2004",
            Self::VerificationFailed => "SAHI_2005",
            Self::RotationFailed => "SAHI_2006",
            Self::DestructionFailed => "SAHI_2007",
            Self::UnsupportedAlgorithm => "SAHI_2008",
            Self::InvalidKeyState => "SAHI_2009",
            Self::DestroyConfirmationFailed => "SAHI_2010",
            // Credential
            Self::InvalidSdJwt => "SAHI_2100",
            Self::CredentialRevoked => "SAHI_2101",
            Self::CredentialExpired => "SAHI_2102",
            // Merkle Log
            Self::SequenceGap => "SAHI_2200",
            Self::RootMismatch => "SAHI_2201",
            Self::AppendFailed => "SAHI_2202",
            // Signing Ceremony
            Self::CeremonyExpired => "SAHI_3001",
            Self::InvalidTransition => "SAHI_3002",
            Self::DocumentHashMismatch => "SAHI_3003",
            // PAdES
            Self::PdfParseError => "SAHI_3100",
            Self::TsaUnreachable => "SAHI_3101",
            Self::AugmentationFailed => "SAHI_3102",
            // COSE / Labels
            Self::TokenExpired => "SAHI_4001",
            Self::SunValidationFailed => "SAHI_4002",
            Self::TagTampered => "SAHI_4003",
            // Gate / Verifier
            Self::CredentialDenied => "SAHI_5001",
            Self::OfflineStaleCache => "SAHI_5002",
            Self::TamperDetected => "SAHI_5003",
            // Internal
            Self::DatabaseError => "SAHI_9001",
            Self::RedisError => "SAHI_9002",
            Self::ConfigurationError => "SAHI_9003",
        }
    }

    /// Returns the error domain name.
    #[must_use]
    pub fn domain(self) -> &'static str {
        match self {
            Self::JwtExpired | Self::InvalidSignature | Self::WebAuthnFailed => "Authentication",
            Self::TenantMismatch | Self::InsufficientRole | Self::RlsViolation => "Authorization",
            Self::KeyNotFound
            | Self::SignFailed
            | Self::ProviderUnavailable
            | Self::KeyGenerationFailed
            | Self::VerificationFailed
            | Self::RotationFailed
            | Self::DestructionFailed
            | Self::UnsupportedAlgorithm
            | Self::InvalidKeyState
            | Self::DestroyConfirmationFailed => "KMS",
            Self::InvalidSdJwt | Self::CredentialRevoked | Self::CredentialExpired => "Credential",
            Self::SequenceGap | Self::RootMismatch | Self::AppendFailed => "MerkleLog",
            Self::CeremonyExpired | Self::InvalidTransition | Self::DocumentHashMismatch => {
                "SigningCeremony"
            }
            Self::PdfParseError | Self::TsaUnreachable | Self::AugmentationFailed => "PAdES",
            Self::TokenExpired | Self::SunValidationFailed | Self::TagTampered => "COSE",
            Self::CredentialDenied | Self::OfflineStaleCache | Self::TamperDetected => "Gate",
            Self::DatabaseError | Self::RedisError | Self::ConfigurationError => "Internal",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Platform-wide error type for API responses.
///
/// Serializes to the standard error response format:
/// ```json
/// {
///   "error": {
///     "code": "SAHI_2101",
///     "message": "Credential has been revoked",
///     "action": "Request a new credential",
///     "request_id": "REQ_01HXK..."
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SahiError {
    pub code: String,
    pub message: String,
    pub action: String,
    pub request_id: Option<String>,
}

impl SahiError {
    pub fn new(code: ErrorCode, message: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            code: code.code().to_string(),
            message: message.into(),
            action: action.into(),
            request_id: None,
        }
    }

    #[must_use]
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

impl fmt::Display for SahiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for SahiError {}

/// Wrapper for JSON error responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: SahiError,
}

impl From<SahiError> for ErrorResponse {
    fn from(error: SahiError) -> Self {
        Self { error }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_have_correct_format() {
        let codes = [
            ErrorCode::JwtExpired,
            ErrorCode::KeyNotFound,
            ErrorCode::InvalidSdJwt,
            ErrorCode::CeremonyExpired,
            ErrorCode::PdfParseError,
            ErrorCode::TokenExpired,
            ErrorCode::CredentialDenied,
            ErrorCode::DatabaseError,
        ];

        for code in codes {
            let s = code.code();
            assert!(s.starts_with("SAHI_"), "Code {s} must start with SAHI_");
            assert_eq!(s.len(), 9, "Code {s} must be 9 chars (SAHI_XXXX)");
            assert!(
                s[5..].chars().all(|c| c.is_ascii_digit()),
                "Code {s} suffix must be digits"
            );
        }
    }

    #[test]
    fn error_codes_have_unique_values() {
        use std::collections::HashSet;
        let all_codes = [
            ErrorCode::JwtExpired,
            ErrorCode::InvalidSignature,
            ErrorCode::WebAuthnFailed,
            ErrorCode::TenantMismatch,
            ErrorCode::InsufficientRole,
            ErrorCode::RlsViolation,
            ErrorCode::KeyNotFound,
            ErrorCode::SignFailed,
            ErrorCode::ProviderUnavailable,
            ErrorCode::KeyGenerationFailed,
            ErrorCode::VerificationFailed,
            ErrorCode::RotationFailed,
            ErrorCode::DestructionFailed,
            ErrorCode::UnsupportedAlgorithm,
            ErrorCode::InvalidKeyState,
            ErrorCode::DestroyConfirmationFailed,
            ErrorCode::InvalidSdJwt,
            ErrorCode::CredentialRevoked,
            ErrorCode::CredentialExpired,
            ErrorCode::SequenceGap,
            ErrorCode::RootMismatch,
            ErrorCode::AppendFailed,
            ErrorCode::CeremonyExpired,
            ErrorCode::InvalidTransition,
            ErrorCode::DocumentHashMismatch,
            ErrorCode::PdfParseError,
            ErrorCode::TsaUnreachable,
            ErrorCode::AugmentationFailed,
            ErrorCode::TokenExpired,
            ErrorCode::SunValidationFailed,
            ErrorCode::TagTampered,
            ErrorCode::CredentialDenied,
            ErrorCode::OfflineStaleCache,
            ErrorCode::TamperDetected,
            ErrorCode::DatabaseError,
            ErrorCode::RedisError,
            ErrorCode::ConfigurationError,
        ];

        let mut seen = HashSet::new();
        for code in all_codes {
            assert!(
                seen.insert(code.code()),
                "Duplicate error code: {}",
                code.code()
            );
        }
        assert_eq!(seen.len(), 37, "Expected 37 unique error codes");
    }

    #[test]
    fn error_domains_are_correct() {
        assert_eq!(ErrorCode::JwtExpired.domain(), "Authentication");
        assert_eq!(ErrorCode::TenantMismatch.domain(), "Authorization");
        assert_eq!(ErrorCode::KeyNotFound.domain(), "KMS");
        assert_eq!(ErrorCode::InvalidSdJwt.domain(), "Credential");
        assert_eq!(ErrorCode::SequenceGap.domain(), "MerkleLog");
        assert_eq!(ErrorCode::CeremonyExpired.domain(), "SigningCeremony");
        assert_eq!(ErrorCode::PdfParseError.domain(), "PAdES");
        assert_eq!(ErrorCode::TokenExpired.domain(), "COSE");
        assert_eq!(ErrorCode::CredentialDenied.domain(), "Gate");
        assert_eq!(ErrorCode::DatabaseError.domain(), "Internal");
    }

    #[test]
    fn sahi_error_serializes_correctly() {
        let err = SahiError::new(
            ErrorCode::CredentialRevoked,
            "Credential has been revoked",
            "Request a new credential from your property administrator",
        )
        .with_request_id("REQ_01HXK4Y5J6P8M2N3Q7R9S0T1");

        let response = ErrorResponse::from(err);
        let json = serde_json::to_value(&response).unwrap();

        assert_eq!(json["error"]["code"], "SAHI_2101");
        assert_eq!(json["error"]["message"], "Credential has been revoked");
        assert!(json["error"]["request_id"]
            .as_str()
            .unwrap()
            .starts_with("REQ_"));
    }

    #[test]
    fn error_display_format() {
        assert_eq!(ErrorCode::KeyNotFound.to_string(), "SAHI_2001");
        let err = SahiError::new(ErrorCode::KeyNotFound, "Key not found", "Check key ID");
        assert_eq!(err.to_string(), "[SAHI_2001] Key not found");
    }
}
