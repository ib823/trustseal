# Phase 2 Security Audit Report

Historical note: this document records the Phase 2 security audit state as of 2026-03-11.
For the current continuation status and pending execution order, use
`docs/CLAUDE_CODE_HANDOFF.md` and `.claude/state/current.md`.

**Date:** 2026-03-11
**Auditor:** Claude Code (automated)
**Scope:** VaultPass Hardening (Phase 2)

---

## Summary

| Check | Status | Notes |
|-------|--------|-------|
| cargo audit | PASS (with accepted risks) | 1 low-impact transitive vuln |
| No .unwrap() in production | PASS | All unwraps in test code |
| No PII in logs | PASS | No name/email/IC logged |
| RLS enabled | PASS | All tenant tables have RLS |
| Input validation | PASS | Axum typed extractors |
| Error responses | PASS | No internal details exposed |

---

## Cargo Audit Results

### RUSTSEC-2023-0071: RSA Marvin Attack (ACCEPTED RISK)

**Crate:** rsa 0.9.10
**Severity:** Medium (5.9)
**Status:** No fix available

**Why this is accepted:**
1. The `rsa` crate is a transitive dependency via `sqlx-mysql`
2. We use PostgreSQL exclusively (not MySQL)
3. We do not use RSA for any cryptographic operations:
   - SD-JWT signing: Ed25519
   - PAdES signing: ECDSA P-256
   - COSE tokens: ES256 (P-256)
4. The vulnerability requires RSA decryption operations which our code never performs
5. The dependency is pulled in by `sqlx-macros-core` for compile-time query verification

**Mitigation:** The vulnerable code path is never executed in our application.

### RUSTSEC-2025-0134: rustls-pemfile Unmaintained (WARNING)

**Crate:** rustls-pemfile 2.2.0
**Status:** Warning (not a vulnerability)

**Why this is accepted:**
1. Transitive dependency via `rumqttc` (MQTT client for edge-verifier)
2. Only affects PEM file parsing for certificate loading
3. Edge verifier uses this for MQTT TLS connection
4. Will be resolved when rumqttc updates its dependencies

**Mitigation:** Monitor for rumqttc updates.

---

## Production Code Analysis

### No .unwrap() in Production Code

Verified all `.unwrap()` calls are in test modules:
- `apps/platform-api/src/routes/ekyc.rs`: All unwraps after line 300 (`#[cfg(test)]`)
- `packages/crypto-engine/`: All unwraps in `#[cfg(test)]` or `#[test]` blocks

### No PII in Logs

Verified no logging of:
- Names (resident, visitor, user)
- Email addresses
- IC numbers / passport numbers
- Phone numbers

All identifiers in logs use opaque ULIDs:
- `KEY_01HXK...` for keys
- `IDV_01HXK...` for identity verifications
- `CRD_01HXK...` for credentials

### Error Response Security

Verified error responses use generic messages:
- SAHI error codes (e.g., `SAHI_5001`) without internal stack traces
- User-friendly messages without system paths or SQL details

---

## Database Security

### Row-Level Security

All tenant-scoped tables have RLS enabled and forced:
- `tenants`
- `users`
- `properties`
- `credentials`
- `identity_verifications`
- `oauth_sessions`
- `gate_events`
- `verifiers`
- `keys`
- `merkle_log`
- `ceremonies`
- `ceremony_signers`

Verified with policy: `USING (tenant_id = current_setting('app.tenant_id', true))`

---

## Performance Benchmarks

All performance targets met or exceeded:

| Operation | Target | Measured | Margin |
|-----------|--------|----------|--------|
| SD-JWT verification | < 50ms | 75 µs | 666x |
| SD-JWT parse + verify | < 50ms | 79 µs | 632x |
| Ed25519 sign | - | 96 µs | - |
| Ed25519 verify | - | 105 µs | - |
| Merkle proof verify | - | 1.6 µs | - |

---

## Recommendations

1. **Monitor sqlx updates** for a version that allows disabling mysql in macros
2. **Monitor rumqttc updates** for rustls-pemfile replacement
3. **Add SAST scanning** to CI (e.g., cargo-clippy with security lints)
4. **Schedule external pen test** per Phase 2 requirements
