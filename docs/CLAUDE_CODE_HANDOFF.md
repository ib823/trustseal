# Claude Code Continuation Guide

Last updated: 2026-03-12
Audience: Claude Code CLI and human maintainers
Purpose: authoritative continuation packet for the current implementation state, decisions, blockers, and next execution order

## Read Order

1. `.claude/state/current.md`
2. `docs/CLAUDE_CODE_HANDOFF.md`
3. `docs/MYDIGITAL_TRACEABILITY_MATRIX.md`
4. `docs/MASTER_PLAN.md`
5. `docs/STRIDE_THREAT_MODEL.md`

## Non-Negotiable Working Rules

- Do not assume missing product, identity, or infrastructure facts.
- Do not revert unrelated worktree changes.
- Use the existing approved execution order unless the user changes it explicitly.
- Treat this repo as a shared, dirty worktree.
- Treat `docs/CLAUDE_CODE_HANDOFF.md` and `.claude/state/current.md` as the current handoff source of truth.

## User-Approved Decisions

| Area | Approved Decision | Notes |
| --- | --- | --- |
| Scope | All apps, services, wallet, verifier, and frontends are in scope | Full platform scope |
| Execution order | `authz/contracts -> signing-service -> issuer/verify -> wallet/verifier runtime -> frontend integration -> E2E/CI -> ops docs` | Explicitly approved |
| Delivery posture | Pilot/dev-only stack, free or very low cost | Not public-production grade |
| Identity standards | `OIDC + PKCE + JWT/JWKS` | MyDigital-compatible direction |
| Credential format | `SD-JWT VC only` at launch | Additional formats deferred unless explicitly requested |
| DID methods | `did:key` and `did:web` only at launch | `did:peer` deferred |
| Authorization model | `roles + scopes + tenant_id`, default deny | Roles approved by user |
| Default roles | `platform_admin`, `tenant_admin`, `issuer_operator`, `verifier_operator`, `guard_operator`, `resident_user`, `guest_user`, `service_internal` | Approved |
| API contracts | `OpenAPI-first` should be canonical | Not yet fully implemented |
| Signing formats | `PAdES` for PDF and `JAdES` for JSON/API payloads | Timestamping, LTV, and strong audit desired |
| Verifier target | Android tablet/phone verifier | No Raspberry Pi assumption |
| Wallet targets | Android and iOS | User said "both"; no exact OS minimum supplied |
| Real MyDigital credentials | Not available yet | Approved fallback is local/staging Keycloak mirror |
| Ops/runbook work | Deferred out of scope for now | Code/platform work remains in scope |

## Current High-Level State

The repo is materially ahead of the original audit baseline, but it is not functionally complete. The strongest completed slices are the `platform-api` auth/eKYC stack, the `crypto-engine` SD-JWT and DID primitives, the `edge/verifier` trust-verification core, the root build wiring, and the management half of `signing-service`.

The major remaining gaps are:

- signer-facing signing flows and final signing semantics
- `issuer-service` and `verify-service`
- route-level authorization beyond tenant extraction
- wallet FFI and real credential presentation
- verifier BLE/NFC/MQTT runtime implementation on Android target
- frontend replacement of mock APIs
- Playwright and DB-backed integration coverage
- real MyDigital registration and interoperability testing

## Work Completed So Far

### Platform API

Completed:

- issuer/JWKS-backed bearer validation with discovery support
- explicit HS256 legacy/dev fallback
- tenant extraction from validated bearer claims
- eKYC persistence layer with `identity_verifications` and `oauth_sessions`
- PKCE, `state`, `nonce`, ID token validation, `at_hash`, and UserInfo `sub` matching
- persisted callback, status, and DID bind flows
- root service startup wiring and CORS tightening

Primary files:

- `apps/platform-api/src/main.rs`
- `apps/platform-api/src/middleware/tenant.rs`
- `apps/platform-api/src/routes/ekyc.rs`
- `apps/platform-api/src/services/ekyc/mod.rs`
- `apps/platform-api/src/services/ekyc/mydigital_id.rs`
- `apps/platform-api/src/services/ekyc/store.rs`
- `apps/platform-api/src/state.rs`
- `migrations/20260311000001_identity_verifications.sql`
- `migrations/20260311000003_oauth_sessions_nonce.sql`

