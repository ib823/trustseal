# Sahi Build State

Last updated: 2026-03-12
Current phase: Phase 3 / multi-track completion
Current objective: continue from the hardened backend baseline toward full pilot/dev completion
Authoritative continuation doc: `docs/CLAUDE_CODE_HANDOFF.md`

## Read First

1. `.claude/state/current.md`
2. `docs/CLAUDE_CODE_HANDOFF.md`
3. `docs/MYDIGITAL_TRACEABILITY_MATRIX.md`
4. `docs/MASTER_PLAN.md`
5. `docs/STRIDE_THREAT_MODEL.md`

## Current Status

Completed and materially improved:

- `platform-api` bearer auth and eKYC persistence/ID-token validation
- `crypto-engine` SD-JWT, DID, status-list, trust-registry primitives
- `edge/verifier` core trust-verification path and startup fixes
- root workspace build wiring
- `signing-service` management endpoints and auth foundation

Still incomplete:

- `signing-service` signer auth/sign flows
- `issuer-service`
- `verify-service`
- wallet FFI and real presentation generation
- verifier BLE/GATT/NFC/MQTT Android runtime
- frontend mock replacement
- DB-backed integration tests and real Playwright E2E
- real MyDigital registration and interoperability

## User-Approved Constraints

- Full platform scope remains in scope
- Use the approved execution order from the handoff doc
- Pilot/dev-only stack, free or very low cost
- `OIDC + PKCE + JWT/JWKS`
- `SD-JWT VC` only
- `did:key` and `did:web` only
- `roles + scopes + tenant_id`, default deny
- `OpenAPI-first` should become canonical
- verifier target is Android tablet/phone
- wallet target is Android and iOS
- formal ops/runbooks are deferred for now

## Immediate Blockers

Do not guess these:

1. Does JWT `sub` equal the platform user ID (`USR_...`)?
2. Are signer actions done by authenticated platform users, invitation-token holders, or both?
3. Is the dev-only file-backed keystore plus local JSON trust-registry model approved for issuer/verify services?
4. Which free-only providers should be used for notifications, storage, queue/broker, and metrics?
5. Real MyDigital credentials and registered redirect URIs still do not exist.

## Latest Implemented Slice

Shared auth context:

- `packages/sahi-core/src/auth.rs`

Signing-service foundation and management endpoints:

- `apps/signing-service/src/auth.rs`
- `apps/signing-service/src/state.rs`
- `apps/signing-service/src/routes/ceremonies.rs`
- `apps/signing-service/src/main.rs`

Domain correctness fixes:

- parsed timestamp expiry checks
- sequential versus parallel ceremony classification

Validation:

- `cargo test -p signing-service`
- `cargo clippy -p signing-service -- -D warnings`

## Environment Notes

- Disk space issue on `/workspaces` was resolved on 2026-03-12 by removing generated outputs only.
- Current free space is about `14G`.
- `target/` and `.next/` build outputs were removed, so expect cold rebuilds.
- Keep `node_modules` unless the user explicitly wants a deeper cleanup.

## Next Approved Execution Order

1. Wait for blocker answers.
2. Add route-level authorization using shared `AuthContext`.
3. Finish signing-service signer flows.
4. Build issuer-service and verify-service on the approved dev-only key/trust model.
5. Formalize OpenAPI contracts and generate clients.
6. Align wallet and frontends to the canonical contracts.
7. Replace mocks, add DB-backed tests, update Playwright, and wire CI.
