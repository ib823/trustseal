-- ─── VP-9: OAUTH SESSION NONCE COLUMN ────────────────────────────────────
-- Stores OIDC nonce alongside PKCE session material so the ID token can be
-- validated during the callback flow.

ALTER TABLE oauth_sessions
    ADD COLUMN IF NOT EXISTS nonce TEXT;

UPDATE oauth_sessions
SET nonce = state
WHERE nonce IS NULL;

ALTER TABLE oauth_sessions
    ALTER COLUMN nonce SET NOT NULL;