Validation already observed in-session:

- `cargo test -p platform-api`
- `cargo clippy -p platform-api --tests -- -D warnings`

Still missing in this area:

- route-level role/scope authorization policy
- DB-backed integration tests for the Postgres store
- real MyDigital staging/prod credentials and UAT validation

### Crypto Engine

Completed:

- SD-JWT issuance, holder binding, and verifier path
- Ed25519 and P-256 holder key-binding interoperability
- `sd_hash` validation
- DID primitives and resolver improvements
- status-list and trust-registry primitives
- compliance/rules primitives already present

Primary files:

- `packages/crypto-engine/src/sd_jwt/holder.rs`
- `packages/crypto-engine/src/sd_jwt/issuer.rs`
- `packages/crypto-engine/src/sd_jwt/verifier.rs`
- `packages/crypto-engine/src/did/*`
- `packages/crypto-engine/src/status_list/*`
- `packages/crypto-engine/src/trust_registry/*`
- `packages/crypto-engine/src/compliance/*`

Validation already observed in-session:

- `cargo test -p crypto-engine`

Still missing in this area:

- persistent free-stack key management for issuer/verify services
- trust registry publication and persistence strategy in app services

### Edge Verifier

Completed:

- fixed SQLite WAL init bug
- fixed display initialization deadlock
- real issuer signature verification and holder key-binding verification in verifier core
- DID resolution improvements
- revocation sync path improvements

Primary files:

- `edge/verifier/src/audit/log.rs`
- `edge/verifier/src/hardware/display.rs`
- `edge/verifier/src/crypto/sd_jwt.rs`
- `edge/verifier/src/crypto/did_resolver.rs`
- `edge/verifier/src/revocation/sync.rs`

Validation already observed in-session:

- `cargo test -p edge-verifier`

Still missing in this area:

- BLE advertising/runtime for Android verifier target
- GATT service runtime
- NFC reader/APDU runtime
- MQTT runtime
- final verifier device integration model

### Workspace and Build

Completed:

- fixed root workspace metadata so `pnpm run build` resolves correctly
- fixed TypeScript package declarations enough for root build
- CI migration application now covers the actual migration set
- cleaned build artifacts on 2026-03-12 to recover disk space

Primary files:

- `package.json`
- `pnpm-workspace.yaml`
- `turbo.json`
- `.github/workflows/ci.yml`

Validation already observed in-session:

- `pnpm run build`

Environment note:

- build artifacts were removed to recover disk: repo `target/`, wallet Rust `target/`, and `.next` folders were deleted
- current free disk after cleanup is about `14G`

### Signing Service

Completed in the latest slice:

- real `AppState` and DB pool wiring
- bearer auth middleware aligned with platform-api style
- shared auth roles/context added to `sahi-core`
- DB-backed management handlers for:
  - `POST /api/v1/ceremonies`
  - `GET /api/v1/ceremonies/:id`
  - `POST /api/v1/ceremonies/:id/prepare`
  - `POST /api/v1/ceremonies/:id/ready`
  - `POST /api/v1/ceremonies/:id/abort`
  - `POST /api/v1/ceremonies/:id/resume`
- secure signer invitation token generation
- fixed ceremony expiry logic to use parsed timestamps, not string comparison
- fixed ceremony type classification so sequential and parallel are distinguished

Primary files:

- `apps/signing-service/src/main.rs`
- `apps/signing-service/src/auth.rs`
- `apps/signing-service/src/state.rs`
- `apps/signing-service/src/routes/ceremonies.rs`
- `apps/signing-service/src/domain/ceremony.rs`
- `apps/signing-service/src/domain/signer.rs`
- `apps/signing-service/src/orchestrator/service.rs`
- `packages/sahi-core/src/auth.rs`
- `packages/sahi-core/src/lib.rs`

Validation re-run after implementation:

- `cargo test -p signing-service`
- `cargo clippy -p signing-service -- -D warnings`

Still missing in this area:

