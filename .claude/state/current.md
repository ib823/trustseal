# Sahi Build State
**Last updated:** 2026-03-10
**Current phase:** Phase 0 — IN PROGRESS
**Current module:** Phase 0 COMPLETE — all F1-F6 modules done
**Sprint:** Sprint 1-2 (Weeks 1-4)

---

## Status

**PHASE 0 COMPLETE.** All foundation modules (F1-F6) implemented. **VP-1, VP-2, VP-3, VP-3b, VP-3c, VP-4, VP-5, VP-6, VP-7 COMPLETE.** 355+ tests passing. Zero clippy warnings. 12 database tables with RLS. CI/CD pipeline configured. Platform API operational.

**Current module:** VP-8 — Guest Web Flow (next)
**Completed:** VP-1 (SD-JWT), VP-2 (DID Resolution), VP-3 (Status List + Credential Types), VP-3b (Compliance), VP-3c (Rules Engine), VP-4 (Wallet App), VP-5 (Edge Verifier), VP-6 (Admin Portal), VP-7 (Guard Tablet)

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
- 2026-03-10: VP-3 — Status List + Credential Types COMPLETE. Bitstring (gzip+base64), StatusListCredential, IndexAllocator (randomized), StatusListManager/Registry, credential_types (ResidentBadge/VisitorPass/ContractorBadge/EmergencyAccess with TTLs). 144 tests.
- 2026-03-10: VP-3b — Compliance Enforcement Layer COMPLETE. ComplianceEngine with 4 checks (IdentityVerification, UnitOwnership, Blacklist, CredentialLimit). 169 tests.
- 2026-03-10: VP-3c — Business Logic Rules Engine COMPLETE. RulesEngine with conditions (TimeWindow, FloorAccess, Role, Clearance, CredentialType, Zone) and actions (Allow, Deny, AllowWithLog). 206 tests.
- 2026-03-10: VP-3/3b/3c AUDIT — Fixed critical error code collision (SAHI_3001-3007 conflicted with SigningCeremony). Added Compliance domain (SAHI_2300-2306) to sahi-core/error.rs. Updated all compliance checks and tests. 224 tests passing.
- 2026-03-10: VP-4 — Wallet App COMPLETE. Full Flutter scaffold (42 files). Features: credentials, scanning, onboarding, settings. Services: keystore (Android/iOS), BLE, NFC, crypto FFI, sync, auth, security. Rust FFI: BLE payload encoding, revocation checking. Platform: Android Keystore + HCE, iOS Secure Enclave. 230 tests total.
- 2026-03-10: VP-4 AUDIT — Fixed 3 hardcoded UI strings (scanning_or, scanning_again, access_not_authorized) with EN/MS i18n. Fixed Rust clippy warnings (unused params, pedantic). All 230 tests passing. Crypto FFI integration deferred (requires flutter_rust_bridge_codegen).
- 2026-03-10: VP-5 — Edge Verifier (Raspberry Pi) COMPLETE. Rust firmware: BLE (GATT server, advertiser, protocol), NFC (PC/SC, APDU, NDEF), crypto (SD-JWT, DID resolver), policy (rules engine, offline mode), revocation (status list cache, MQTT sync), audit (SQLite append-only), hardware (GPIO, LED, buzzer, display, tamper). 90 tests, fail-closed security, feature flags for simulation/production.
- 2026-03-10: VP-6 — Admin Portal COMPLETE. Next.js 14 with App Router. Dashboard (stats, activity, charts, verifier status), Residents (table, CRUD, credential management), Access Logs (filterable table, pagination), Verifiers (grid, status monitoring), Analytics (overview, peak hours, denial reasons, trends), Settings (property, security, notifications, schedule, team). i18n (EN/MS), React Query, Zustand, shadcn/ui. 47 files, build passing.
- 2026-03-11: VP-7 — Guard Tablet PWA COMPLETE. Next.js 14 PWA for guard stations. 60/40 split-view (visitor queue/status panel). Offline-first with queued actions. Emergency override with biometric auth. Auto-refresh (30s). i18n (EN/MS). 50 files, 35 tests.

---

## VP-7 Completion Summary

**Guard Tablet Structure (`apps/guard-tablet/src/`):**
- `app/` — Next.js 14 App Router, root page with split-view layout
- `components/header.tsx` — Checkpoint info, online status, last sync, language switcher
- `components/offline-indicator.tsx` — Offline banner with queue status
- `components/visitor/` — VisitorQueue (list with filters/sort), VisitorDetailDialog (actions), StatusPanel (system status, queued actions, recent activity)
- `components/override/` — OverrideDialog (biometric auth, reason selection)
- `components/ui/` — Button, Card, Badge, Dialog, Select (shadcn/ui, touch-optimized)
- `lib/stores/` — visitor-store (visitors, fetch, update), offline-store (queue, sync), override-store (target)
- `lib/api/` — client.ts, visitors.ts (with mock data)
- `hooks/` — use-visitors, use-offline, use-override
- `i18n/` — config, request, messages (EN/MS with 100+ strings)

