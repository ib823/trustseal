# Sahi — Completed Modules

Format: `- [x] MODULE_ID — YYYY-MM-DD — brief description`

## Phase 0 — Shared Foundation

- [x] F1 — 2026-03-10 — KMS abstraction (KmsProvider trait, SoftwareKmsProvider, audit events)
- [x] F2 — 2026-03-10 — Merkle tree (tamper-evident log, inclusion/consistency proofs)
- [x] F3 — 2026-03-10 — API Gateway (Axum middleware: request ID, tenant, rate limit, CORS)
- [x] F4 — 2026-03-10 — Multi-tenant DB (12 tables, RLS, append-only triggers)
- [x] F5 — 2026-03-10 — CI/CD (GitHub Actions: fmt, clippy, test, audit, security)
- [x] F6 — 2026-03-10 — Trust Registry (TrustRegistry data model, authorization)

## Phase 1 — VaultPass

- [x] VP-1 — 2026-03-10 — SD-JWT Crypto Engine (issuer, holder, verifier, KMS integration)
- [x] VP-2 — 2026-03-10 — DID Resolution Layer (did:key, did:web, LRU cache with TTL)
- [x] VP-3 — 2026-03-10 — Credential Format & Status List (bitstring, allocator, manager, credential types)
- [x] VP-3b — 2026-03-10 — Compliance Enforcement Layer (IdentityVerification, UnitOwnership, Blacklist, CredentialLimit)
- [x] VP-3c — 2026-03-10 — Business Logic Rules Engine (TimeWindow, FloorAccess, Role, Clearance, Zone conditions)
- [x] VP-4 — 2026-03-10 — Wallet App (Flutter scaffold, BLE/NFC services, keystore Android/iOS, Rust FFI)
- [x] VP-5 — 2026-03-10 — Edge Verifier (Raspberry Pi firmware, BLE/NFC, SD-JWT, policy engine, fail-closed)
- [x] VP-6 — 2026-03-10 — Admin Portal (Next.js 14, dashboard, residents, logs, verifiers, analytics, settings, i18n)
- [x] VP-7 — 2026-03-11 — Guard Tablet PWA (Next.js 14, split-view, offline queue, override, i18n, 35 tests)

## Phase 2 — VaultPass Hardening

(none yet)

## Phase 3 — TrustMark

(none yet)

## Phase 4 — TrustMark Hardening

(none yet)
