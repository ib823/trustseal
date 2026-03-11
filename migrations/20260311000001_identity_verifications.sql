-- ─── VP-9: IDENTITY VERIFICATIONS TABLE ───────────────────────────────────
-- Stores eKYC verification status for users/residents
-- MyDigital ID integration via OAuth 2.0 with PKCE
-- IMPORTANT: No raw PII stored - only verification status and hashed identifiers

CREATE TABLE IF NOT EXISTS identity_verifications (
    id              TEXT PRIMARY KEY,           -- IDV_ ULID
    tenant_id       TEXT NOT NULL REFERENCES tenants(id),
    user_id         TEXT REFERENCES users(id),  -- nullable for pre-registration

    -- Verification status
    status          TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'in_progress', 'verified', 'failed', 'expired')),
    provider        TEXT NOT NULL DEFAULT 'mydigital_id'
                    CHECK (provider IN ('mydigital_id', 'manual')),

    -- Assurance level per TrustMark spec (P1/P2/P3)
    assurance_level TEXT NOT NULL DEFAULT 'P1'
                    CHECK (assurance_level IN ('P1', 'P2', 'P3')),

    -- OAuth state (encrypted, short-lived)
    oauth_state     TEXT,                       -- PKCE state parameter
    code_verifier   TEXT,                       -- PKCE code verifier (encrypted at rest)

    -- Verified claims (hashed, never raw PII)
    name_hash       TEXT,                       -- SHA-256 of normalized name
    ic_hash         TEXT,                       -- SHA-256 of IC number

    -- DID binding (after wallet key generation)
    did             TEXT,                       -- did:key:z6Mk... bound after verification
    did_bound_at    TIMESTAMPTZ,

    -- Audit trail
    verified_at     TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ,                -- Verification validity period
    failure_reason  TEXT,

    -- Timestamps
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE identity_verifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_verifications FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON identity_verifications
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_verifications_tenant ON identity_verifications(tenant_id);
CREATE INDEX idx_verifications_user ON identity_verifications(user_id);
CREATE INDEX idx_verifications_status ON identity_verifications(tenant_id, status);
CREATE INDEX idx_verifications_oauth_state ON identity_verifications(oauth_state) WHERE oauth_state IS NOT NULL;
CREATE INDEX idx_verifications_did ON identity_verifications(did) WHERE did IS NOT NULL;

-- ─── PKCE SESSIONS TABLE ──────────────────────────────────────────────────
-- Short-lived OAuth sessions for PKCE flow
-- Auto-cleaned after 10 minutes

CREATE TABLE IF NOT EXISTS oauth_sessions (
    id              TEXT PRIMARY KEY,           -- OAS_ ULID
    tenant_id       TEXT NOT NULL REFERENCES tenants(id),
    verification_id TEXT NOT NULL REFERENCES identity_verifications(id) ON DELETE CASCADE,

    -- PKCE parameters
    state           TEXT NOT NULL UNIQUE,
    code_verifier   TEXT NOT NULL,              -- Base64URL-encoded
    code_challenge  TEXT NOT NULL,              -- SHA256(code_verifier), Base64URL-encoded

    -- Session metadata
    redirect_uri    TEXT NOT NULL,
    scope           TEXT NOT NULL DEFAULT 'openid profile',

    -- Expiration (10 minutes)
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '10 minutes'),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE oauth_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_sessions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON oauth_sessions
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_oauth_sessions_state ON oauth_sessions(state);
CREATE INDEX idx_oauth_sessions_expires ON oauth_sessions(expires_at);

-- ─── CLEANUP FUNCTION ─────────────────────────────────────────────────────
-- Auto-delete expired OAuth sessions

CREATE OR REPLACE FUNCTION cleanup_expired_oauth_sessions()
RETURNS void AS $$
BEGIN
    DELETE FROM oauth_sessions WHERE expires_at < NOW();
END;
$$ LANGUAGE plpgsql;

-- ─── TRIGGER: UPDATE TIMESTAMP ────────────────────────────────────────────

CREATE OR REPLACE FUNCTION update_identity_verification_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_identity_verifications_updated
    BEFORE UPDATE ON identity_verifications
    FOR EACH ROW
    EXECUTE FUNCTION update_identity_verification_timestamp();

-- ─── DOWN MIGRATION ───────────────────────────────────────────────────────
-- To rollback: DROP TABLE oauth_sessions; DROP TABLE identity_verifications;
