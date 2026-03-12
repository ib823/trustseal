-- TM-1: Signing Ceremonies Database Schema
--
-- Tables for TrustMark signing ceremony management.
-- State machine: CREATED -> PREPARING -> READY_FOR_SIGNATURES -> ...
-- -> FULLY_SIGNED -> TIMESTAMPING -> AUGMENTING_LTV -> COMPLETE

-- UP Migration

-- Signing ceremonies table
CREATE TABLE IF NOT EXISTS signing_ceremonies (
    id VARCHAR(30) PRIMARY KEY,  -- CER_ prefix
    tenant_id VARCHAR(30) NOT NULL REFERENCES tenants(id),
    created_by VARCHAR(30) NOT NULL,  -- USR_ prefix

    -- State machine
    state VARCHAR(30) NOT NULL DEFAULT 'CREATED',
    state_before_abort VARCHAR(30),  -- For resume functionality
    version BIGINT NOT NULL DEFAULT 1,  -- Optimistic locking

    -- Ceremony type
    ceremony_type VARCHAR(30) NOT NULL DEFAULT 'single_signer',

    -- Configuration (JSON)
    config JSONB NOT NULL DEFAULT '{
        "ttl_hours": 72,
        "require_all_signers": true,
        "allow_decline": true,
        "send_reminders": true,
        "min_assurance_level": "P1"
    }'::jsonb,

    -- Metadata
    title VARCHAR(255) NOT NULL,
    description TEXT,
    reference VARCHAR(100),  -- External reference number
    tags TEXT[] DEFAULT '{}',

    -- Merkle log integration
    merkle_log_id VARCHAR(30),  -- LOG_ prefix

    -- Timestamps (RFC 3339)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT valid_state CHECK (state IN (
        'CREATED', 'PREPARING', 'READY_FOR_SIGNATURES', 'SIGNING_IN_PROGRESS',
        'PARTIALLY_SIGNED', 'FULLY_SIGNED', 'TIMESTAMPING',
        'AUGMENTING_LTV', 'COMPLETE', 'ABORTED', 'RESUMING'
    )),
    CONSTRAINT valid_ceremony_type CHECK (ceremony_type IN (
        'single_signer', 'multi_signer_sequential', 'multi_signer_parallel', 'batch'
    ))
);

-- Ceremony documents table
CREATE TABLE IF NOT EXISTS ceremony_documents (
    id VARCHAR(30) PRIMARY KEY,  -- DOC_ prefix
    ceremony_id VARCHAR(30) NOT NULL REFERENCES signing_ceremonies(id) ON DELETE CASCADE,
    tenant_id VARCHAR(30) NOT NULL REFERENCES tenants(id),

    -- Document info
    filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL DEFAULT 'application/pdf',
    content_hash VARCHAR(64) NOT NULL,  -- SHA-256 hex
    signed_hash VARCHAR(64),  -- Hash after signing
    size_bytes BIGINT NOT NULL,
    storage_key VARCHAR(500) NOT NULL,  -- S3 key

    -- PAdES level progression
    pades_level VARCHAR(10),  -- NULL, 'B-T', 'B-LT', 'B-LTA'

    -- Signature field coordinates (JSON array)
    signature_fields JSONB DEFAULT '[]'::jsonb,

    -- Timestamps
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    signed_at TIMESTAMPTZ,

    CONSTRAINT valid_pades_level CHECK (pades_level IS NULL OR pades_level IN ('B-T', 'B-LT', 'B-LTA'))
);

-- Signer slots table
CREATE TABLE IF NOT EXISTS signer_slots (
    id VARCHAR(30) PRIMARY KEY,  -- SLT_ prefix
    ceremony_id VARCHAR(30) NOT NULL REFERENCES signing_ceremonies(id) ON DELETE CASCADE,
    tenant_id VARCHAR(30) NOT NULL REFERENCES tenants(id),

    -- Signer info
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    phone VARCHAR(50),

    -- Role and ordering
    role VARCHAR(20) NOT NULL DEFAULT 'signatory',
    signing_order INTEGER NOT NULL DEFAULT 1,
    is_required BOOLEAN NOT NULL DEFAULT TRUE,

    -- Status sub-state machine
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING',

    -- Invitation
    invitation_token VARCHAR(64),  -- For invitation URL
    invitation_sent_at TIMESTAMPTZ,
    invitation_expires_at TIMESTAMPTZ,
    reminders_sent INTEGER NOT NULL DEFAULT 0,

    -- Authentication
    webauthn_credential_id VARCHAR(500),
    assurance_level VARCHAR(5),  -- P1, P2, P3
    authenticated_at TIMESTAMPTZ,

    -- Signature
    signature_data JSONB,  -- CMS signature, cert chain, etc.
    signed_at TIMESTAMPTZ,
    decline_reason TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT valid_role CHECK (role IN ('signatory', 'witness', 'approver', 'notary')),
    CONSTRAINT valid_status CHECK (status IN ('PENDING', 'INVITED', 'AUTHENTICATED', 'SIGNED', 'DECLINED', 'EXPIRED')),
    CONSTRAINT valid_assurance_level CHECK (assurance_level IS NULL OR assurance_level IN ('P1', 'P2', 'P3'))
);

