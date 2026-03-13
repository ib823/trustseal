use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use tokio::sync::RwLock;

use super::types::{
    AssuranceLevel, IdentityVerification, OAuthSession, VerificationProvider, VerificationStatus,
    VerifiedClaims,
};
use super::EkycError;

#[async_trait]
pub trait EkycStore: Send + Sync {
    async fn create_flow(
        &self,
        verification: &IdentityVerification,
        session: &OAuthSession,
    ) -> Result<(), EkycError>;

    async fn find_session_by_state(
        &self,
        tenant_id: &str,
        state: &str,
    ) -> Result<Option<OAuthSession>, EkycError>;

    async fn get_verification(
        &self,
        tenant_id: &str,
        verification_id: &str,
    ) -> Result<Option<IdentityVerification>, EkycError>;

    async fn mark_verified(
        &self,
        tenant_id: &str,
        verification_id: &str,
        claims: &VerifiedClaims,
        verified_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<IdentityVerification, EkycError>;

    async fn mark_failed(
        &self,
        tenant_id: &str,
        verification_id: &str,
        failure_reason: &str,
    ) -> Result<(), EkycError>;

    async fn bind_did(
        &self,
        tenant_id: &str,
        verification_id: &str,
        did: &str,
        bound_at: DateTime<Utc>,
    ) -> Result<IdentityVerification, EkycError>;

    async fn delete_session(&self, tenant_id: &str, session_id: &str) -> Result<(), EkycError>;

    /// Health check — verify database connectivity. Default returns Ok.
    async fn health_check(&self) -> Result<(), EkycError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct InMemoryEkycStore {
    verifications: RwLock<HashMap<String, IdentityVerification>>,
    sessions: RwLock<HashMap<String, OAuthSession>>,
}

impl InMemoryEkycStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl EkycStore for InMemoryEkycStore {
    async fn create_flow(
        &self,
        verification: &IdentityVerification,
        session: &OAuthSession,
    ) -> Result<(), EkycError> {
        self.verifications
            .write()
            .await
            .insert(verification.id.clone(), verification.clone());
        self.sessions
            .write()
            .await
            .insert(session.id.clone(), session.clone());
        Ok(())
    }

    async fn find_session_by_state(
        &self,
        tenant_id: &str,
        state: &str,
    ) -> Result<Option<OAuthSession>, EkycError> {
        Ok(self
            .sessions
            .read()
            .await
            .values()
            .find(|session| session.tenant_id == tenant_id && session.state == state)
            .cloned())
    }

    async fn get_verification(
        &self,
        tenant_id: &str,
        verification_id: &str,
    ) -> Result<Option<IdentityVerification>, EkycError> {
        Ok(self
            .verifications
            .read()
            .await
            .get(verification_id)
            .filter(|verification| verification.tenant_id == tenant_id)
            .cloned())
    }

    async fn mark_verified(
        &self,
        tenant_id: &str,
        verification_id: &str,
        claims: &VerifiedClaims,
        verified_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<IdentityVerification, EkycError> {
        let mut verifications = self.verifications.write().await;
        let verification = verifications
            .get_mut(verification_id)
            .filter(|verification| verification.tenant_id == tenant_id)
            .ok_or(EkycError::NotFound)?;
        verification.status = VerificationStatus::Verified;
        verification.assurance_level = claims.assurance_level;
        verification.name_hash = Some(claims.name_hash.clone());
        verification.ic_hash = Some(claims.ic_hash.clone());
        verification.verified_at = Some(verified_at);
        verification.expires_at = Some(expires_at);
        verification.failure_reason = None;
        verification.updated_at = Utc::now();
        Ok(verification.clone())
    }

    async fn mark_failed(
        &self,
        tenant_id: &str,
        verification_id: &str,
        failure_reason: &str,
    ) -> Result<(), EkycError> {
        let mut verifications = self.verifications.write().await;
        let verification = verifications
            .get_mut(verification_id)
            .filter(|verification| verification.tenant_id == tenant_id)
            .ok_or(EkycError::NotFound)?;
        verification.status = VerificationStatus::Failed;
        verification.failure_reason = Some(failure_reason.to_string());
        verification.updated_at = Utc::now();
        Ok(())
    }

    async fn bind_did(
        &self,
        tenant_id: &str,
        verification_id: &str,
        did: &str,
        bound_at: DateTime<Utc>,
    ) -> Result<IdentityVerification, EkycError> {
        let mut verifications = self.verifications.write().await;
        let verification = verifications
            .get_mut(verification_id)
            .filter(|verification| verification.tenant_id == tenant_id)
            .ok_or(EkycError::NotFound)?;
        verification.did = Some(did.to_string());
        verification.did_bound_at = Some(bound_at);
        verification.updated_at = Utc::now();
        Ok(verification.clone())
    }

    async fn delete_session(&self, tenant_id: &str, session_id: &str) -> Result<(), EkycError> {
        let mut sessions = self.sessions.write().await;
        match sessions.get(session_id) {
            Some(session) if session.tenant_id == tenant_id => {
                sessions.remove(session_id);
                Ok(())
            }
            _ => Err(EkycError::NotFound),
        }
    }
}

pub struct PostgresEkycStore {
    pool: PgPool,
}

impl PostgresEkycStore {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct DbVerificationRow {
    id: String,
    tenant_id: String,
    user_id: Option<String>,
    status: String,
    provider: String,
    assurance_level: String,
    name_hash: Option<String>,
    ic_hash: Option<String>,
    did: Option<String>,
    did_bound_at: Option<DateTime<Utc>>,
    verified_at: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    failure_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct DbSessionRow {
    id: String,
    tenant_id: String,
    verification_id: String,
    state: String,
    nonce: String,
    code_verifier: String,
    code_challenge: String,
    redirect_uri: String,
    scope: String,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl TryFrom<DbVerificationRow> for IdentityVerification {
    type Error = EkycError;

    fn try_from(row: DbVerificationRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            tenant_id: row.tenant_id,
            user_id: row.user_id,
            status: parse_status(&row.status)?,
            provider: parse_provider(&row.provider)?,
            assurance_level: parse_assurance(&row.assurance_level)?,
            name_hash: row.name_hash,
            ic_hash: row.ic_hash,
            did: row.did,
            did_bound_at: row.did_bound_at,
            verified_at: row.verified_at,
            expires_at: row.expires_at,
            failure_reason: row.failure_reason,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

impl From<DbSessionRow> for OAuthSession {
    fn from(row: DbSessionRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            verification_id: row.verification_id,
            state: row.state,
            nonce: row.nonce,
            code_verifier: row.code_verifier,
            code_challenge: row.code_challenge,
            redirect_uri: row.redirect_uri,
            scope: row.scope,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl EkycStore for PostgresEkycStore {
    async fn create_flow(
        &self,
        verification: &IdentityVerification,
        session: &OAuthSession,
    ) -> Result<(), EkycError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        set_tenant_context(&mut tx, &verification.tenant_id).await?;

        sqlx::query(
            r"INSERT INTO identity_verifications (
                id, tenant_id, user_id, status, provider, assurance_level,
                name_hash, ic_hash, did, did_bound_at, verified_at, expires_at,
                failure_reason, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11, $12,
                $13, $14, $15
            )",
        )
        .bind(&verification.id)
        .bind(&verification.tenant_id)
        .bind(&verification.user_id)
        .bind(verification.status.to_string())
        .bind(verification.provider.to_string())
        .bind(verification.assurance_level.to_string())
        .bind(&verification.name_hash)
        .bind(&verification.ic_hash)
        .bind(&verification.did)
        .bind(verification.did_bound_at)
        .bind(verification.verified_at)
        .bind(verification.expires_at)
        .bind(&verification.failure_reason)
        .bind(verification.created_at)
        .bind(verification.updated_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;

        sqlx::query(
            r"INSERT INTO oauth_sessions (
                id, tenant_id, verification_id, state, nonce, code_verifier,
                code_challenge, redirect_uri, scope, expires_at, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11
            )",
        )
        .bind(&session.id)
        .bind(&session.tenant_id)
        .bind(&session.verification_id)
        .bind(&session.state)
        .bind(&session.nonce)
        .bind(&session.code_verifier)
        .bind(&session.code_challenge)
        .bind(&session.redirect_uri)
        .bind(&session.scope)
        .bind(session.expires_at)
        .bind(session.created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        Ok(())
    }

    async fn find_session_by_state(
        &self,
        tenant_id: &str,
        state: &str,
    ) -> Result<Option<OAuthSession>, EkycError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        set_tenant_context(&mut tx, tenant_id).await?;

        let row = sqlx::query_as::<_, DbSessionRow>(
            r"SELECT
                id, tenant_id, verification_id, state, nonce, code_verifier,
                code_challenge, redirect_uri, scope, expires_at, created_at
            FROM oauth_sessions
            WHERE state = $1
            LIMIT 1",
        )
        .bind(state)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn get_verification(
        &self,
        tenant_id: &str,
        verification_id: &str,
    ) -> Result<Option<IdentityVerification>, EkycError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        set_tenant_context(&mut tx, tenant_id).await?;

        let row = sqlx::query_as::<_, DbVerificationRow>(
            r"SELECT
                id, tenant_id, user_id, status, provider, assurance_level,
                name_hash, ic_hash, did, did_bound_at, verified_at, expires_at,
                failure_reason, created_at, updated_at
            FROM identity_verifications
            WHERE id = $1
            LIMIT 1",
        )
        .bind(verification_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;

        row.map(TryInto::try_into).transpose()
    }

    async fn mark_verified(
        &self,
        tenant_id: &str,
        verification_id: &str,
        claims: &VerifiedClaims,
        verified_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<IdentityVerification, EkycError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        set_tenant_context(&mut tx, tenant_id).await?;

        let row = sqlx::query_as::<_, DbVerificationRow>(
            r"UPDATE identity_verifications
            SET status = 'verified',
                assurance_level = $2,
                name_hash = $3,
                ic_hash = $4,
                verified_at = $5,
                expires_at = $6,
                failure_reason = NULL
            WHERE id = $1
            RETURNING
                id, tenant_id, user_id, status, provider, assurance_level,
                name_hash, ic_hash, did, did_bound_at, verified_at, expires_at,
                failure_reason, created_at, updated_at",
        )
        .bind(verification_id)
        .bind(claims.assurance_level.to_string())
        .bind(&claims.name_hash)
        .bind(&claims.ic_hash)
        .bind(verified_at)
        .bind(expires_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        row.try_into()
    }

    async fn mark_failed(
        &self,
        tenant_id: &str,
        verification_id: &str,
        failure_reason: &str,
    ) -> Result<(), EkycError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        set_tenant_context(&mut tx, tenant_id).await?;

        sqlx::query(
            r"UPDATE identity_verifications
            SET status = 'failed', failure_reason = $2
            WHERE id = $1",
        )
        .bind(verification_id)
        .bind(failure_reason)
        .execute(&mut *tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        Ok(())
    }

    async fn bind_did(
        &self,
        tenant_id: &str,
        verification_id: &str,
        did: &str,
        bound_at: DateTime<Utc>,
    ) -> Result<IdentityVerification, EkycError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        set_tenant_context(&mut tx, tenant_id).await?;

        let row = sqlx::query_as::<_, DbVerificationRow>(
            r"UPDATE identity_verifications
            SET did = $2, did_bound_at = $3
            WHERE id = $1
            RETURNING
                id, tenant_id, user_id, status, provider, assurance_level,
                name_hash, ic_hash, did, did_bound_at, verified_at, expires_at,
                failure_reason, created_at, updated_at",
        )
        .bind(verification_id)
        .bind(did)
        .bind(bound_at)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        row.try_into()
    }

    async fn delete_session(&self, tenant_id: &str, session_id: &str) -> Result<(), EkycError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        set_tenant_context(&mut tx, tenant_id).await?;

        sqlx::query("DELETE FROM oauth_sessions WHERE id = $1")
            .bind(session_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        Ok(())
    }

    async fn health_check(&self) -> Result<(), EkycError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| EkycError::Database(e.to_string()))?;
        Ok(())
    }
}

async fn set_tenant_context(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: &str,
) -> Result<(), EkycError> {
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| EkycError::Database(e.to_string()))?;
    Ok(())
}

fn parse_status(value: &str) -> Result<VerificationStatus, EkycError> {
    match value {
        "pending" => Ok(VerificationStatus::Pending),
        "in_progress" => Ok(VerificationStatus::InProgress),
        "verified" => Ok(VerificationStatus::Verified),
        "failed" => Ok(VerificationStatus::Failed),
        "expired" => Ok(VerificationStatus::Expired),
        _ => Err(EkycError::Database(format!(
            "Unknown verification status: {value}"
        ))),
    }
}

fn parse_provider(value: &str) -> Result<VerificationProvider, EkycError> {
    match value {
        "mydigital_id" => Ok(VerificationProvider::MydigitalId),
        "manual" => Ok(VerificationProvider::Manual),
        _ => Err(EkycError::Database(format!(
            "Unknown verification provider: {value}"
        ))),
    }
}

fn parse_assurance(value: &str) -> Result<AssuranceLevel, EkycError> {
    match value {
        "P1" => Ok(AssuranceLevel::P1),
        "P2" => Ok(AssuranceLevel::P2),
        "P3" => Ok(AssuranceLevel::P3),
        _ => Err(EkycError::Database(format!(
            "Unknown assurance level: {value}"
        ))),
    }
}

pub type SharedEkycStore = Arc<dyn EkycStore>;
