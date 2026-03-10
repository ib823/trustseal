# Sahi Build State
**Last updated:** 2026-03-10
**Current phase:** Phase 0 — IN PROGRESS
**Current module:** Phase 0 COMPLETE — all F1-F6 modules done
**Sprint:** Sprint 1-2 (Weeks 1-4)

---

## Status

**PHASE 0 COMPLETE.** All foundation modules (F1-F6) implemented. **VP-1 COMPLETE. VP-2 COMPLETE (audited).** 126 tests passing. Zero clippy warnings. 12 database tables with RLS. CI/CD pipeline configured. Platform API operational.

**Current module:** VP-3 — Credential Format & Status List (next)
**Completed:** VP-1 (SD-JWT), VP-2 (DID Resolution)

---

## Phase 0 Module Checklist

- [x] F1 — Key Management Service (HSM/KMS abstraction) ✓ 24 tests
- [x] F2 — Tamper-Evident Log Engine (Merkle tree, proofs) ✓ 24 tests
- [x] F3 — API Gateway (Axum middleware stack, rate limiting, tenant extraction)
- [x] F4 — Multi-tenant management, RLS, all DDL migrations ✓ 12 tables
- [x] F5 — CI/CD pipeline (GitHub Actions: fmt, clippy, test, audit, integration, security)
- [x] F6 — Trust Registry (TrustRegistry, TrustEntry, authorization checks)

**Phase 0 prerequisites before Phase 1 starts:**
- [x] Error Code Registry implemented (Appendix E) ✓ 38 codes across 11 domains in sahi-core
- [x] ULID convention enforced (Appendix G) ✓ TypedUlid with 17 prefixes, parse/validate/serialize
- [ ] i18n infrastructure in place (rust-i18n + next-intl) — deferred to Phase 1 UI work
- [x] Time format enforcement (all timestamps as `timestamptz` / RFC 3339) ✓ sahi-core::time module
- [x] STRIDE Threat Model document drafted ✓ docs/STRIDE_THREAT_MODEL.md
- [x] docker-compose.yml working (Postgres 16, Redis 7, Mosquitto 2)
- [x] CI/CD pipeline configured (GitHub Actions: fmt, clippy, test, audit, integration, security)

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
- 2026-03-10: F3 (Gateway) COMPLETE — Axum middleware chain (request ID, tenant extraction, rate limiting, CORS, compression, tracing), /health endpoint, graceful shutdown.
- 2026-03-10: F4 (Tenants) COMPLETE — 12 tables (tenants, users, properties, keys, merkle_log, credentials, verifiers, gate_events, ceremonies, ceremony_signers, products, merkle_tree_state). RLS + FORCE RLS on all tenant-scoped tables. Merkle log append-only triggers.
- 2026-03-10: F5 (CI/CD) COMPLETE — GitHub Actions workflow (rust, typescript, integration, security jobs).
- 2026-03-10: F6 (Trust Registry) COMPLETE — TrustRegistry data model, authorization checks, issuer revocation.
- 2026-03-10: **PHASE 0 COMPLETE** — All foundation modules operational.
- 2026-03-10: Phase 0 prerequisites — sahi-core crate (error codes, ULID factory, time utils), STRIDE threat model. 66 tests total.
- 2026-03-10: VP-1 — SD-JWT module implemented (issuer, holder, verifier with KMS integration). VaultPassCredential types. 81 tests total.
- 2026-03-10: Fixed SAHI_1200 (RateLimitExceeded) — added to ErrorCode enum, updated rate_limit.rs to use SahiError. 38 error codes, 11 domains.
- 2026-03-10: VP-2 — DID Resolution Layer COMPLETE. did:key (Ed25519/P-256), did:web (URL mapping, validation), LRU cache. 43 DID tests, 124 total.
- 2026-03-10: VP-2 AUDIT — Fixed TTL (5m for L1), P-256 compressed point rejection, added 2 tests. Redis/PostgreSQL L2/L3 deferred to hardening. 126 tests.

---

## Phase 1 Module Checklist

- [x] VP-1 — SD-JWT Crypto Engine ✓ 15 tests
- [x] VP-2 — DID Resolution Layer ✓ 43 tests
- [ ] VP-3 — Credential Format & Status List
- [ ] VP-3b — Compliance Enforcement Layer
- [ ] VP-3c — Business Logic Rules Engine
- [ ] VP-4 — Wallet App (Flutter)
- [ ] VP-5 — Edge Verifier (Raspberry Pi)
- [ ] VP-6 — Admin Portal (Next.js)
- [ ] VP-7 — Guard Tablet (Next.js PWA)
- [ ] VP-8 — Guest Web Flow
- [ ] VP-9 — Onboarding + KYC flows

---

## VP-2 Completion Summary

**Implemented:**
- `Did` type with parsing (method, path, query, fragment)
- `DidDocument` following W3C DID Core 1.0 (context, verification methods, services)
- `VerificationMethod` with JWK/multibase/base58 public key extraction
- `did:key` resolver (Ed25519, P-256, X25519, secp256k1 multicodec)
- `did:web` URL mapping (domain, port, path handling per spec)
- `did:peer` stub (Phase 2)
- `DidCache` — LRU cache with TTL, statistics, Redis key helpers
- `DidResolver` — sync resolver for did:key with caching
- `AsyncDidResolver` — async resolver for did:web (requires http feature)

**Tests (43 passing):**
- DID parsing (key, web, peer, fragments)
- did:key resolution (Ed25519, roundtrip, relationships)
- did:web URL conversion (domain, port, path, localhost)
- Document validation
- Cache operations (insert, get, evict, expire, stats)
- Resolver caching behavior
