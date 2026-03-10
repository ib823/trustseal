# STRIDE Threat Model — Sahi Platform

**Version:** 1.0
**Date:** 2026-03-10
**Scope:** All platform components (KMS, API Gateway, Edge Verifier, Wallet, Signing Service)

---

## 1. Overview

This document applies the STRIDE methodology to each major Sahi component.
Each threat category is evaluated with its mitigation strategy and implementation status.

**STRIDE Categories:**
- **S**poofing — Impersonating a user, device, or service
- **T**ampering — Unauthorized modification of data or code
- **R**epudiation — Denying an action occurred
- **I**nformation Disclosure — Exposing data to unauthorized parties
- **D**enial of Service — Making a service unavailable
- **E**levation of Privilege — Gaining unauthorized access levels

---

## 2. Component Threat Analysis

### 2.1 Key Management Service (KMS)

| Threat | Category | Mitigation | Status |
|--------|----------|------------|--------|
| Unauthorized key generation | S | HSM authentication required; RBAC on all key operations | Planned |
| Key material extraction | T | HSM tamper resistance; software keys zeroized on drop | Implemented |
| Deny signing operation | R | `KmsAuditEvent` emitted on every operation (success + failure) | Implemented |
| Key material leaked in logs | I | Key isolation; no key material in log output; zeroize on drop | Implemented |
| Key operation flood | D | Rate limiting per tenant tier (Free: 60/min, Standard: 600/min, Enterprise: 6000/min) | Implemented |
| Cross-tenant key access | E | Tenant ID on all keys; RLS on `keys` table; `FORCE ROW LEVEL SECURITY` | Implemented |

### 2.2 API Gateway

| Threat | Category | Mitigation | Status |
|--------|----------|------------|--------|
| Token forgery | S | JWT signature validation with Ed25519/ECDSA P-256 | Planned |
| Request modification in transit | T | TLS 1.3 mandatory; request signing for sensitive operations | Planned |
| Deny API action | R | Request ID (`REQ_` ULID) on every request; structured logging with tracing | Implemented |
| PII in error responses | I | Error responses use SAHI error codes only; no internal details exposed | Implemented |
| API flood | D | Token bucket rate limiting per IP and tenant; 429 with Retry-After header | Implemented |
| Cross-tenant data access | E | Tenant extraction middleware; RLS on all tenant-scoped tables | Implemented |

### 2.3 Edge Verifier

| Threat | Category | Mitigation | Status |
|--------|----------|------------|--------|
| Fake verifier device | S | Device certificate + mutual TLS; hardware attestation | Planned |
| Firmware tampering | T | Tamper switch triggers LUKS wipe; verified boot chain | Planned |
| Deny gate decision | R | Local audit log with Merkle tree; sync to cloud when online | Planned |
| Credential data leakage | I | Minimal data stored (hash only); encrypted local storage | Planned |
| Device unavailable | D | Hardware watchdog; fail-closed policy; offline operation mode | Planned |
| Privilege escalation on device | E | Single-purpose OS; no shell access; read-only rootfs | Planned |

### 2.4 Mobile Wallet (Flutter)

| Threat | Category | Mitigation | Status |
|--------|----------|------------|--------|
| Stolen device credential use | S | Biometric + Secure Enclave binding; credential requires user presence | Planned |
| App binary tampering | T | freeRASP integrity checks; app attestation (Play Integrity / App Attest) | Planned |
| Deny credential presentation | R | Presentation log stored locally; synced to platform | Planned |
| Credential data at rest | I | Encrypted storage (Secure Enclave / Android Keystore) | Planned |
| BLE/NFC timeout | D | 200ms presentation target; graceful timeout handling | Planned |
| Sandbox escape | E | OS sandbox; no inter-app data sharing; minimal permissions | Planned |

### 2.5 Signing Service (TrustMark)

| Threat | Category | Mitigation | Status |
|--------|----------|------------|--------|
| Unauthorized signing | S | WebAuthn authentication + HSM-bound signing key | Planned |
| Document modification post-sign | T | PAdES integrity seal; Merkle log entry for each signature | Planned |
| Deny signing action | R | TSA timestamps; audit trail with Merkle proof | Planned |
| Document content exposure | I | Document encryption at rest; key-per-document isolation | Planned |
| Signing queue flood | D | Queue-based processing; per-tenant rate limits | Planned |
| Role bypass | E | Role-based access (signer, witness, notary); ceremony state machine | Planned |

---

## 3. Cross-Cutting Concerns

### 3.1 Supply Chain Security

| Threat | Mitigation | Status |
|--------|------------|--------|
| Compromised dependency | `cargo audit` in CI; Dependabot alerts; lockfile pinning | Implemented |
| Malicious container image | Distroless base images; image signing; SBOM generation | Planned |
| Compromised build pipeline | GitHub Actions with OIDC; no long-lived secrets; signed commits | Partial |

### 3.2 Data in Transit

| Threat | Mitigation | Status |
|--------|------------|--------|
| MITM attack | TLS 1.3 everywhere; HSTS headers; certificate pinning in wallet | Planned |
| MQTT eavesdropping | MQTT over TLS; per-device client certificates | Planned |
| BLE interception | Encrypted BLE channel; short-lived presentation tokens | Planned |

### 3.3 Data at Rest

| Threat | Mitigation | Status |
|--------|------------|--------|
| Database breach | Column-level encryption for sensitive fields; RLS enforced | Partial |
| Backup exposure | Encrypted backups; key rotation; access logging | Planned |
| Log file exposure | No PII in logs; structured logging; log rotation | Implemented |

---

## 4. Risk Severity Matrix

| Severity | Definition | Response Time |
|----------|-----------|---------------|
| Critical | Active exploitation possible; data breach risk | Immediate (< 4 hours) |
| High | Exploitable with moderate effort; integrity risk | < 24 hours |
| Medium | Requires specific conditions; limited impact | < 1 week |
| Low | Theoretical; defense-in-depth layer | Next sprint |

---

## 5. Review Schedule

- **Quarterly:** Full STRIDE review of all components
- **Per-release:** Delta review of changed components
- **Incident-driven:** Ad-hoc review after any security incident

---

## 6. References

- MASTER_PLAN.md §15.1 — STRIDE Threat Model matrix
- MASTER_PLAN.md §15.4 — Supply chain security
- OWASP Threat Modeling Guide
- NIST SP 800-154 — Guide to Data-Centric System Threat Modeling
