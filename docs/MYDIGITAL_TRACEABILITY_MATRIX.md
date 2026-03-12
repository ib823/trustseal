# MyDigital ID and OIDC Traceability Matrix

Last updated: 2026-03-12

## Scope

This matrix maps the MyDigital ID SSO Integration Guideline v5.0 and core OIDC/OAuth
requirements to the current TrustSeal codebase. It is intended to drive implementation
order, auditability, and regression review.

Primary sources:

- MyDigital ID SSO Integration Guideline v5.0:
  https://github.com/MyIDSSO/SSO-Integration-Guideline/blob/branch-v5.0/Guideline-v5.0.md
- OpenID Connect Core 1.0:
  https://openid.net/specs/openid-connect-core-1_0-final.html
- RFC 7636 (PKCE):
  https://datatracker.ietf.org/doc/html/rfc7636
- RFC 8252 (OAuth 2.0 for Native Apps):
  https://datatracker.ietf.org/doc/html/rfc8252

## Status Legend

- `Done`: implemented and covered by tests
- `Partial`: implemented but still depends on adjacent unfinished work
- `Missing`: not implemented yet

## Identity Integration Matrix

| Requirement | Source | Repo Area | Status | Evidence | Next Action |
| --- | --- | --- | --- | --- | --- |
| Authorization code flow | Guideline v5.0, OIDC Core | `apps/platform-api` | `Done` | `EkycService::initiate_verification`, `MyDigitalIdClient::build_authorization_url` | Keep aligned with provider metadata |
| `scope` includes `openid` | Guideline v5.0, OIDC Core | `apps/platform-api` | `Done` | `ensure_openid_scope()` in `mydigital_id.rs` | None |
| PKCE code verifier/challenge | Guideline v5.0, RFC 7636 | `apps/platform-api` | `Done` | `pkce.rs`, `OAuthSession`, route persistence | None |
| `state` CSRF binding | Guideline v5.0, OIDC Core | `apps/platform-api` | `Done` | `handle_callback()` constant-time state check, persisted `oauth_sessions.state` | None |
| `nonce` generation and validation | Guideline v5.0, OIDC Core | `apps/platform-api` | `Done` | `build_authorization_url()`, `validate_id_token()`, migration `20260311000003_oauth_sessions_nonce.sql` | None |
| Token exchange | Guideline v5.0 | `apps/platform-api` | `Done` | `exchange_code()` | Add provider conformance tests when staging env exists |
| ID token required and validated | OIDC Core | `apps/platform-api` | `Done` | `validate_id_token()` checks signature, `iss`, `aud`, `exp`, `nonce`, `azp`, `at_hash` | Add real-provider integration test |
| JWKS-based validation for asymmetric tokens | Guideline endpoint list, OIDC Core | `apps/platform-api` | `Done` | `MyDigitalIdConfig.jwks_url`, `fetch_jwks()` | Add metadata discovery if provider supports it |
| UserInfo subject matches ID token subject | OIDC Core | `apps/platform-api` | `Done` | `ensure_matching_subject()` | None |
| No raw PII stored | Internal privacy requirement | `apps/platform-api` | `Done` | `process_claims()` hashes normalized name and IC number | Add retention policy docs |
| Persist OAuth session and verification record | Guideline checklist, internal flow | `apps/platform-api` | `Done` | `store.rs`, route persistence in `ekyc.rs` | Add DB-backed integration tests |
| Verification status lookup API | Product requirement | `apps/platform-api` | `Done` | `GET /api/v1/ekyc/status/:verification_id` now reads persisted state | Add verified-state route test with store fixture |
| DID binding after verification | Product requirement | `apps/platform-api` | `Done` | `POST /api/v1/ekyc/bind-did` now loads persisted record and updates it | Add verified-state success route test with seeded store |
| Real auth-derived tenant context | Internal multi-tenant requirement | `apps/platform-api` | `Done` | `middleware/tenant.rs` now validates issuer/audience-bound bearer JWTs via JWKS or OIDC discovery, with explicit HS256 fallback only for legacy/dev use | None |
| Role/scope authorization on protected routes | Internal security requirement | all backend services | `Partial` | shared `AuthContext` has now been added to `sahi-core`; explicit route policy is not yet consistently applied across services | Implement default-deny route guards after subject-mapping decision |
| Native-app redirect and callback safety | Guideline v5.0, RFC 8252 | wallet + platform API | `Partial` | platform stores callback redirect URI and nonce; wallet path still needs full end-to-end verification | Run mobile integration against real redirect URIs |

## Platform Completion Matrix

| Workstream | Current Status | Evidence | Next Action |
| --- | --- | --- | --- |
| eKYC persistence and callback workflow | `Done` for API/storage layer | `apps/platform-api/src/routes/ekyc.rs`, `store.rs` | Add DB-backed integration tests and staging-provider validation |
| Platform auth model | `Partial` | `middleware/tenant.rs` uses issuer/JWKS-backed bearer validation with discovery support, allowed-algorithm enforcement, and JWKS caching | Finish route-level authorization policy and subject mapping |
| Signing service | `Partial` | management routes are now DB-backed and authenticated; signer-facing actions remain intentionally incomplete pending identity model decisions | Finish signer auth/sign flows and signing finalization pipeline |
| Issuer service | `Missing` | `apps/issuer-service/src/main.rs` placeholder | Implement issuance API and credential lifecycle |
| Verify service | `Missing` | `apps/verify-service/src/main.rs` placeholder | Implement remote verification API and policy surface |
| Wallet contract alignment | `Partial` | wallet eKYC and presentation paths still do not match the hardened backend contract end to end | Add bearer-auth client behavior and real FFI-backed presentation |
| Edge verifier runtime | `Partial` | verifier trust core is improved, but device/runtime layers are still stubbed for the Android target | Implement BLE/GATT/NFC/APDU/MQTT runtime |
| Frontend placeholders (`trustmark-web`, `verify-web`) | `Missing` | workspace entries removed because apps are empty | Build actual apps, then restore to workspace |
| Frontend mock replacement (`admin-portal`, `guest-web`, `guard-tablet`) | `Missing` | apps still contain mock API/data paths | Replace mocks with generated clients from canonical contracts |
| End-to-end and CI alignment | `Partial` | CI now applies all migrations, but critical domain E2E paths are still absent and current Playwright specs are stale | Add end-to-end coverage for eKYC, issuance, verification, signing, and run it in CI |
| Real MyDigital interoperability | `Missing` | no real provider credentials or redirect registrations exist yet | Use staging Keycloak mirror first, then switch to real provider when credentials exist |

## Current Blocking Decisions

These remain unresolved and should not be guessed:

1. Whether JWT `sub` maps directly to the platform `USR_...` identifier.
2. Whether signer actions are invitation-token based, authenticated-user based, or both.
3. Whether a dev-only file-backed keystore plus local JSON trust-registry is approved for `issuer-service` and `verify-service`.
4. Which free-only providers should be used for notifications, storage, queue/broker, and metrics.

## Immediate Execution Order

1. Resolve the blocking decisions listed above.
2. Add shared route-level authorization using the approved `roles + scopes + tenant_id` model.
3. Finish signing-service signer flows.
4. Complete issuer service.
5. Complete verify service.
6. Align wallet clients and FFI to the backend contracts.
7. Implement verifier runtime for Android tablet/phone target.
8. Replace frontend mocks and generated-client drift.
9. Add end-to-end coverage across identity, issuance, verification, and signing.
