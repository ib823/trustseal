use chrono::{DateTime, Utc};

/// Returns the current UTC timestamp.
///
/// All internal timestamps must use UTC. Local timezone conversion
/// happens only in the UI display layer (MASTER_PLAN §24.3).
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Format a timestamp as RFC 3339 (e.g., "2026-03-10T12:00:00Z").
///
/// This is the only acceptable format for API responses and logs.
pub fn to_rfc3339(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Parse an RFC 3339 timestamp string.
///
/// # Errors
/// Returns an error if the string is not valid RFC 3339.
pub fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    s.parse::<DateTime<Utc>>()
}

/// Check if a timestamp is in the past.
pub fn is_expired(dt: &DateTime<Utc>) -> bool {
    *dt < Utc::now()
}

/// Check if a timestamp is in the future.
pub fn is_future(dt: &DateTime<Utc>) -> bool {
    *dt > Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn now_returns_utc() {
        let ts = now();
        assert_eq!(ts.timezone(), Utc);
    }

    #[test]
    fn rfc3339_roundtrip() {
        let ts = now();
        let s = to_rfc3339(&ts);
        let parsed = parse_rfc3339(&s).unwrap();
        assert_eq!(ts.timestamp(), parsed.timestamp());
    }

    #[test]
    fn rfc3339_format_ends_with_z() {
        let s = to_rfc3339(&now());
        assert!(s.ends_with('Z'), "RFC 3339 must use Z suffix, got {s}");
    }

    #[test]
    fn expired_and_future_checks() {
        let past = now() - Duration::hours(1);
        let future = now() + Duration::hours(1);
        assert!(is_expired(&past));
        assert!(!is_expired(&future));
        assert!(is_future(&future));
        assert!(!is_future(&past));
    }

    #[test]
    fn parse_rejects_invalid_format() {
        assert!(parse_rfc3339("not-a-date").is_err());
        assert!(parse_rfc3339("2026-03-10").is_err());
    }
}