-- Ceremony state transitions log (for audit)
CREATE TABLE IF NOT EXISTS ceremony_transitions (
    id VARCHAR(30) PRIMARY KEY,  -- TRN_ prefix
    ceremony_id VARCHAR(30) NOT NULL REFERENCES signing_ceremonies(id) ON DELETE CASCADE,
    tenant_id VARCHAR(30) NOT NULL REFERENCES tenants(id),

    -- Transition details
    from_state VARCHAR(30) NOT NULL,
    to_state VARCHAR(30) NOT NULL,
    reason TEXT NOT NULL,
    actor VARCHAR(30) NOT NULL,  -- USR_ or SVC_ prefix

    -- Merkle log
    merkle_log_id VARCHAR(30),  -- LOG_ prefix

    -- Timestamp
    transitioned_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_ceremonies_tenant ON signing_ceremonies(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ceremonies_state ON signing_ceremonies(state);
CREATE INDEX IF NOT EXISTS idx_ceremonies_created_by ON signing_ceremonies(created_by);
CREATE INDEX IF NOT EXISTS idx_ceremonies_expires_at ON signing_ceremonies(expires_at);

CREATE INDEX IF NOT EXISTS idx_ceremony_docs_ceremony ON ceremony_documents(ceremony_id);
CREATE INDEX IF NOT EXISTS idx_ceremony_docs_tenant ON ceremony_documents(tenant_id);

CREATE INDEX IF NOT EXISTS idx_signer_slots_ceremony ON signer_slots(ceremony_id);
CREATE INDEX IF NOT EXISTS idx_signer_slots_tenant ON signer_slots(tenant_id);
CREATE INDEX IF NOT EXISTS idx_signer_slots_email ON signer_slots(email);
CREATE INDEX IF NOT EXISTS idx_signer_slots_token ON signer_slots(invitation_token);

CREATE INDEX IF NOT EXISTS idx_ceremony_transitions_ceremony ON ceremony_transitions(ceremony_id);

-- Row Level Security
ALTER TABLE signing_ceremonies ENABLE ROW LEVEL SECURITY;
ALTER TABLE signing_ceremonies FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_ceremonies ON signing_ceremonies
    USING (tenant_id = current_setting('app.tenant_id', true));

ALTER TABLE ceremony_documents ENABLE ROW LEVEL SECURITY;
ALTER TABLE ceremony_documents FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_docs ON ceremony_documents
    USING (tenant_id = current_setting('app.tenant_id', true));

ALTER TABLE signer_slots ENABLE ROW LEVEL SECURITY;
ALTER TABLE signer_slots FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_signers ON signer_slots
    USING (tenant_id = current_setting('app.tenant_id', true));

ALTER TABLE ceremony_transitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE ceremony_transitions FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_transitions ON ceremony_transitions
    USING (tenant_id = current_setting('app.tenant_id', true));

-- Updated_at trigger
CREATE OR REPLACE FUNCTION update_ceremony_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_ceremonies_updated_at
    BEFORE UPDATE ON signing_ceremonies
    FOR EACH ROW
    EXECUTE FUNCTION update_ceremony_updated_at();

CREATE TRIGGER trigger_signer_slots_updated_at
    BEFORE UPDATE ON signer_slots
    FOR EACH ROW
    EXECUTE FUNCTION update_ceremony_updated_at();

-- DOWN Migration (commented out for safety)
-- DROP TABLE IF EXISTS ceremony_transitions;
-- DROP TABLE IF EXISTS signer_slots;
-- DROP TABLE IF EXISTS ceremony_documents;
-- DROP TABLE IF EXISTS signing_ceremonies;
