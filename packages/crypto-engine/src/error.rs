use std::fmt;

/// All crypto-engine errors follow the SAHI_XXXX code convention.
/// KMS errors: SAHI_2000-2099
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
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
}

impl ErrorCode {
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
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
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("[{code}] KMS error: {message}")]
    Kms { code: ErrorCode, message: String },

    #[error("[SAHI_9001] Internal error: {0}")]
    Internal(String),
}

impl CryptoError {
    pub fn kms(code: ErrorCode, message: impl Into<String>) -> Self {
        Self::Kms {
            code,
            message: message.into(),
        }
    }
}
