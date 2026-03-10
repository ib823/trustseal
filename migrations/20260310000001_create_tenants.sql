-- F4: Multi-tenant management
-- Creates the tenants table, RLS infrastructure, and tenant lifecycle management.

-- ─── TENANT TABLE ───────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS tenants (
    id          TEXT PRIMARY KEY,           -- TNT_ ULID
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,       -- URL-safe identifier
    state       TEXT NOT NULL DEFAULT 'provisioning'
                CHECK (state IN ('provisioning', 'active', 'suspended', 'terminated')),
    tier        TEXT NOT NULL DEFAULT 'free'
                CHECK (tier IN ('free', 'standard', 'enterprise')),
    config      JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    suspended_at TIMESTAMPTZ,
    terminated_at TIMESTAMPTZ
);

CREATE INDEX idx_tenants_slug ON tenants(slug);
CREATE INDEX idx_tenants_state ON tenants(state) WHERE state = 'active';

-- ─── USERS TABLE ────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY,           -- USR_ ULID
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    email       TEXT NOT NULL,
    name        TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'user'
                CHECK (role IN ('owner', 'admin', 'operator', 'guard', 'user')),
    state       TEXT NOT NULL DEFAULT 'active'
                CHECK (state IN ('active', 'suspended', 'deleted')),
    password_hash TEXT,                    -- Argon2id hash (nullable for passkey-only)
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE users FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON users
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_users_tenant ON users(tenant_id);
CREATE INDEX idx_users_email ON users(tenant_id, email);

