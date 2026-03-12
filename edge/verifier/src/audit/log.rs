//! Append-only audit log implementation using SQLite.

use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

/// Audit log errors.
#[derive(Debug, Error)]
pub enum AuditError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Access decision outcome.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessOutcome {
    Granted,
    Denied,
    Error,
}

/// Reason for denial.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialReason {
    InvalidCredential,
    ExpiredCredential,
    RevokedCredential,
    PolicyViolation,
    TimeRestriction,
    FloorRestriction,
    UnknownIssuer,
    CryptoFailure,
    StaleCache,
    TamperDetected,
    SystemError,
}

/// Audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID (auto-generated).
    pub id: i64,

    /// Timestamp of the event.
    pub timestamp: DateTime<Utc>,

    /// Site ID where event occurred.
    pub site_id: String,

    /// Event type.
    pub event_type: AuditEventType,

    /// Credential ID (if applicable).
    pub credential_id: Option<String>,

    /// Holder DID (if applicable).
    pub holder_did: Option<String>,

    /// Access outcome.
    pub outcome: Option<AccessOutcome>,

    /// Denial reason (if denied).
    pub denial_reason: Option<DenialReason>,

    /// Presentation method (BLE/NFC).
    pub method: Option<String>,

    /// Processing duration in milliseconds.
    pub duration_ms: Option<u32>,

    /// Additional metadata as JSON.
    pub metadata: Option<serde_json::Value>,

    /// Whether this entry has been synced to platform.
    pub synced: bool,
}

/// Audit event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    Startup,
    Shutdown,
    AccessAttempt,
    TamperAlert,
    ConnectivityChange,
    ConfigUpdate,
    CacheRefresh,
    Error,
}

/// Append-only audit log.
pub struct AuditLog {
    conn: Mutex<Connection>,
}

impl AuditLog {
    /// Create or open audit log database.
    pub fn new(path: &Path) -> Result<Self, AuditError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;

        // Create table if not exists
        conn.execute(
            r"CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                site_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                credential_id TEXT,
                holder_did TEXT,
                outcome TEXT,
                denial_reason TEXT,
                method TEXT,
                duration_ms INTEGER,
                metadata TEXT,
                synced INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Create index for sync queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_audit_synced ON audit_log (synced, timestamp)",
            [],
        )?;

        // Enable WAL mode for better concurrent access
        conn.pragma_update(None, "journal_mode", "WAL")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Log a startup event.
    pub async fn log_startup(&mut self, site_id: &str) -> Result<i64, AuditError> {
        self.log_event(AuditEntry {
            id: 0,
            timestamp: Utc::now(),
            site_id: site_id.to_string(),
            event_type: AuditEventType::Startup,
            credential_id: None,
            holder_did: None,
            outcome: None,
            denial_reason: None,
            method: None,
            duration_ms: None,
            metadata: None,
            synced: false,
        })
        .await
    }

    /// Log a shutdown event.
    pub async fn log_shutdown(&mut self, site_id: &str) -> Result<i64, AuditError> {
        self.log_event(AuditEntry {
            id: 0,
            timestamp: Utc::now(),
            site_id: site_id.to_string(),
            event_type: AuditEventType::Shutdown,
            credential_id: None,
            holder_did: None,
            outcome: None,
            denial_reason: None,
            method: None,
            duration_ms: None,
            metadata: None,
            synced: false,
        })
        .await
    }

    /// Log an access attempt.
    pub async fn log_access(
        &mut self,
        site_id: &str,
        credential_id: Option<&str>,
        holder_did: Option<&str>,
        outcome: AccessOutcome,
        denial_reason: Option<DenialReason>,
        method: &str,
        duration_ms: u32,
    ) -> Result<i64, AuditError> {
        self.log_event(AuditEntry {
            id: 0,
            timestamp: Utc::now(),
            site_id: site_id.to_string(),
            event_type: AuditEventType::AccessAttempt,
            credential_id: credential_id.map(String::from),
            holder_did: holder_did.map(String::from),
            outcome: Some(outcome),
            denial_reason,
            method: Some(method.to_string()),
            duration_ms: Some(duration_ms),
            metadata: None,
            synced: false,
        })
        .await
    }

    /// Log a tamper alert.
    pub async fn log_tamper(&mut self, site_id: &str, details: &str) -> Result<i64, AuditError> {
        self.log_event(AuditEntry {
            id: 0,
            timestamp: Utc::now(),
            site_id: site_id.to_string(),
            event_type: AuditEventType::TamperAlert,
            credential_id: None,
            holder_did: None,
            outcome: None,
            denial_reason: None,
            method: None,
            duration_ms: None,
            metadata: Some(serde_json::json!({ "details": details })),
            synced: false,
        })
        .await
    }

