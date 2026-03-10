use std::fmt;

/// ULID prefix registry (MASTER_PLAN Appendix G).
///
/// All entity identifiers follow the pattern `{PREFIX}_{ULID}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UlidPrefix {
    /// TNT_ — Tenant
    Tenant,
    /// USR_ — User
    User,
    /// CRD_ — Credential
    Credential,
    /// PRD_ — Product
    Product,
    /// KEY_ — Key Handle
    Key,
    /// LOG_ — Log Entry
    LogEntry,
    /// EVT_ — Event
    Event,
    /// REQ_ — Request
    Request,
    /// SVC_ — Service
    Service,
    /// CRM_ — Ceremony
    Ceremony,
    /// VRF_ — Verifier
    Verifier,
    /// PRY_ — Property
    Property,
    /// BTH_ — Batch
    Batch,
    /// CRT_ — Certificate
    Certificate,
    /// SIG_ — Signature
    Signature,
    /// TAG_ — NFC Tag
    Tag,
    /// INV_ — Invitation
    Invitation,
}

impl UlidPrefix {
    /// Returns the 3-4 character prefix string (without underscore).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tenant => "TNT",
            Self::User => "USR",
            Self::Credential => "CRD",
            Self::Product => "PRD",
            Self::Key => "KEY",
            Self::LogEntry => "LOG",
            Self::Event => "EVT",
            Self::Request => "REQ",
            Self::Service => "SVC",
            Self::Ceremony => "CRM",
            Self::Verifier => "VRF",
            Self::Property => "PRY",
            Self::Batch => "BTH",
            Self::Certificate => "CRT",
            Self::Signature => "SIG",
            Self::Tag => "TAG",
            Self::Invitation => "INV",
        }
    }

    /// Parse a prefix string (with or without underscore) into a `UlidPrefix`.
    pub fn parse_prefix(s: &str) -> Option<Self> {
        let prefix = s.trim_end_matches('_');
        match prefix {
            "TNT" => Some(Self::Tenant),
            "USR" => Some(Self::User),
            "CRD" => Some(Self::Credential),
            "PRD" => Some(Self::Product),
            "KEY" => Some(Self::Key),
            "LOG" => Some(Self::LogEntry),
            "EVT" => Some(Self::Event),
            "REQ" => Some(Self::Request),
            "SVC" => Some(Self::Service),
            "CRM" => Some(Self::Ceremony),
            "VRF" => Some(Self::Verifier),
            "PRY" => Some(Self::Property),
            "BTH" => Some(Self::Batch),
            "CRT" => Some(Self::Certificate),
            "SIG" => Some(Self::Signature),
            "TAG" => Some(Self::Tag),
            "INV" => Some(Self::Invitation),
            _ => None,
        }
    }
}

impl fmt::Display for UlidPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A typed ULID identifier with a validated prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedUlid {
    prefix: UlidPrefix,
    ulid: ulid::Ulid,
}

impl TypedUlid {
    /// Generate a new typed ULID with the given prefix.
    pub fn new(prefix: UlidPrefix) -> Self {
        Self {
            prefix,
            ulid: ulid::Ulid::new(),
        }
    }

    /// Returns the prefix.
    #[must_use]
    pub fn prefix(&self) -> UlidPrefix {
        self.prefix
    }

    /// Returns the raw ULID value.
    #[must_use]
    pub fn ulid(&self) -> ulid::Ulid {
        self.ulid
    }

    /// Parse a prefixed ULID string (e.g., "TNT_01HXK4Y5J6P8M2N3Q7R9S0T1").
    ///
    /// # Errors
    /// Returns an error if the prefix is unknown or the ULID portion is invalid.
    pub fn parse(s: &str) -> Result<Self, UlidParseError> {
        let (prefix_str, ulid_str) = s
            .split_once('_')
            .ok_or_else(|| UlidParseError::MissingUnderscore(s.to_string()))?;

        let prefix = UlidPrefix::parse_prefix(prefix_str)
            .ok_or_else(|| UlidParseError::UnknownPrefix(prefix_str.to_string()))?;

        let ulid = ulid_str
            .parse::<ulid::Ulid>()
            .map_err(|e| UlidParseError::InvalidUlid(e.to_string()))?;

        Ok(Self { prefix, ulid })
    }