- signer-facing authentication and sign submission endpoints
- invitation-token versus authenticated-user signer model
- actual TSA and LTV provider strategy
- JSON/JAdES and PDF/PAdES finalization pipeline

## Blockers That Must Be Answered Before Proceeding Further

These are unresolved and should not be guessed by Claude Code.

### Blocker 1: Subject Mapping

Question:

- Does JWT `sub` equal the platform user ID, for example `USR_...`?
- Or is there a separate user-mapping table/service between identity provider subject and platform user ID?

Why it matters:

- route-level self-service authorization
- signer authorization
- wallet user actions
- eKYC self-binding and DID ownership checks

### Blocker 2: Signer Identity Model

Question:

- Are signer actions performed by authenticated platform users, by invitation-token holders, or by both?

Why it matters:

- shape of `authenticate_signer` and `sign` endpoints
- invitation validation model
- authorization rules
- signer session design

### Blocker 3: Dev-Only Key and Trust Model

Question:

- Is the following temporary dev-only model approved for `issuer-service` and `verify-service`?
  - file-backed local keystore for issuer keys
  - env-configured issuer DID and governance DID
  - local JSON trust-registry file published by service

Why it matters:

- current repo only has in-memory software KMS plus trust-registry types
- free-stack pilot mode still requires persistence across restarts

### Blocker 4: Free-Only External Services

User constraint:

- external systems should be free where possible

Still unresolved:

- notification provider
- object/document storage provider
- queue/broker provider
- audit/metrics export provider

Important:

- the previously suggested AWS defaults are not free for sustained use
- do not silently wire paid managed services under the current constraint

### Blocker 5: Real MyDigital Credentials

Current state:

- no registered MyDigital staging or production credentials

Approved fallback:

- use local/staging Keycloak that mirrors MyDigital OIDC semantics

Do not assume:

- discovery URL
- client ID
- redirect URIs
- logout URIs
- issuer

## Pending Work Matrix

| Track | Status | Blocking Inputs | Next Concrete Work | Acceptance Criteria |
| --- | --- | --- | --- | --- |
| Route-level authorization | `Partial` | Blocker 1 | Add shared route guards using `AuthContext`, roles, and scopes to `platform-api`, `signing-service`, then new services | Protected routes enforce default deny with explicit role/scope checks |
| Signing-service signer actions | `Partial` | Blockers 1-2 | Implement signer auth/sign endpoints, invitation resolution, signer self-access rules, audit transitions | Signer can authenticate and sign under approved identity model; tests cover success and denial cases |
| Issuer-service | `Missing` | Blockers 1, 3, 4 | Build OpenAPI-backed issuance API on crypto-engine issuance/compliance/status-list primitives | Credentials can be issued persistently with trust and revocation data |
| Verify-service | `Missing` | Blockers 1, 3, 4 | Build remote verification API, trust-registry loading, DID/key resolution, revocation checks | Service returns policy decision and disclosed claim subset with tests |
| Wallet bearer auth alignment | `Partial` | Blocker 1 | Replace body-posted `tenant_id`, add bearer token handling, align client contract to backend | Wallet API calls use current auth contract |
| Wallet crypto FFI | `Missing` | None beyond current scope | Implement actual Rust bridge calls and presentation construction | Wallet can create presentation payloads and no longer throws `UnimplementedError` |
| Wallet presentation flow | `Missing` | Depends on FFI | Replace `Uint8List(0)` simulation with real presentation | End-to-end holder presentation succeeds |
| Android verifier runtime | `Missing` | Hardware/API model still implied | Implement BLE/GATT/NFC/APDU/MQTT runtime for Android target | Verifier performs real local verification, not stubs |
| Frontend mock replacement | `Missing` | Depends on service contracts | Replace admin/guest/guard mock data with real API clients generated from contracts | UI reads and mutates live backend state |
| OpenAPI contracts | `Missing` | None | Author OpenAPI 3.1 for platform, signing, issuer, verify services and generate clients | Contracts become the canonical source of truth |
| DB-backed integration tests | `Partial` | None | Add integration tests for eKYC store, signing repos/routes, issuer/verify persistence | Critical persistence flows tested against Postgres |
| Playwright E2E | `Partial` | Depends on real API flows | Rewrite stale specs to current contracts, add full user journey coverage, run in CI | CI executes E2E and catches contract drift |
| MyDigital interoperability | `Missing` | Blocker 5 | Replace Keycloak mirror with real provider config and UAT path | Real provider flow works end to end |
| Ops artifacts | `Deferred` | User explicitly deferred | Do not implement in current phase | N/A |

