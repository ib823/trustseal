# Sahi Build State
**Last updated:** 2026-03-10
**Current phase:** Phase 0 — IN PROGRESS
**Current module:** F2 (Tamper-Evident Log Engine) — COMPLETE
**Sprint:** Sprint 1-2 (Weeks 1-4)

---

## Status

Modules F1 (KMS) and F2 (Merkle log) are COMPLETE. 48 tests passing. Full workspace compiles with zero clippy warnings.

**Next action:** Begin Module F3 — API Gateway (Axum middleware stack: auth, rate limiting, metering, RLS setup).

---

## Phase 0 Module Checklist

- [x] F1 — Key Management Service (HSM/KMS abstraction) ✓ 24 tests
- [x] F2 — Tamper-Evident Log Engine (Merkle tree, proofs) ✓ 24 tests
- [ ] F3 — API Gateway, metering, rate limiting (Kong)
- [ ] F4 — Multi-tenant management, RLS, all DDL migrations
- [ ] F5 — Database, storage, DevOps infrastructure (Terraform, CI/CD, monitoring)
- [ ] F6 — Trust Registry

**Phase 0 prerequisites before Phase 1 starts:**
- [ ] Error Code Registry implemented (Appendix E)
- [ ] ULID convention enforced (Appendix G)
- [ ] i18n infrastructure in place (rust-i18n + next-intl)
- [ ] Time format enforcement (all timestamps as `timestamptz` / RFC 3339)
- [ ] STRIDE Threat Model document drafted
- [x] docker-compose.yml working (Postgres 16, Redis 7, Mosquitto 2)
- [ ] CI/CD pipeline running (lint, test, build gates)

---

## F1 Completion Summary

**Implemented:**
- `KmsProvider` trait — async, `Send + Sync`, 8 operations (generate, sign, verify, export, rotate, destroy, list, get_metadata)
- `SoftwareKmsProvider` — in-memory keys via `ring`, `RwLock<HashMap>` for thread safety
- `KeyAlgorithm` — Ed25519, ECDSA P-256
- `KeyState` lifecycle — Active → VerifyOnly → PendingDestruction → Destroyed
- `KeyRotationResult` — 30-day grace period for old key verification
- `DestroyConfirmation` — requires matching phrase "DESTROY {key_id}"
- `KmsAuditEvent` — emitted on every operation (success/failure), SAHI error codes
- `CryptoError` — typed errors with SAHI_XXXX codes (2001-2010)
- Zeroize on key material drop

**Tests (24 passing):**
- Key generation (Ed25519, ECDSA P-256, unique handles)
- Sign & verify roundtrip (Ed25519, ECDSA P-256)
- Tamper detection (modified data, wrong key)
- Public key export (both algorithms)
- Key rotation (state transition, verify-only, sign-blocked)
- Key destruction (valid/invalid confirmation, post-destroy operations blocked)
- Key listing (all, tenant-filtered)
- Audit event emission (success + failure events)
- Concurrent access (10 parallel keygen, 20 parallel sign/verify)

---

## Performance baselines (to be filled as measured)

| Target | Spec | Measured | Status |
|--------|------|----------|--------|
| BLE presentation | < 200ms | - | not tested |
| Gate entry E2E | < 2s | - | not tested |
| SD-JWT verify | < 50ms | - | not tested |
| TM signing overhead | < 1s | - | not tested |

---

## Session log

- 2026-03-09: Setup complete. Ready to begin Phase 0.
- 2026-03-10: Monorepo scaffolded. Rust workspace compiles. Clippy + tests pass. Docker-compose configured.
- 2026-03-10: Master plan v1.1 committed with gap analysis (50+ fixes across 24 sections).
- 2026-03-10: Agent personas adapted from agency-agents repo (Identity Trust, Security, Backend).
- 2026-03-10: F1 (KMS) COMPLETE — KmsProvider trait, SoftwareKmsProvider, audit events, 24 tests passing.
- 2026-03-10: F2 (Merkle) COMPLETE — MerkleTree, inclusion/consistency proofs, tamper detection, 24 tests passing.