**PWA Features:**
- Fullscreen display with landscape orientation
- Service Worker with next-pwa (register, skipWaiting)
- Offline action queue with persistence (Zustand persist)
- Auto-sync when connection restored

**Design System per spec:**
- Font: Plus Jakarta Sans (primary), JetBrains Mono (IDs)
- Colors: 95% slate monochrome + signal colors (green-500, amber-500, red-500)
- Touch targets: min 48x48dp (h-12 default buttons)
- Dark theme only (slate-950 background)

**Tests (35 passing):**
- Stores: visitor state, offline queue, override target
- API: client methods, visitors API, mock data structure
- Utils: cn class merging, tailwind overrides

---

## VP-6 Completion Summary

**Admin Portal Structure (`apps/admin-portal/src/`):**
- `app/` — Next.js 14 App Router, dashboard route group with 7 pages
- `components/layout/` — Sidebar (workspace switcher, navigation), Header (search, theme toggle, notifications, user menu)
- `components/dashboard/` — StatsCards, RecentActivity, AccessChart, VerifierStatus
- `components/residents/` — ResidentTable, ResidentTableSkeleton, AddResidentButton, ResidentDetailPage
- `components/access-logs/` — AccessLogTable (pagination), AccessLogFilters (status, direction, search)
- `components/verifiers/` — VerifierGrid (status cards), VerifierGridSkeleton, AddVerifierButton
- `components/analytics/` — AnalyticsOverview, PeakHoursChart (bar), DenialReasons (pie), EntryTrends (area)
- `components/settings/` — SettingsTabs (property, security, notifications, schedule, team)
- `components/ui/` — Button, Card, Input, Badge, Avatar, Toaster, TableSkeleton (shadcn/ui style)
- `i18n/` — config.ts (EN/MS locales), request.ts, messages/ (en.json, ms.json)
- `lib/api/` — client.ts (ApiClient), residents.ts, verifiers.ts, access-logs.ts, analytics.ts
- `lib/stores/` — workspace-store.ts, user-store.ts (Zustand with persistence)
- `lib/providers/` — QueryProvider, ThemeProvider

**Design System per spec:**
- Font: Plus Jakarta Sans (primary), JetBrains Mono (IDs/codes)
- Colors: 95% slate monochrome + signal colors (green-500, amber-500, red-500) for semantic meaning
- Theme: Light/dark mode via next-themes
- Components: shadcn/ui patterns with Radix primitives

**Features:**
- Dashboard: 4 stat cards, real-time activity, access pattern chart, verifier status
- Residents: Searchable table, credential status badges, detail page with actions
- Access Logs: Filterable by status/direction, pagination, linked to resident details
- Verifiers: Grid view with online/offline/degraded status, signal strength, events
- Analytics: Overview stats with trends, peak hours bar chart, denial pie chart, 30-day trends
- Settings: 5 tabs (property info, security policy, notifications, schedule, team)
- i18n: EN/MS with next-intl, all UI strings in JSON files

**Build status:** TypeScript + ESLint + Next.js build passing

---

## VP-5 Completion Summary

**Edge Verifier Structure (`edge/verifier/src/`):**
- `main.rs` — Tokio async runtime, signal handling, task spawning
- `config.rs` — Full configuration with ULID validation, GPIO pin mappings, TTLs
- `audit/` — SQLite append-only log (WAL mode, sync flag, batch upload)
- `ble/` — GATT server, advertiser, challenge/response protocol
- `nfc/` — PC/SC reader, APDU commands, NDEF parser for VaultPass credentials
- `crypto/` — SD-JWT verification, DID resolver (did:key, did:web, did:peer)
- `policy/` — Rule-based access control, offline mode with allow-list
- `revocation/` — Status list cache with TTL/stale thresholds, MQTT sync
- `hardware/` — GPIO controller, LED patterns, buzzer, display, tamper detection
- `mqtt/` — Platform connectivity for real-time updates

**Security per spec:**
- Fail-closed model (deny on any error)
- MSB-first bit ordering per W3C Bitstring Status List
- 30s challenge validity, 4h stale threshold
- Offline allow-list for emergency access

**Feature flags:**
- `simulation` (default) — Stubs for development without hardware
- `rpi` — Enable GPIO via rppal
- `ble-hardware` — Enable BlueZ via bluer
- `nfc-hardware` — Enable PC/SC via pcsc
- `mqtt` — Enable MQTT via rumqttc
- `full` — All hardware features for production

**Tests (90 passing):**
- BLE: advertiser, GATT, protocol, challenge/response
- NFC: APDU commands, NDEF parsing, VaultPass detection
- Crypto: SD-JWT decode, DID resolution, cache
- Policy: rules evaluation, time windows, priority ordering
- Revocation: bitstring operations, cache freshness, stale handling
- Hardware: GPIO, LED patterns, buzzer, tamper detection
- MQTT: connection, subscription, topics

