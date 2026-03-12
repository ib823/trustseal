//! Ceremony repository with `SQLx` queries.

// Allow common patterns in database repository code
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::needless_raw_string_hashes)]

use sqlx::postgres::PgPool;

use crate::domain::{
    Ceremony, CeremonyConfig, CeremonyDocument, CeremonyId, CeremonyMetadata, CeremonyState,
    CeremonyType, SignatureField, SignerSlot,
};
use crate::orchestrator::OrchestratorError;

/// Repository for signing ceremony database operations.
pub struct CeremonyRepository {
    pool: PgPool,
}

/// Database row for ceremony.
#[derive(Debug, sqlx::FromRow)]
struct CeremonyRow {
    id: String,
    tenant_id: String,
    created_by: String,
    state: String,
    state_before_abort: Option<String>,
    version: i64,
    ceremony_type: String,
    config: sqlx::types::Json<CeremonyConfig>,
    title: String,
    description: Option<String>,
    reference: Option<String>,
    tags: Vec<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

/// Database row for ceremony document.
#[derive(Debug, sqlx::FromRow)]
struct DocumentRow {
    id: String,
    #[allow(dead_code)]
    ceremony_id: String,
    filename: String,
    content_type: String,
    content_hash: String,
    size_bytes: i64,
    storage_key: String,
    signature_fields: sqlx::types::Json<Vec<SignatureField>>,
}

impl CeremonyRepository {
    /// Create a new repository with the given database pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new ceremony in the database.
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn create(&self, ceremony: &Ceremony) -> Result<(), OrchestratorError> {
        let mut tx = self.pool.begin().await.map_err(|e| {
            OrchestratorError::DatabaseError(format!("Failed to begin transaction: {e}"))
        })?;

        let state_before = ceremony.state_before_abort.map(|s| s.as_str().to_string());
        let created_at = parse_rfc3339(&ceremony.created_at)?;
        let updated_at = parse_rfc3339(&ceremony.updated_at)?;
        let expires_at = parse_rfc3339(&ceremony.expires_at)?;

        // Insert ceremony
        sqlx::query(
            r#"
            INSERT INTO signing_ceremonies (
                id, tenant_id, created_by, state, state_before_abort, version,
                ceremony_type, config, title, description, reference, tags,
                created_at, updated_at, expires_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            )
            "#,
        )
        .bind(&ceremony.id.0)
        .bind(&ceremony.tenant_id)
        .bind(&ceremony.created_by)
        .bind(ceremony.state.as_str())
        .bind(&state_before)
        .bind(ceremony.version)
        .bind(ceremony.ceremony_type.as_str())
        .bind(sqlx::types::Json(&ceremony.config))
        .bind(&ceremony.metadata.title)
        .bind(&ceremony.metadata.description)
        .bind(&ceremony.metadata.reference)
        .bind(&ceremony.metadata.tags)
        .bind(created_at)
        .bind(updated_at)
        .bind(expires_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to insert ceremony: {e}")))?;

        // Insert document
        sqlx::query(
            r#"
            INSERT INTO ceremony_documents (
                id, ceremony_id, tenant_id, filename, content_type,
                content_hash, size_bytes, storage_key, signature_fields
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(&ceremony.document.id)
        .bind(&ceremony.id.0)
        .bind(&ceremony.tenant_id)
        .bind(&ceremony.document.filename)
        .bind(&ceremony.document.content_type)
        .bind(&ceremony.document.content_hash)
        .bind(ceremony.document.size_bytes as i64)
        .bind(&ceremony.document.storage_key)
        .bind(sqlx::types::Json(&ceremony.document.signature_fields))
        .execute(&mut *tx)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to insert document: {e}")))?;

        // Insert signer slots
        for signer in &ceremony.signers {
            sqlx::query(
                r#"
                INSERT INTO signer_slots (
                    id, ceremony_id, tenant_id, name, email,
                    role, signing_order, is_required, status
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(&signer.id.0)
            .bind(&ceremony.id.0)
            .bind(&ceremony.tenant_id)
            .bind(&signer.name)
            .bind(&signer.email)
            .bind(signer.role.as_str())
            .bind(signer.order as i32)
            .bind(signer.is_required)
            .bind(signer.status.as_str())
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                OrchestratorError::DatabaseError(format!("Failed to insert signer: {e}"))
            })?;
        }

        tx.commit().await.map_err(|e| {
            OrchestratorError::DatabaseError(format!("Failed to commit transaction: {e}"))
        })?;

        Ok(())
    }

    /// Get a ceremony by ID.
    ///
    /// # Errors
    /// Returns error if ceremony not found or database operation fails.
    pub async fn get_by_id(&self, id: &CeremonyId) -> Result<Ceremony, OrchestratorError> {
        // Fetch ceremony
        let row: CeremonyRow = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, created_by, state, state_before_abort, version,
                ceremony_type, config, title, description, reference, tags,
                created_at, updated_at, expires_at
            FROM signing_ceremonies
            WHERE id = $1
            "#,
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to fetch ceremony: {e}")))?
        .ok_or_else(|| OrchestratorError::CeremonyNotFound(id.0.clone()))?;

        // Fetch document
        let doc_row: DocumentRow = sqlx::query_as(
            r#"
            SELECT
                id, ceremony_id, filename, content_type, content_hash,
                size_bytes, storage_key, signature_fields
            FROM ceremony_documents
            WHERE ceremony_id = $1
            "#,
        )
        .bind(&id.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to fetch document: {e}")))?;

        // Fetch signers
        let signers = self.get_signers_for_ceremony(&id.0).await?;

        // Build ceremony
        let state_before_abort = row
            .state_before_abort
            .as_ref()
            .map(|s| CeremonyState::from_str(s));

        let ceremony = Ceremony {
            id: CeremonyId(row.id),
            tenant_id: row.tenant_id,
            created_by: row.created_by,
            state: CeremonyState::from_str(&row.state),
            state_before_abort,
            ceremony_type: CeremonyType::from_str(&row.ceremony_type),
            document: CeremonyDocument {
                id: doc_row.id,
                filename: doc_row.filename,
                content_type: doc_row.content_type,
                content_hash: doc_row.content_hash,
                size_bytes: doc_row.size_bytes as u64,
                storage_key: doc_row.storage_key,
                signature_fields: doc_row.signature_fields.0,
            },
            signers,
            config: row.config.0,
            metadata: CeremonyMetadata {
                title: row.title,
                description: row.description,
                reference: row.reference,
                tags: row.tags,
            },
            version: row.version,
            created_at: row.created_at.to_rfc3339(),
            updated_at: row.updated_at.to_rfc3339(),
            expires_at: row.expires_at.to_rfc3339(),
        };

        Ok(ceremony)
    }

    /// Update a ceremony (state, version, timestamps).
    ///
    /// Uses optimistic locking — fails if version doesn't match.
    ///
    /// # Errors
    /// Returns error if version mismatch or database operation fails.
    pub async fn update(&self, ceremony: &Ceremony) -> Result<(), OrchestratorError> {
        let state_before = ceremony.state_before_abort.map(|s| s.as_str().to_string());
        let updated_at = parse_rfc3339(&ceremony.updated_at)?;

        let result = sqlx::query(
            r#"
            UPDATE signing_ceremonies
            SET state = $1,
                state_before_abort = $2,
                version = $3,
                updated_at = $4
            WHERE id = $5 AND version = $6
            "#,
        )
        .bind(ceremony.state.as_str())
        .bind(&state_before)
        .bind(ceremony.version)
        .bind(updated_at)
        .bind(&ceremony.id.0)
        .bind(ceremony.version - 1) // Optimistic locking
        .execute(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to update ceremony: {e}")))?;

        if result.rows_affected() == 0 {
            return Err(OrchestratorError::DatabaseError(
                "Optimistic lock failure: ceremony was modified".to_string(),
            ));
        }

        Ok(())
    }

    /// List ceremonies for a tenant.
    ///
    /// # Errors
    /// Returns error if database operation fails.
    pub async fn list_by_tenant(
        &self,
        tenant_id: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Ceremony>, OrchestratorError> {
        let rows: Vec<CeremonyRow> = sqlx::query_as(
            r#"
            SELECT
                id, tenant_id, created_by, state, state_before_abort, version,
                ceremony_type, config, title, description, reference, tags,
                created_at, updated_at, expires_at
            FROM signing_ceremonies
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tenant_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to list ceremonies: {e}")))?;

        let mut ceremonies = Vec::with_capacity(rows.len());
        for row in rows {
            let id = CeremonyId(row.id.clone());
            let ceremony = self.get_by_id(&id).await?;
            ceremonies.push(ceremony);
        }

        Ok(ceremonies)
    }

    /// Get signers for a ceremony.
    async fn get_signers_for_ceremony(
        &self,
        ceremony_id: &str,
    ) -> Result<Vec<SignerSlot>, OrchestratorError> {
        use crate::domain::{
            SignatureData, SignerInvitation, SignerRole, SignerSlotId, SignerStatus,
        };

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

        let rows: Vec<SignerRow> = sqlx::query_as(
            r#"
            SELECT
                id, name, email, role, signing_order, is_required, status,
                invitation_token, invitation_sent_at, invitation_expires_at,
                webauthn_credential_id, assurance_level, signature_data,
                signed_at, decline_reason
            FROM signer_slots
            WHERE ceremony_id = $1
            ORDER BY signing_order ASC
            "#,
        )
        .bind(ceremony_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OrchestratorError::DatabaseError(format!("Failed to fetch signers: {e}")))?;

        let signers = rows
            .into_iter()
            .map(|row| {
                let invitation = row.invitation_token.map(|token| {
                    let sent_at: Option<String> = row.invitation_sent_at.map(|t| t.to_rfc3339());
                    let expires_at: String = row
                        .invitation_expires_at
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_default();
                    SignerInvitation {
                        token,
                        sent_at,
                        expires_at,
                        reminders_sent: 0, // Not stored in this query
                    }
                });

                let signed_at: Option<String> = row.signed_at.map(|t| t.to_rfc3339());

                SignerSlot {
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
                }
            })
            .collect();

        Ok(signers)
    }
}

/// Parse RFC 3339 timestamp string to `DateTime`.
fn parse_rfc3339(s: &str) -> Result<chrono::DateTime<chrono::Utc>, OrchestratorError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| OrchestratorError::DatabaseError(format!("Invalid timestamp: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rfc3339() {
        let ts = "2026-03-11T12:00:00Z";
        let result = parse_rfc3339(ts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_rfc3339_invalid() {
        let ts = "not-a-timestamp";
        let result = parse_rfc3339(ts);
        assert!(result.is_err());
    }
}