-- ─── PROPERTIES TABLE ──────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS properties (
    id          TEXT PRIMARY KEY,           -- PRY_ ULID
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    name        TEXT NOT NULL,
    address     TEXT,
    config      JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE properties ENABLE ROW LEVEL SECURITY;
ALTER TABLE properties FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON properties
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_properties_tenant ON properties(tenant_id);

-- ─── KEYS METADATA TABLE ───────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS keys (
    id          TEXT PRIMARY KEY,           -- KEY_ ULID
    tenant_id   TEXT REFERENCES tenants(id),
    algorithm   TEXT NOT NULL CHECK (algorithm IN ('Ed25519', 'ECDSA-P256')),
    label       TEXT NOT NULL,
    state       TEXT NOT NULL DEFAULT 'active'
                CHECK (state IN ('active', 'verify_only', 'pending_destruction', 'destroyed')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at  TIMESTAMPTZ,
    expires_at  TIMESTAMPTZ
);

ALTER TABLE keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE keys FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON keys
    USING (tenant_id = current_setting('app.tenant_id', true)
           OR tenant_id IS NULL); -- Platform keys have no tenant

CREATE INDEX idx_keys_tenant ON keys(tenant_id);
CREATE INDEX idx_keys_state ON keys(state) WHERE state = 'active';

-- ─── MERKLE LOG TABLE ──────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS merkle_log_entries (
    id          TEXT PRIMARY KEY,           -- LOG_ ULID
    sequence    BIGINT NOT NULL UNIQUE,     -- Monotonic, gapless
    timestamp   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type  TEXT NOT NULL,
    payload_hash BYTEA NOT NULL,           -- SHA-256 (32 bytes)
    previous_root BYTEA NOT NULL,          -- SHA-256 (32 bytes)
    new_root    BYTEA NOT NULL,            -- SHA-256 (32 bytes)
    tenant_id   TEXT REFERENCES tenants(id)
);

ALTER TABLE merkle_log_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE merkle_log_entries FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON merkle_log_entries
    USING (tenant_id = current_setting('app.tenant_id', true)
           OR tenant_id IS NULL);

-- Append-only constraint: prevent UPDATE and DELETE
CREATE OR REPLACE FUNCTION prevent_modify_merkle_log()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Merkle log entries are append-only: UPDATE and DELETE are prohibited';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER merkle_log_no_update
    BEFORE UPDATE ON merkle_log_entries
    FOR EACH ROW EXECUTE FUNCTION prevent_modify_merkle_log();

CREATE TRIGGER merkle_log_no_delete
    BEFORE DELETE ON merkle_log_entries
    FOR EACH ROW EXECUTE FUNCTION prevent_modify_merkle_log();

CREATE INDEX idx_merkle_log_sequence ON merkle_log_entries(sequence);
CREATE INDEX idx_merkle_log_tenant ON merkle_log_entries(tenant_id);

-- ─── MERKLE TREE STATE ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS merkle_tree_state (
    id          INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1), -- Singleton
    root_hash   BYTEA NOT NULL DEFAULT '\x0000000000000000000000000000000000000000000000000000000000000000',
    sequence    BIGINT NOT NULL DEFAULT 0,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO merkle_tree_state (id) VALUES (1) ON CONFLICT DO NOTHING;

-- ─── CREDENTIALS TABLE (VaultPass) ─────────────────────────────────────

CREATE TABLE IF NOT EXISTS credentials (
    id          TEXT PRIMARY KEY,           -- CRD_ ULID
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    holder_did  TEXT NOT NULL,
    issuer_did  TEXT NOT NULL,
    credential_type TEXT NOT NULL
                CHECK (credential_type IN ('ResidentBadge', 'VisitorPass', 'ContractorBadge', 'EmergencyAccess')),
    key_id      TEXT NOT NULL REFERENCES keys(id),
    status      TEXT NOT NULL DEFAULT 'active'
                CHECK (status IN ('active', 'revoked', 'expired', 'suspended')),
    status_list_index INTEGER,
    issued_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at  TIMESTAMPTZ NOT NULL,
    revoked_at  TIMESTAMPTZ,
    metadata    JSONB NOT NULL DEFAULT '{}'
);

ALTER TABLE credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE credentials FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON credentials
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_credentials_tenant ON credentials(tenant_id);
CREATE INDEX idx_credentials_holder ON credentials(tenant_id, holder_did);
CREATE INDEX idx_credentials_status ON credentials(status) WHERE status = 'active';
CREATE INDEX idx_credentials_expiry ON credentials(expires_at) WHERE status = 'active';

-- ─── VERIFIERS TABLE (Edge devices) ────────────────────────────────────

CREATE TABLE IF NOT EXISTS verifiers (
    id          TEXT PRIMARY KEY,           -- VRF_ ULID
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    property_id TEXT NOT NULL REFERENCES properties(id),
    name        TEXT NOT NULL,
    device_did  TEXT,
    state       TEXT NOT NULL DEFAULT 'provisioning'
                CHECK (state IN ('provisioning', 'active', 'offline', 'lockdown', 'decommissioned')),
    last_heartbeat TIMESTAMPTZ,
    config      JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE verifiers ENABLE ROW LEVEL SECURITY;
ALTER TABLE verifiers FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON verifiers
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_verifiers_tenant ON verifiers(tenant_id);
CREATE INDEX idx_verifiers_property ON verifiers(property_id);

-- ─── GATE EVENTS TABLE ─────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS gate_events (
    id          TEXT PRIMARY KEY,           -- EVT_ ULID
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    verifier_id TEXT NOT NULL REFERENCES verifiers(id),
    credential_id TEXT REFERENCES credentials(id),
    decision    TEXT NOT NULL CHECK (decision IN ('granted', 'denied')),
    reason      TEXT,
    timestamp   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata    JSONB NOT NULL DEFAULT '{}'
);

ALTER TABLE gate_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE gate_events FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON gate_events
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_gate_events_tenant ON gate_events(tenant_id);
CREATE INDEX idx_gate_events_verifier ON gate_events(verifier_id);
CREATE INDEX idx_gate_events_timestamp ON gate_events(timestamp);

-- ─── SIGNING CEREMONIES TABLE (TrustMark) ──────────────────────────────

CREATE TABLE IF NOT EXISTS ceremonies (
    id          TEXT PRIMARY KEY,           -- CRM_ ULID
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    state       TEXT NOT NULL DEFAULT 'created'
                CHECK (state IN (
                    'created', 'preparing', 'ready_for_signatures',
                    'signing_in_progress', 'partially_signed', 'fully_signed',
                    'timestamping', 'augmenting_ltv', 'complete', 'aborted'
                )),
    document_hash BYTEA,                   -- SHA-256 of the document
    signing_order TEXT NOT NULL DEFAULT 'sequential'
                CHECK (signing_order IN ('sequential', 'parallel')),
    expires_at  TIMESTAMPTZ NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1, -- Optimistic locking
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata    JSONB NOT NULL DEFAULT '{}'
);

ALTER TABLE ceremonies ENABLE ROW LEVEL SECURITY;
ALTER TABLE ceremonies FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ceremonies
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_ceremonies_tenant ON ceremonies(tenant_id);
CREATE INDEX idx_ceremonies_state ON ceremonies(state) WHERE state NOT IN ('complete', 'aborted');

-- ─── CEREMONY SIGNERS TABLE ────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS ceremony_signers (
    id          TEXT PRIMARY KEY,           -- SIG_ ULID
    ceremony_id TEXT NOT NULL REFERENCES ceremonies(id),
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    user_id     TEXT REFERENCES users(id),
    email       TEXT NOT NULL,
    role        TEXT NOT NULL DEFAULT 'signer',
    state       TEXT NOT NULL DEFAULT 'pending'
                CHECK (state IN ('pending', 'invited', 'authenticated', 'signed', 'declined')),
    signed_at   TIMESTAMPTZ,
    order_index INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE ceremony_signers ENABLE ROW LEVEL SECURITY;
ALTER TABLE ceremony_signers FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON ceremony_signers
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_ceremony_signers_ceremony ON ceremony_signers(ceremony_id);

-- ─── PRODUCTS TABLE (TrustMark labels) ─────────────────────────────────

CREATE TABLE IF NOT EXISTS products (
    id          TEXT PRIMARY KEY,           -- PRD_ ULID
    tenant_id   TEXT NOT NULL REFERENCES tenants(id),
    name        TEXT NOT NULL,
    certification_ref TEXT,
    batch_id    TEXT,                       -- BTH_ ULID
    cose_token  BYTEA,                     -- Signed COSE_Sign1 token
    nfc_tag_uid TEXT,                      -- NTAG 424 DNA UID
    state       TEXT NOT NULL DEFAULT 'active'
                CHECK (state IN ('active', 'revoked', 'expired')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata    JSONB NOT NULL DEFAULT '{}'
);

ALTER TABLE products ENABLE ROW LEVEL SECURITY;
ALTER TABLE products FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON products
    USING (tenant_id = current_setting('app.tenant_id', true));

CREATE INDEX idx_products_tenant ON products(tenant_id);
CREATE INDEX idx_products_batch ON products(batch_id);

-- ─── UPDATED_AT TRIGGER ────────────────────────────────────────────────

CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON properties
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
CREATE TRIGGER set_updated_at BEFORE UPDATE ON ceremonies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();
