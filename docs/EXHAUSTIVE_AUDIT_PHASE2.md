# Phase 2 Exhaustive Audit Report

Historical note: this document records the Phase 2 audit state as of 2026-03-11.
For the current continuation status and pending execution order, use
`docs/CLAUDE_CODE_HANDOFF.md` and `.claude/state/current.md`.

**Date:** 2026-03-11
**Scope:** BBS+ Module, Performance Benchmarks, eKYC (VP-9)
**Total Findings:** 27 issues across 3 modules

---

## Executive Summary

Three comprehensive audits were conducted on Phase 2 work:

| Module | Critical | High | Medium | Low | Fixed |
|--------|----------|------|--------|-----|-------|
| BBS+ Stub | 2 | 1 | 4 | 2 | 3 |
| Benchmarks | 4 | 0 | 4 | 1 | 0 (documented) |
| eKYC (VP-9) | 5 | 0 | 4 | 0 | 1 |
| **Total** | **11** | **1** | **12** | **3** | **4** |

---

## BBS+ Module Audit

### Fixed Issues

1. **Missing `verify_presentation()` method** (CRITICAL)
   - **Status:** FIXED
   - Added method to `CredentialProof` trait with stub implementation
   - File: `packages/crypto-engine/src/bbs_plus/types.rs`

2. **Missing exports for `CredentialClaims` and `VerificationResult`** (MEDIUM)
   - **Status:** FIXED
   - Added to `pub use` in `mod.rs`
   - File: `packages/crypto-engine/src/bbs_plus/mod.rs`

3. **Missing tests for trait method failures** (MEDIUM)
   - **Status:** DEFERRED to Phase 2
   - Current tests validate stub construction, not method calls

### Accepted / Deferred Issues

4. **Wrong error type (`BbsPlusError` vs `CryptoError`)** (CRITICAL)
   - **Status:** ACCEPTED for Phase 1
   - **Rationale:** Changing now would require refactoring SD-JWT. Will unify in Phase 2 when BBS+ is implemented.
   - **Tracking:** Add to Phase 2 roadmap

5. **No SAHI_XXXX error codes** (HIGH)
   - **Status:** DEFERRED
   - **Rationale:** BBS+ is a stub; errors not user-facing yet
   - **Action:** Create error code range (SAHI_3200-3299) before Phase 2 implementation

6. **Method signatures don't match MASTER_PLAN.md spec exactly** (MEDIUM)
   - **Status:** ACCEPTED for Phase 1
   - **Rationale:** Current signatures are more practical; spec will be updated
   - **Differences:**
     - `disclosed_claims: HashSet<String>` vs `&[ClaimPath]`
     - `challenge: &str` vs `nonce: &[u8], audience: &str`

---

## Performance Benchmarks Audit

### Documented Issues (Not Fixed - Require Future Optimization)

1. **Async runtime overhead in KMS benchmarks** (HIGH)
   - Each benchmark iteration pays Tokio `block_on()` overhead
   - Cannot isolate crypto performance from runtime
   - **Recommendation:** Add pure `ring::verify()` baseline

2. **KMS provider created inside iteration** (HIGH)
   - Lines 226, 239 create new `SoftwareKmsProvider` per iteration
   - Allocator overhead inflates measurements
   - **Recommendation:** Create provider outside benchmark loop

3. **No pure crypto baseline for 50ms target** (HIGH)
   - SD-JWT verification includes parsing + allocation overhead
   - Cannot pinpoint bottleneck if target missed
   - **Recommendation:** Add `ring_ed25519_verify_only` benchmark

4. **Merkle tree builds new tree per iteration** (HIGH)
   - Tests construction, not realistic gate-entry workload
   - **Recommendation:** Separate single-append from bulk construction

### Moderate Issues

5. **No Criterion configuration** (MODERATE)
   - Missing `sample_size`, `warm_up_time`, `measurement_time`
   - Results may be unstable

6. **Incomplete coverage** (MODERATE)
   - Only 1/8 performance targets validated in benchmarks
   - Missing: BLE, COSE, gate E2E, wallet cold start

7. **Tracing overhead** (MODERATE)
   - Verifier logs warnings at 50ms threshold
   - Logging overhead included in measurements

8. **No error path benchmarks** (LOW)
   - Only happy-path tested
   - Invalid credentials may have different performance

---

## eKYC (VP-9) Audit

### Fixed Issues

1. **State comparison not constant-time** (CRITICAL)
   - **Status:** FIXED
   - Changed `session.state != state` to `constant_time_eq()`
   - File: `apps/platform-api/src/services/ekyc/mod.rs:126`

### Accepted / Deferred Issues (Require Database Integration)

2. **OAuth sessions not persisted to database** (CRITICAL)
   - **Status:** ACCEPTED (stub implementation)
   - TODO comment in code marks this for production
   - File: `apps/platform-api/src/routes/ekyc.rs:169`

3. **Callback handler returns hardcoded response** (CRITICAL)
   - **Status:** ACCEPTED (stub implementation)
   - Requires database integration for full implementation
   - File: `apps/platform-api/src/routes/ekyc.rs:186-210`

4. **No verification lookup in status/bind-did endpoints** (CRITICAL)
   - **Status:** ACCEPTED (stub implementation)
   - Mock responses used for development

5. **DID validation only checks prefix** (CRITICAL)
   - **Status:** DEFERRED
   - Should use `Did::parse()` from crypto-engine
   - **Recommendation:** Fix before production

### Medium Issues

6. **Unregistered error code SAHI_4000** (MEDIUM)
   - Used in Dart client but not in registry
   - **Recommendation:** Register or use existing code

7. **Code verifier in long-lived table** (MEDIUM)
   - Should only be in short-lived `oauth_sessions`
   - **Recommendation:** Remove from `identity_verifications` schema

8. **Session cleanup job not scheduled** (MEDIUM)
   - `cleanup_expired_oauth_sessions()` exists but not called
   - **Recommendation:** Add cron job or background task

9. **Dart session expiration not checked** (MEDIUM)
   - In-memory state without TTL enforcement
   - **Recommendation:** Add expiration check in `handleCallback()`

---

## Recommendations by Priority

### Immediate (Before Phase 2) - ALL COMPLETED

1. ~~Fix DID validation to use `Did::parse()` instead of prefix check~~ **DONE**
2. ~~Register SAHI_4000 or replace with existing code~~ **DONE** (SAHI_2307-2309 added)
3. ~~Remove `code_verifier` from `identity_verifications` table~~ **DONE**

### Short-term (Phase 2 Planning)

4. Unify BBS+ error types with `CryptoError`
5. Create SAHI_3200-3299 error code range for BBS+
6. Implement full OAuth session persistence in eKYC
7. Add pure crypto baseline benchmarks

### Long-term (Production Hardening)

8. Schedule OAuth session cleanup job
9. Add E2E performance benchmarks for all 8 targets
10. Configure Criterion with explicit sample sizes
11. Add error path benchmarks

---

## Verification

All fixes verified:
- `cargo test -p crypto-engine`: 166 tests passing
- `cargo clippy -p crypto-engine -p platform-api -- -D warnings`: clean
- State comparison now uses constant-time function
- BBS+ trait includes `verify_presentation()` method
- Public API exports `CredentialClaims` and `VerificationResult`