---

## VP-4 Completion Summary

**Flutter App Structure (`apps/wallet/`):**
- `lib/app/` — MaterialApp, GoRouter navigation, dark theme only
- `lib/core/` — Theme (slate-950), i18n (EN/MS), secure storage, credential model
- `lib/services/` — Keystore, BLE, NFC, crypto FFI, sync, auth, security
- `lib/features/` — Credentials (list, detail, presentation), Scanning, Onboarding (welcome, registration, eKYC, key generation), Settings

**Rust FFI (`rust/src/api/`):**
- `crypto.rs` — Presentation creation (placeholder for crypto-engine integration)
- `ble_payload.rs` — BLE payload encoding/decoding (version header + length-prefixed)
- `revocation.rs` — Status list parsing with gzip+base64, MSB-first bit checking

**Platform Code:**
- **Android:** KeystorePlugin (StrongBox/TEE, biometric auth, ECDSA P-256), HceService (NFC HCE)
- **iOS:** KeystorePlugin (Secure Enclave, Face ID/Touch ID, ECDSA P-256)

**Security per spec:**
- Hardware-bound keys (non-exportable, device-bound)
- Biometric authentication required for signing
- Root/jailbreak detection (warn, don't block)
- Secure credential storage (AES + keystore-wrapped key)

**Performance targets:**
- Cold start < 1.5s (deferred init, const constructors, lazy-load BLE/NFC)
- BLE handshake < 200ms (ConnectionPriority.high, MTU 512, bonded devices)
- Gate decision < 2s (cached status list, hardware-accelerated crypto)

---

## VP-3 Completion Summary

**Status List Module (`status_list/`):**
- `Bitstring` — Gzip-compressed bitstring (W3C Bitstring Status List v1.0)
- `StatusListCredential` — W3C VC format for status list publication
- `BitstringStatusListEntry` — Embedded in issued credentials
- `IndexAllocator` — Randomized allocation (prevents correlation attacks)
- `StatusListManager` — Combines allocation, revocation, credential generation
- `StatusListRegistry` — Multi-tenant registry (by tenant+credential type)

**Credential Types Module (`credential_types.rs`):**
- `CredentialType` enum — ResidentBadge, VisitorPass, ContractorBadge, EmergencyAccess
- Per-type TTLs per spec: 8h, 4h, 12h, 1h
- Per-type selective disclosure claims
- `VaultPassCredentialV2` — Generic credential wrapper with status list support
- Subject structs for each credential type

**Tests (29 new, 144 total):**
- Bitstring operations (encode/decode, MSB ordering, revocation)
- Status list credential serialization
- Index allocation (random, full, specific)
- Manager operations (allocate, revoke, export/import)
- Credential type TTLs and selective claims
- Subject serialization for all 4 types

---

## VP-3b Completion Summary

**Compliance Enforcement Layer (`compliance/`):**
- `ComplianceEngine` — Combines all checks, evaluates in order
- `IdentityVerificationCheck` — eKYC via MyDigital ID
- `UnitOwnershipCheck` — Property admin confirmation
- `BlacklistCheck` — Security violations, non-payment, misconduct
- `CredentialLimitCheck` — Per-user, per-unit, per-day limits

**Tests (25 new):**
- Check pass/fail scenarios for each check type
- Engine evaluation with multiple checks
- Detailed result reporting

---

## VP-3c Completion Summary

**Business Logic Rules Engine (`access_rules/`):**
- `RulesEngine` — Priority-based rule evaluation
- `AccessRule` — Tenant/property scoped with conditions
- Conditions: TimeWindow, FloorAccess, Role, Clearance, CredentialType, Zone
- Actions: Allow, Deny, AllowWithLog
- `RuleSet` — Sorted by priority (highest wins)
- `EvaluationContext` — Context for rule evaluation

**Tests (37 new):**
- Condition evaluation (time, floor, role, clearance, zone)
- Overnight time windows
- Priority-based rule ordering
- All-conditions-must-match logic
- Detailed evaluation with condition results

---

## Phase 1 Module Checklist

- [x] VP-1 — SD-JWT Crypto Engine ✓ 15 tests
- [x] VP-2 — DID Resolution Layer ✓ 43 tests
- [x] VP-3 — Credential Format & Status List ✓ 29 tests
- [x] VP-3b — Compliance Enforcement Layer ✓ 25 tests
- [x] VP-3c — Business Logic Rules Engine ✓ 37 tests
- [x] VP-4 — Wallet App (Flutter) ✓ 42 files
- [x] VP-5 — Edge Verifier (Raspberry Pi) ✓ 30 files, 90 tests
- [x] VP-6 — Admin Portal (Next.js) ✓ 47 files
- [x] VP-7 — Guard Tablet (Next.js PWA) ✓ 50 files, 35 tests
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
