//! Signer slot repository with `SQLx` queries.

// Allow common patterns in database repository code
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_raw_string_hashes)]

use sqlx::postgres::PgPool;

use crate::domain::{SignatureData, SignerSlot, SignerSlotId, SignerStatus};
use crate::orchestrator::OrchestratorError;

/// Repository for signer slot database operations.
pub struct SignerSlotRepository {
    pool: PgPool,
}

impl SignerSlotRepository {
    /// Create a new repository with the given database pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Update a signer slot's status and related fields.
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn update_status(
        &self,
        signer_id: &SignerSlotId,
        status: SignerStatus,
        webauthn_credential_id: Option<&str>,
        assurance_level: Option<&str>,
    ) -> Result<(), OrchestratorError> {
        sqlx::query(
            r#"
            UPDATE signer_slots
            SET status = $1,
                webauthn_credential_id = COALESCE($2, webauthn_credential_id),
                assurance_level = COALESCE($3, assurance_level),
                updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(status.as_str())
        .bind(webauthn_credential_id)
        .bind(assurance_level)
        .bind(&signer_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            OrchestratorError::DatabaseError(format!("Failed to update signer status: {e}"))
        })?;

        Ok(())
    }

    /// Record a signature for a signer.
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn record_signature(
        &self,
        signer_id: &SignerSlotId,
        signature_data: &SignatureData,
        signed_at: &str,
    ) -> Result<(), OrchestratorError> {
        let signed_at = chrono::DateTime::parse_from_rfc3339(signed_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| OrchestratorError::DatabaseError(format!("Invalid timestamp: {e}")))?;

        sqlx::query(
            r#"
            UPDATE signer_slots
            SET status = 'SIGNED',
                signature_data = $1,
                signed_at = $2,
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(sqlx::types::Json(signature_data))
        .bind(signed_at)
        .bind(&signer_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            OrchestratorError::DatabaseError(format!("Failed to record signature: {e}"))
        })?;

        Ok(())
    }

    /// Record a signer decline.
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn record_decline(
        &self,
        signer_id: &SignerSlotId,
        reason: &str,
    ) -> Result<(), OrchestratorError> {
        sqlx::query(
            r#"
            UPDATE signer_slots
            SET status = 'DECLINED',
                decline_reason = $1,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(reason)
        .bind(&signer_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to record decline: {e}")))?;

        Ok(())
    }

    /// Send an invitation to a signer.
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn send_invitation(
        &self,
        signer_id: &SignerSlotId,
        token: &str,
        expires_at: &str,
    ) -> Result<(), OrchestratorError> {
        let expires_at = chrono::DateTime::parse_from_rfc3339(expires_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .map_err(|e| OrchestratorError::DatabaseError(format!("Invalid timestamp: {e}")))?;

        sqlx::query(
            r#"
            UPDATE signer_slots
            SET status = 'INVITED',
                invitation_token = $1,
                invitation_sent_at = NOW(),
                invitation_expires_at = $2,
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(token)
        .bind(expires_at)
        .bind(&signer_id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to send invitation: {e}")))?;

        Ok(())
    }

    /// Get a signer by invitation token.
    ///
    /// # Errors
    /// Returns error if not found or database operation fails.
    pub async fn get_by_token(&self, token: &str) -> Result<SignerSlot, OrchestratorError> {
        use crate::domain::{SignerInvitation, SignerRole};

        #[derive(Debug, sqlx::FromRow)]
        struct SignerRow {
            id: String,
            name: String,
            email: String,
            role: String,
            signing_order: i32,
            is_required: bool,
            status: String,
            invitation_token: Option<String>,
            invitation_sent_at: Option<chrono::DateTime<chrono::Utc>>,
            invitation_expires_at: Option<chrono::DateTime<chrono::Utc>>,
            webauthn_credential_id: Option<String>,
            assurance_level: Option<String>,
            signature_data: Option<sqlx::types::Json<SignatureData>>,
            signed_at: Option<chrono::DateTime<chrono::Utc>>,
            decline_reason: Option<String>,
        }

        let row: SignerRow = sqlx::query_as(
            r#"
            SELECT
                id, name, email, role, signing_order, is_required, status,
                invitation_token, invitation_sent_at, invitation_expires_at,
                webauthn_credential_id, assurance_level, signature_data,
                signed_at, decline_reason
            FROM signer_slots
            WHERE invitation_token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to fetch signer: {e}")))?
        .ok_or_else(|| OrchestratorError::SignerNotFound(format!("token:{token}")))?;

        let invitation = row.invitation_token.map(|t| {
            let sent_at: Option<String> = row.invitation_sent_at.map(|ts| ts.to_rfc3339());
            let expires_at: String = row
                .invitation_expires_at
                .map(|ts| ts.to_rfc3339())
                .unwrap_or_default();
            SignerInvitation {
                token: t,
                sent_at,
                expires_at,
                reminders_sent: 0,
            }
        });

        let signed_at: Option<String> = row.signed_at.map(|t| t.to_rfc3339());

        Ok(SignerSlot {
            id: SignerSlotId(row.id),
            order: row.signing_order as u32,
            name: row.name,
            email: row.email,
            role: SignerRole::from_str(&row.role),
            is_required: row.is_required,
            status: SignerStatus::from_str(&row.status),
            invitation,
            webauthn_credential_id: row.webauthn_credential_id,
            assurance_level: row.assurance_level,
            signed_at,
            signature_data: row.signature_data.map(|j| j.0),
            decline_reason: row.decline_reason,
        })
    }
}