## Exact Next Execution Order For Claude Code

Do not skip ahead while blockers remain unresolved.

### Step 1

Wait for answers to:

- subject mapping
- signer identity model
- dev-only key/trust model
- free-only external service/provider choices

### Step 2

Once subject mapping is approved:

- add shared authz guards in `platform-api`
- reuse same `AuthContext` shape in all new services
- make route-level authorization explicit and test it

### Step 3

Once signer model is approved:

- finish `authenticate_signer`
- finish `sign`
- add signer invitation lookup or self-user enforcement
- add ceremony transition and signer-state tests

### Step 4

Once dev key/trust model is approved:

- implement `issuer-service`
- implement `verify-service`
- publish local trust-registry JSON
- persist issuer keys outside process memory

### Step 5

After service contracts are stable:

- author OpenAPI specs
- generate TS/Dart clients
- update admin/guest/guard/wallet clients to those contracts

### Step 6

Then:

- replace frontend mocks
- implement wallet FFI and real presentation path
- port verifier runtime from stubs toward Android device implementation

### Step 7

Then:

- add DB-backed integration tests
- rewrite Playwright suites to real contracts
- run them in CI

## Validation History

The following were green at different points in the current session history:

- `cargo test -p platform-api`
- `cargo clippy -p platform-api --tests -- -D warnings`
- `cargo test -p crypto-engine`
- `cargo test -p edge-verifier`
- `pnpm run build`
- `cargo test -p signing-service`
- `cargo clippy -p signing-service -- -D warnings`

The only validations re-run after the 2026-03-12 disk cleanup were:

- `cargo test -p signing-service`
- `cargo clippy -p signing-service -- -D warnings`

## Current Known Mismatches and Risks

- `apps/wallet/lib/services/crypto_service.dart` still throws `UnimplementedError`
- `apps/wallet/lib/features/credentials/screens/presentation_screen.dart` still simulates presentation with empty bytes
- `apps/wallet/lib/services/ekyc_service.dart` is still not aligned to the current bearer-auth contract
- `edge/verifier` still contains runtime stubs for BLE/GATT/NFC/MQTT on the device side
- `admin-portal`, `guest-web`, and `guard-tablet` still contain mock API/data paths
- Playwright tests are stale and not representative of the hardened platform flows
- no real MyDigital client registration exists yet
- free-only stack constraint remains incompatible with a serious public-production posture; current target remains pilot/dev-only

## Files Added or Changed in the Latest Documented Slice

Shared auth:

- `packages/sahi-core/src/auth.rs`
- `packages/sahi-core/src/lib.rs`

Signing-service:

- `apps/signing-service/Cargo.toml`
- `apps/signing-service/src/main.rs`
- `apps/signing-service/src/auth.rs`
- `apps/signing-service/src/state.rs`
- `apps/signing-service/src/routes/ceremonies.rs`
- `apps/signing-service/src/domain/ceremony.rs`
- `apps/signing-service/src/domain/signer.rs`
- `apps/signing-service/src/orchestrator/service.rs`

## Practical Continuation Notes

- The repository had a disk-space incident on 2026-03-12. Generated outputs were cleaned. Expect cold builds.
- `target/` is gone. Recompilation cost is expected.
- `node_modules` was intentionally kept to avoid reinstallation cost.
- Worktree is dirty. Do not use destructive git cleanup commands.

## Historical Documents

These remain useful, but they are not the current continuation source of truth:

- `docs/EXHAUSTIVE_AUDIT_PHASE2.md`
- `docs/SECURITY_AUDIT_PHASE2.md`

They describe earlier audit phases, not the current end-to-end delivery state.