    /// Log a generic event.
    async fn log_event(&mut self, entry: AuditEntry) -> Result<i64, AuditError> {
        let conn = self.conn.lock().await;

        conn.execute(
            r"INSERT INTO audit_log
              (timestamp, site_id, event_type, credential_id, holder_did,
               outcome, denial_reason, method, duration_ms, metadata, synced)
              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                entry.timestamp.to_rfc3339(),
                entry.site_id,
                serde_json::to_string(&entry.event_type)?,
                entry.credential_id,
                entry.holder_did,
                entry
                    .outcome
                    .map(|o| serde_json::to_string(&o))
                    .transpose()?,
                entry
                    .denial_reason
                    .map(|r| serde_json::to_string(&r))
                    .transpose()?,
                entry.method,
                entry.duration_ms,
                entry.metadata.map(|m| m.to_string()),
                entry.synced as i32,
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get unsynced entries for batch upload.
    pub async fn get_unsynced(&self, limit: usize) -> Result<Vec<AuditEntry>, AuditError> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            r"SELECT id, timestamp, site_id, event_type, credential_id, holder_did,
                     outcome, denial_reason, method, duration_ms, metadata, synced
              FROM audit_log
              WHERE synced = 0
              ORDER BY timestamp ASC
              LIMIT ?",
        )?;

        let entries = stmt
            .query_map([limit], |row| {
                Ok(AuditEntry {
                    id: row.get(0)?,
                    timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(1)?)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    site_id: row.get(2)?,
                    event_type: serde_json::from_str(&row.get::<_, String>(3)?)
                        .unwrap_or(AuditEventType::Error),
                    credential_id: row.get(4)?,
                    holder_did: row.get(5)?,
                    outcome: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    denial_reason: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    method: row.get(8)?,
                    duration_ms: row.get(9)?,
                    metadata: row
                        .get::<_, Option<String>>(10)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    synced: row.get::<_, i32>(11)? != 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Mark entries as synced.
    pub async fn mark_synced(&mut self, ids: &[i64]) -> Result<(), AuditError> {
        if ids.is_empty() {
            return Ok(());
        }

        let conn = self.conn.lock().await;

        let placeholders: Vec<_> = ids.iter().map(|_| "?").collect();
        let sql = format!(
            "UPDATE audit_log SET synced = 1 WHERE id IN ({})",
            placeholders.join(",")
        );

        let params: Vec<_> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
        conn.execute(&sql, params.as_slice())?;

        Ok(())
    }

    /// Get total entry count.
    pub async fn count(&self) -> Result<i64, AuditError> {
        let conn = self.conn.lock().await;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get unsynced entry count.
    pub async fn unsynced_count(&self) -> Result<i64, AuditError> {
        let conn = self.conn.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_log WHERE synced = 0",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_audit_log_creation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit.db");

        let log = AuditLog::new(&db_path).unwrap();
        assert_eq!(log.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_log_startup() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit.db");

        let mut log = AuditLog::new(&db_path).unwrap();
        let id = log.log_startup("VRF_01HXK").await.unwrap();

        assert_eq!(id, 1);
        assert_eq!(log.count().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_log_access_granted() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit.db");

        let mut log = AuditLog::new(&db_path).unwrap();
        let id = log
            .log_access(
                "VRF_01HXK",
                Some("CRD_01HXK"),
                Some("did:key:z6Mk..."),
                AccessOutcome::Granted,
                None,
                "BLE",
                150,
            )
            .await
            .unwrap();

        assert_eq!(id, 1);

        let entries = log.get_unsynced(10).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].outcome, Some(AccessOutcome::Granted));
    }

    #[tokio::test]
    async fn test_log_access_denied() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit.db");

        let mut log = AuditLog::new(&db_path).unwrap();
        log.log_access(
            "VRF_01HXK",
            Some("CRD_01HXK"),
            Some("did:key:z6Mk..."),
            AccessOutcome::Denied,
            Some(DenialReason::RevokedCredential),
            "NFC",
            50,
        )
        .await
        .unwrap();

        let entries = log.get_unsynced(10).await.unwrap();
        assert_eq!(entries[0].outcome, Some(AccessOutcome::Denied));
        assert!(matches!(
            entries[0].denial_reason,
            Some(DenialReason::RevokedCredential)
        ));
    }

    #[tokio::test]
    async fn test_mark_synced() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("audit.db");

        let mut log = AuditLog::new(&db_path).unwrap();
        log.log_startup("VRF_01HXK").await.unwrap();
        log.log_startup("VRF_01HXK").await.unwrap();

        assert_eq!(log.unsynced_count().await.unwrap(), 2);

        log.mark_synced(&[1]).await.unwrap();
        assert_eq!(log.unsynced_count().await.unwrap(), 1);
    }
}
