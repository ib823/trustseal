//! Ceremony transition repository for audit logging.

// Allow common patterns in database repository code
#![allow(clippy::needless_raw_string_hashes)]

use sqlx::postgres::PgPool;

use crate::domain::{CeremonyState, CeremonyTransition};
use crate::orchestrator::OrchestratorError;

/// Repository for ceremony transition (audit log) operations.
pub struct TransitionRepository {
    pool: PgPool,
}

impl TransitionRepository {
    /// Create a new repository with the given database pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Record a state transition in the audit log.
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn record(
        &self,
        ceremony_id: &str,
        tenant_id: &str,
        transition: &CeremonyTransition,
    ) -> Result<String, OrchestratorError> {
        let id = format!("TRN_{}", ulid::Ulid::new());

        let timestamp = chrono::DateTime::parse_from_rfc3339(&transition.timestamp)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| OrchestratorError::DatabaseError(format!("Invalid timestamp: {e}")))?;

        sqlx::query(
            r#"
            INSERT INTO ceremony_transitions (
                id, ceremony_id, tenant_id,
                from_state, to_state, reason, actor,
                transitioned_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&id)
        .bind(ceremony_id)
        .bind(tenant_id)
        .bind(transition.from_state.as_str())
        .bind(transition.to_state.as_str())
        .bind(&transition.reason)
        .bind(&transition.actor)
        .bind(timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            OrchestratorError::DatabaseError(format!("Failed to record transition: {e}"))
        })?;

        Ok(id)
    }

    /// Get all transitions for a ceremony (for audit trail).
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn list_for_ceremony(
        &self,
        ceremony_id: &str,
    ) -> Result<Vec<CeremonyTransition>, OrchestratorError> {
        #[derive(Debug, sqlx::FromRow)]
        struct TransitionRow {
            from_state: String,
            to_state: String,
            reason: String,
            actor: String,
            transitioned_at: chrono::DateTime<chrono::Utc>,
        }

        let rows: Vec<TransitionRow> = sqlx::query_as(
            r#"
            SELECT from_state, to_state, reason, actor, transitioned_at
            FROM ceremony_transitions
            WHERE ceremony_id = $1
            ORDER BY transitioned_at ASC
            "#,
        )
        .bind(ceremony_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            OrchestratorError::DatabaseError(format!("Failed to list transitions: {e}"))
        })?;

        let transitions = rows
            .into_iter()
            .map(|row| CeremonyTransition {
                from_state: CeremonyState::from_str(&row.from_state),
                to_state: CeremonyState::from_str(&row.to_state),
                reason: row.reason,
                actor: row.actor,
                timestamp: row.transitioned_at.to_rfc3339(),
            })
            .collect();

        Ok(transitions)
    }
}