    /// Validate that a string has the expected prefix format.
    ///
    /// # Errors
    /// Returns an error if parsing fails or the prefix doesn't match `expected`.
    pub fn validate(s: &str, expected: UlidPrefix) -> Result<(), UlidParseError> {
        let parsed = Self::parse(s)?;
        if parsed.prefix != expected {
            return Err(UlidParseError::WrongPrefix {
                expected: expected.as_str().to_string(),
                actual: parsed.prefix.as_str().to_string(),
            });
        }
        Ok(())
    }
}

impl fmt::Display for TypedUlid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}_{}", self.prefix.as_str(), self.ulid)
    }
}

impl serde::Serialize for TypedUlid {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for TypedUlid {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Errors from parsing typed ULIDs.
#[derive(Debug, Clone, thiserror::Error)]
pub enum UlidParseError {
    #[error("Missing underscore separator in ULID: {0}")]
    MissingUnderscore(String),

    #[error("Unknown ULID prefix: {0}")]
    UnknownPrefix(String),

    #[error("Invalid ULID value: {0}")]
    InvalidUlid(String),

    #[error("Wrong prefix: expected {expected}, got {actual}")]
    WrongPrefix { expected: String, actual: String },
}

/// Convenience function to generate a new prefixed ULID string.
pub fn generate(prefix: UlidPrefix) -> String {
    TypedUlid::new(prefix).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_has_correct_format() {
        let id = generate(UlidPrefix::Tenant);
        assert!(id.starts_with("TNT_"), "Expected TNT_ prefix, got {id}");
        assert_eq!(
            id.len(),
            30,
            "Expected 30 chars (3 prefix + 1 underscore + 26 ULID), got {}",
            id.len()
        );
    }

    #[test]
    fn all_prefixes_generate_unique_ids() {
        let prefixes = [
            UlidPrefix::Tenant,
            UlidPrefix::User,
            UlidPrefix::Credential,
            UlidPrefix::Product,
            UlidPrefix::Key,
            UlidPrefix::LogEntry,
            UlidPrefix::Event,
            UlidPrefix::Request,
            UlidPrefix::Service,
            UlidPrefix::Ceremony,
            UlidPrefix::Verifier,
            UlidPrefix::Property,
            UlidPrefix::Batch,
            UlidPrefix::Certificate,
            UlidPrefix::Signature,
            UlidPrefix::Tag,
            UlidPrefix::Invitation,
        ];

        for prefix in prefixes {
            let id = generate(prefix);
            assert!(id.starts_with(&format!("{}_", prefix.as_str())));
        }
    }

    #[test]
    fn parse_roundtrip() {
        let original = TypedUlid::new(UlidPrefix::Key);
        let s = original.to_string();
        let parsed = TypedUlid::parse(&s).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn parse_rejects_invalid_prefix() {
        let result = TypedUlid::parse("XXX_01HXK4Y5J6P8M2N3Q7R9S0T1");
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_missing_underscore() {
        let result = TypedUlid::parse("TNT01HXK4Y5J6P8M2N3Q7R9S0T1");
        assert!(result.is_err());
    }

    #[test]
    fn validate_checks_expected_prefix() {
        let id = generate(UlidPrefix::Tenant);
        assert!(TypedUlid::validate(&id, UlidPrefix::Tenant).is_ok());
        assert!(TypedUlid::validate(&id, UlidPrefix::User).is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let original = TypedUlid::new(UlidPrefix::Credential);
        let json = serde_json::to_string(&original).unwrap();
        let parsed: TypedUlid = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn prefix_parse_roundtrip() {
        let prefixes = [
            "TNT", "USR", "CRD", "PRD", "KEY", "LOG", "EVT", "REQ", "SVC", "CRM", "VRF", "PRY",
            "BTH", "CRT", "SIG", "TAG", "INV",
        ];
        for s in prefixes {
            let prefix = UlidPrefix::parse_prefix(s).unwrap();
            assert_eq!(prefix.as_str(), s);
        }
    }
}
