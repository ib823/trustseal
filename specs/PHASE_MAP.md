# Sahi Platform — Phase and Module Map

Quick orientation file. Claude Code reads this at the start of each session.
For full module specs, read the relevant sections of `attack-plan.md`.

---

## Phase 0 — Shared Foundation (Weeks 1-4, Sprints 1-2)

All Phase 1+ modules depend on Phase 0 being complete.

| Module | Description | Key deliverables |
|--------|-------------|-----------------|
| **F1** | Key Management Service | CloudHSM abstraction, software mock KMS for dev, key rotation interface, audit events |
| **F2** | Tamper-Evident Log Engine | Merkle tree implementation, 3-channel root publication (IPFS/CT/Notary), append-only guarantee |
| **F3** | API Gateway | Kong configuration, rate limiting, metering, tenant auth middleware |
| **F4** | Multi-Tenant Management | Tenant CRUD, RLS policy factory, billing hooks, all DDL migrations (VaultPass + TrustMark tables) |
| **F5** | DB / Storage / DevOps | Terraform (prod), docker-compose (dev), CI/CD pipeline, monitoring (Prometheus/Grafana stubs) |
| **F6** | Trust Registry | Verifier registration, DID publication, property-verifier mapping |

**Phase 0 also delivers:**
- Error Code Registry (Appendix E) — implemented as `packages/error-codes/`
- ULID convention (Appendix G) — implemented as `packages/ids/`
- i18n infrastructure — `rust-i18n` backend + `next-intl` frontend
- Time format enforcement — RFC 3339 everywhere, `timestamptz` in Postgres
- STRIDE Threat Model document — `docs/stride-threat-model.md`

---

## Phase 1 — VaultPass MVP (Weeks 5-20, Sprints 3-10)

| Module | Description | Sprint |
|--------|-------------|--------|
| **VP-1** | Crypto Engine (Rust) | 3-4 |
| **VP-2** | DID Resolution Layer | 3-4 |
| **VP-3** | Credential Format + Status List | 3-4 |
| **VP-3b** | Compliance Enforcement Layer (C9) | 3-4 |
| **VP-3c** | Business Logic Rules Engine (V5) | 3-4 |
| **VP-4** | Wallet App (Flutter) + BLE + NFC | 5-6 |
| **VP-5** | Edge Verifier (Raspberry Pi, Rust) | 5-6 |
| **VP-6** | Admin Portal (Next.js) | 7-8 |
| **VP-7** | Guard Tablet (Next.js PWA) | 7-8 |
| **VP-8** | Guest Web Flow (Next.js) | 7-8 |
| **VP-9** | Onboarding + KYC flows | 7-8 |

---

## Phase 2 — VaultPass Hardening (Weeks 21-26, Sprints 11-12 start)

- External penetration test
- Legal document review
- PDPA compliance audit
- Performance tuning to meet all targets
- Pilot deployment at 1-2 properties

---

## Phase 3 — TrustMark MVP (Weeks 24-38, Sprints 12-18)

| Module | Description | Sprint |
|--------|-------------|--------|
| **TM-1** | Signing Orchestrator (state machine) | 12-13 |
| **TM-2** | PDF Engine (PAdES-B-T → PAdES-B-LTA) | 13-14 |
| **TM-2b** | Signature Appearance Management | 13 |
| **TM-3** | WebAuthn (passkeys) | 12 |
| **TM-3b** | P2/P3 Assurance Profiles (eKYC, CA) | 14-15 |
| **TM-4** | COSE Token Signer + Label Issuance | 14-15 |
| **TM-5** | Verification Service (doc + print) | 14-15 |
| **TM-6** | Issuer Console + Batch Management | 15-16 |
| **TM-6b** | Supply Chain Controls | 17 |
| **TM-7** | Detached Evidence Bundles | 16-17 |

---

## Phase 4 — TrustMark Hardening (Weeks 39-42, Sprints 19-20)

- Pen test
- Red team (14 scenarios: RT-D1 through RT-D7, RT-P1 through RT-P7)
- Claims policy audit
- Halal pilot
- Capacity planning validation

---

## Dependency Graph (simplified)

```
F1 (KMS) ──────────────────────────────── VP-1, TM-1
F2 (Merkle Log) ───────────────────────── VP-3, TM-1
F3 (API Gateway) ──────────────────────── all apps
F4 (Multi-Tenant + DDL) ───────────────── all apps
F5 (Infra) ────────────────────────────── everything
F6 (Trust Registry) ───────────────────── VP-5, VP-9

VP-1 (Crypto) ─────────────────────────── VP-2, VP-3, VP-4
VP-2 (DID) ────────────────────────────── VP-3, VP-5
VP-3 (Credential Format) ──────────────── VP-4, VP-5, VP-6
VP-3b (Compliance) ────────────────────── VP-4, VP-5
VP-3c (Business Logic) ────────────────── VP-5, VP-6, VP-7

VP-4 (Wallet) ─────────────────────────── [tested against VP-5]
VP-5 (Edge Verifier) ──────────────────── VP-6, VP-7
VP-6 (Admin Portal) ───────────────────── VP-7, VP-8, VP-9
VP-7 (Guard Tablet) ───────────────────── [standalone, reads from F4]
VP-8 (Guest Web) ──────────────────────── VP-5
VP-9 (Onboarding) ─────────────────────── VP-4

F1,F4 ─────────────────────────────────── TM-1
TM-1 (Signing Orchestrator) ───────────── TM-2, TM-3
TM-2 (PDF Engine) ─────────────────────── TM-7
TM-3 (WebAuthn) ───────────────────────── TM-3b
TM-4 (COSE) ───────────────────────────── TM-5, TM-6
TM-5 (Verification) ───────────────────── [standalone API]
TM-6 (Issuer Console) ─────────────────── TM-6b
```
