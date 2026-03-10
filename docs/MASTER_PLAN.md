# SAHI PLATFORM — COMPLETE MASTER PLAN

**Version:** 1.1 | **Date:** 2026-03-10 | **Updated:** Gap analysis applied
**Products:** VaultPass (physical access control) + TrustMark (document signing & label authentication)
**Target:** Production-grade Malaysian trust infrastructure

---

## TABLE OF CONTENTS

1. [Vision & Architecture Overview](#1-vision--architecture-overview)
2. [Cryptographic Foundation](#2-cryptographic-foundation)
3. [Key Management Service (F1)](#3-key-management-service-f1)
4. [Tamper-Evident Log Engine (F2)](#4-tamper-evident-log-engine-f2)
5. [API Gateway & Multi-Tenancy (F3/F4)](#5-api-gateway--multi-tenancy-f3f4)
6. [Infrastructure & DevOps (F5)](#6-infrastructure--devops-f5)
7. [Trust Registry (F6)](#7-trust-registry-f6)
8. [VaultPass — Credential System (VP-1 to VP-3c)](#8-vaultpass--credential-system-vp-1-to-vp-3c)
9. [VaultPass — Mobile Wallet (VP-4)](#9-vaultpass--mobile-wallet-vp-4)
10. [VaultPass — Edge Verifier (VP-5)](#10-vaultpass--edge-verifier-vp-5)
11. [VaultPass — Web Applications (VP-6 to VP-9)](#11-vaultpass--web-applications-vp-6-to-vp-9)
12. [TrustMark — Signing System (TM-1 to TM-3b)](#12-trustmark--signing-system-tm-1-to-tm-3b)
13. [TrustMark — Labels & Verification (TM-4 to TM-7)](#13-trustmark--labels--verification-tm-4-to-tm-7)
14. [UI/UX Design System](#14-uiux-design-system)
15. [Security Architecture](#15-security-architecture)
16. [Performance Engineering](#16-performance-engineering)
17. [Testing Strategy](#17-testing-strategy)
18. [CI/CD Pipeline](#18-cicd-pipeline)
19. [Observability & Monitoring](#19-observability--monitoring)
20. [Compliance & Legal](#20-compliance--legal)
21. [Deployment & Infrastructure](#21-deployment--infrastructure)
22. [Phase Execution Timeline](#22-phase-execution-timeline)
23. [Risk Mitigation](#23-risk-mitigation)
24. [Cross-Cutting Concerns](#24-cross-cutting-concerns)

---

## 1. VISION & ARCHITECTURE OVERVIEW

### 1.1 What Sahi Is

Sahi is a **trust infrastructure platform** for Malaysia that provides two products built on a shared cryptographic foundation:

- **VaultPass:** Privacy-preserving physical access control using SD-JWT credentials, BLE/NFC presentation, and zero-knowledge gate entry
- **TrustMark:** Cryptographic document signing (PAdES-B-LTA) and physical label authentication (COSE tokens on QR/NFC)

### 1.2 Architectural Principles

| Principle | Implementation |
|-----------|---------------|
| **Cryptographic verifiability** | Every claim is backed by a digital signature verifiable without trusting the platform |
| **Privacy by design** | Selective disclosure (SD-JWT/BBS+), no PII in logs, minimal data collection |
| **Fail-closed security** | Every gate decision defaults to DENY on error, never GRANT |
| **Offline-capable** | Edge verifiers and wallets operate with cached trust material |
| **Multi-tenant isolation** | PostgreSQL RLS enforces tenant boundaries at the database layer |
| **HSM-only signing** | Production signing keys never exist in software |
| **Tamper-evident audit** | Merkle tree logs with 3-channel root publication |

### 1.3 System Topology

```
                    ┌─────────────────────────────────┐
                    │         TRUST LAYER              │
                    │  KMS (F1) · Merkle Log (F2)      │
                    │  Trust Registry (F6)             │
                    └──────────┬──────────────────────┘
                               │
                    ┌──────────▼──────────────────────┐
                    │       PLATFORM LAYER             │
                    │  Platform API · Issuer Service   │
                    │  Signing Service · Verify Svc    │
                    │  API Gateway (F3) · Tenants (F4) │
                    └──┬───────────┬──────────┬───────┘
                       │           │          │
          ┌────────────▼──┐  ┌────▼─────┐  ┌─▼──────────┐
          │  VAULTPASS     │  │ TRUSTMARK│  │  SHARED     │
          │  Wallet (Flutter)│  │ Web UI  │  │  Verify Web │
          │  Admin Portal  │  │ Issuer   │  │  Landing Pg │
          │  Guard Tablet  │  │ Console  │  │             │
          │  Guest Web     │  │          │  │             │
          └───────┬────────┘  └──────────┘  └─────────────┘
                  │
          ┌───────▼────────┐
          │  EDGE LAYER    │
          │  RPi Verifier  │
          │  BLE + NFC     │
          │  GPIO → Lock   │
          └────────────────┘
```

### 1.4 Data Flow — Gate Entry (VaultPass)

```
1. Resident approaches gate
2. Phone BLE detects verifier broadcast (background scan)
3. Wallet prompts biometric
4. On biometric success:
   a. Wallet reads challenge nonce from verifier GATT characteristic
   b. Wallet constructs SD-JWT presentation (selective disclosure)
   c. Wallet signs Key Binding JWT with hardware-bound key
   d. Wallet writes presentation to verifier GATT characteristic
5. Edge verifier:
   a. Validates issuer signature (Ed25519)
   b. Validates KB-JWT holder binding
   c. Checks revocation via cached Status List
   d. Evaluates access rules (floor, time, role)
   e. Sends GRANTED/DENIED via GATT notify
   f. Actuates door lock relay via GPIO
   g. Logs event to local audit log
   h. Syncs event via MQTT when online
```

**Total latency budget: <2 seconds end-to-end**

### 1.5 Data Flow — Document Signing (TrustMark)

```
1. Issuer creates signing ceremony via Issuer Console
2. Ceremony state machine: CREATED → PREPARING → READY
3. For each signer:
   a. Signer authenticates via WebAuthn passkey
   b. Document hash computed (SHA-256)
   c. CMS signature created via HSM
   d. Signature embedded in PDF (PAdES-B-T with timestamp)
4. After all signers complete:
   a. Async job fetches OCSP/CRL data → augments to PAdES-B-LT
   b. Batch job adds document timestamp → augments to PAdES-B-LTA
5. Ceremony state: FULLY_SIGNED → TIMESTAMPING → AUGMENTING_LTV → COMPLETE
6. All transitions logged to Merkle tree
```

---

## 2. CRYPTOGRAPHIC FOUNDATION

### 2.1 Algorithm Selection

| Purpose | Algorithm | Crate | Rationale |
|---------|-----------|-------|-----------|
| SD-JWT issuer signing | Ed25519 | `ring` 0.17+ | Fastest, smallest signatures, deterministic |
| SD-JWT holder binding | Ed25519 | `ring` | Key Binding JWT |
| BBS+ (Phase 2) | BLS12-381 | `bbs` | Unlinkable presentations |
| PAdES document signing | ECDSA P-256 | `ring` / HSM | Broad PDF viewer compatibility |
| COSE label tokens | ES256 (P-256) | `coset` + `ring` | Constrained verifier compatibility |
| Password hashing | Argon2id | `argon2` (RustCrypto) | Memory-hard, GPU-resistant |
| Integrity hashing | BLAKE2b | `blake2` (RustCrypto) | Faster than SHA-256, equally secure |
| Merkle tree hashing | SHA-256 | `ring` | Interoperability with CT logs |
| TLS | ECDHE + AES-256-GCM | `rustls` | No OpenSSL dependency |
| Token encryption | AES-256-GCM-SIV | `aes-gcm-siv` | Nonce-misuse resistant |

### 2.2 Key Hierarchy

```
HSM Master Key (never exported)
├── Tenant Root Key (per tenant, HSM-wrapped)
│   ├── Issuer Signing Key (Ed25519 — SD-JWT)
│   │   └── Rotated every 90 days, old keys retained for verification
│   ├── Issuer BBS+ Key (BLS12-381 — Phase 2)
│   ├── Document Signing Key (ECDSA P-256 — PAdES)
│   ├── COSE Signing Key (ES256 — labels)
│   └── Encryption Key (AES-256 — data at rest)
├── Platform Root Key
│   ├── Merkle Log Signing Key (Ed25519)
│   ├── Status List Signing Key (Ed25519)
│   └── Trust Registry Signing Key (Ed25519)
└── Operational Keys
    ├── mTLS Service Certificates
    └── API JWT Signing Key (RS256 or EdDSA)
```

### 2.3 Key Rotation Strategy

- **Issuer signing keys:** Rotate every 90 days. New key published in DID document with `validFrom`. Old key remains for verification with `validUntil` grace period (30 days overlap).
- **Document signing keys:** Rotate yearly or on algorithm deprecation.
- **COSE keys:** Rotate per batch campaign. Key ID embedded in unprotected header enables rotation without re-signing existing tokens.
- **Platform keys:** Rotate annually with HSM ceremony.
- **API JWT keys:** Rotate every 30 days with JWKS endpoint.
- **Key rotation coordination:** On rotation, old key remains active for `verification-only` during overlap period. New key published in DID document / JWKS **before** old key starts signing. All key rotation events logged to Merkle tree (F2).
- **Key destruction ceremony:** Requires dual authorization (two admin users) + audit log entry. HSM key zeroization confirmed via HSM audit log.

### 2.4 SD-JWT Architecture

**Credential structure (W3C VC Data Model 2.0):**
```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2"],
  "type": ["VerifiableCredential", "AccessBadge"],
  "issuer": "did:web:issuer.sahi.my",
  "validFrom": "2026-03-10T00:00:00Z",
  "validUntil": "2026-03-11T00:00:00Z",
  "credentialSubject": {
    "id": "did:key:z6Mkh...",
    "_sd": ["...hash of name...", "...hash of unit...", "...hash of role..."],
    "floor_access": [1, 2, 3],
    "clearance": "standard"
  },
  "credentialStatus": {
    "type": "BitstringStatusListEntry",
    "statusListIndex": 42,
    "statusListCredential": "https://sahi.my/status/1"
  }
}
```

**Implementation rules:**
- Salt: 128-bit cryptographically random, encoded as `Base64url(random(16))`
- Decoy digests: 2-4 per credential (prevents claim-count inference)
- KB-JWT: Always required (audience + nonce + 5-second expiry)
- Hash algorithm: SHA-256 (`_sd_alg: "sha-256"`)

### 2.5 SD-JWT → BBS+ Migration (Phase 1 → Phase 2)

**Core abstraction trait (implement from day one):**
```rust
pub trait CredentialProof: Send + Sync {
    fn sign(
        &self,
        credential: &Credential,
        key: &dyn SigningKey,
    ) -> Result<SecuredCredential, CryptoError>;

    fn derive_presentation(
        &self,
        credential: &SecuredCredential,
        disclosed_claims: &[ClaimPath],
        nonce: &[u8],
        audience: &str,
    ) -> Result<Presentation, CryptoError>;

    fn verify_presentation(
        &self,
        presentation: &Presentation,
        issuer_pk: &dyn VerifyingKey,
        nonce: &[u8],
    ) -> Result<VerifiedClaims, CryptoError>;
}
```

- Phase 1: `SdJwtProof` implementation
- Phase 2: `BbsPlusProof` implementation (additive, no breaking changes)
- Dual issuance during transition: wallet stores both formats
- Verifier capability negotiation: `supported_cryptosuites: ["sd-jwt", "bbs-2023"]`

### 2.6 Revocation Strategy

**Hybrid approach for optimal privacy/freshness trade-off:**
- **Primary:** Short-lived credentials (4-8 hour TTL) — eliminates most revocation checks
- **Emergency:** Bitstring Status List for immediate revocation within validity window
  - 16KB covers 131,072 credentials
  - Cached by edge verifiers with 900-second TTL (15 minutes)
  - Status list published as a signed VC at a well-known URL
  - Randomized index assignment prevents correlation
- **Credential refresh strategy:** Wallet auto-refreshes credentials at 75% of TTL (e.g., 6 hours into an 8-hour TTL). Refresh is silent if biometric session is still active; otherwise, prompts biometric re-auth.
- **Batch revocation:** Admin can revoke all credentials for a tenant, property, or user in a single operation. Status list updated atomically.

---

## 3. KEY MANAGEMENT SERVICE (F1)

### 3.1 Architecture

```rust
/// KMS provider abstraction — the core of F1
#[async_trait]
pub trait KmsProvider: Send + Sync {
    /// Generate a new key pair within the HSM
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: &str,
    ) -> Result<KeyHandle, KmsError>;

    /// Sign data using a key stored in the HSM
    async fn sign(
        &self,
        key_handle: &KeyHandle,
        data: &[u8],
    ) -> Result<Signature, KmsError>;

    /// Verify a signature (can be done in software)
    async fn verify(
        &self,
        key_handle: &KeyHandle,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, KmsError>;

    /// Export public key (private key never leaves HSM)
    async fn export_public_key(
        &self,
        key_handle: &KeyHandle,
    ) -> Result<PublicKeyBytes, KmsError>;

    /// Rotate key: generate new key, mark old key as verify-only
    async fn rotate_key(
        &self,
        old_handle: &KeyHandle,
    ) -> Result<KeyRotationResult, KmsError>;

    /// List all keys with metadata
    async fn list_keys(&self) -> Result<Vec<KeyMetadata>, KmsError>;

    /// Destroy a key (irreversible, requires confirmation)
    async fn destroy_key(
        &self,
        key_handle: &KeyHandle,
        confirmation: DestroyConfirmation,
    ) -> Result<(), KmsError>;
}
```

### 3.2 Implementations

| Provider | Context | Notes |
|----------|---------|-------|
| `SoftwareKmsProvider` | Local development | In-memory keys using `ring`. Keys stored encrypted on disk. |
| `AwsCloudHsmProvider` | Production | PKCS#11 interface to CloudHSM cluster. Keys never leave the HSM. |
| `AwsKmsProvider` | Staging | AWS KMS API. Cheaper than CloudHSM, suitable for non-production. |

**Provider selection:** Determined by `SAHI_KMS_PROVIDER` environment variable (`software`, `aws-kms`, `aws-cloudhsm`). Provider is initialized once at startup and injected via `Arc<dyn KmsProvider>`. Provider swap requires only config change — zero code changes.

**Concurrency safety:** All KMS operations are `async` and thread-safe. The `SoftwareKmsProvider` uses `RwLock<HashMap<KeyHandle, KeyMaterial>>` internally. CloudHSM provider uses a connection pool (PKCS#11 session pool, max 10 concurrent sessions).

### 3.3 Audit Requirements

Every KMS operation emits an audit event:
```rust
struct KmsAuditEvent {
    event_id: Ulid,          // Prefix: EVT_
    timestamp: DateTime<Utc>,
    operation: KmsOperation, // GenerateKey, Sign, Verify, Rotate, Destroy
    key_handle: KeyHandle,
    tenant_id: Ulid,         // Prefix: TNT_
    actor_id: Ulid,          // Prefix: USR_ or SVC_
    success: bool,
    error_code: Option<SahiErrorCode>,
    ip_address: Option<IpAddr>, // Encrypted at rest
}
```

---

## 4. TAMPER-EVIDENT LOG ENGINE (F2)

### 4.1 Merkle Tree Implementation

**Structure:** Binary hash tree using SHA-256. Each leaf is a hash of a log entry. Interior nodes are `SHA-256(left_child || right_child)`.

**Append-only guarantee:** The tree is stored in PostgreSQL with an append-only constraint (no UPDATE/DELETE, enforced via RLS + trigger). Each new entry references the previous root hash, forming a hash chain.

**Atomicity requirement:** Every Merkle log append MUST be atomic — the log entry INSERT, root hash UPDATE, and the business event that triggered the log entry MUST be in the same database transaction. If the business event succeeds but the log append fails, the entire transaction rolls back. This prevents "unlogged events" that could undermine audit integrity.

```sql
BEGIN;
  INSERT INTO merkle_log_entries (...) VALUES (...);
  UPDATE merkle_tree_state SET root_hash = $new_root, sequence = sequence + 1;
  -- Business event (e.g., credential issuance) also in this transaction
COMMIT;
```

### 4.2 Log Entry Structure

```rust
struct MerkleLogEntry {
    entry_id: Ulid,          // Prefix: LOG_
    sequence: u64,           // Monotonic, gapless
    timestamp: DateTime<Utc>,
    event_type: EventType,   // KmsOp, CredentialIssue, CredentialRevoke, CeremonyTransition, GateDecision
    payload_hash: [u8; 32],  // SHA-256 of the event payload
    previous_root: [u8; 32], // Root hash before this entry
    new_root: [u8; 32],      // Root hash after this entry
    tenant_id: Ulid,
}
```

### 4.3 Three-Channel Root Publication

Merkle roots are published to three independent channels to prevent undetected tampering:

| Channel | Implementation | Frequency | Verification |
|---------|---------------|-----------|-------------|
| **IPFS** | Pin root hash as IPFS CID | Every 1000 entries or hourly | Anyone can fetch and compare |
| **Certificate Transparency** | Submit to CT log as precertificate | Daily | CT monitors detect inconsistencies |
| **Notary** | Signed timestamp from independent TSA | Every 1000 entries or hourly | TSA receipt proves existence at time |

### 4.4 Verification API

```
GET /api/v1/audit/verify/{entry_id}
→ Returns: Merkle proof (inclusion proof), root hash, root publication receipts

GET /api/v1/audit/consistency/{old_root}/{new_root}
→ Returns: Consistency proof (old tree is prefix of new tree)
```

---

## 5. API GATEWAY & MULTI-TENANCY (F3/F4)

### 5.1 API Gateway (F3)

**Implementation:** Kong Gateway (open-source) or custom Axum middleware stack.

**Middleware chain (order matters):**
```
Request
  → TLS termination
  → Rate limiting (per tenant, per IP, per endpoint)
  → Request ID injection (ULID with REQ_ prefix)
  → JWT validation (signature + expiry + audience)
  → Tenant extraction (from JWT claims)
  → RLS session setup (SET app.tenant_id)
  → Input validation (Axum typed extractors)
  → Business logic handler
  → Audit event emission
  → Response serialization
  → Metering (track API usage per tenant)
```

### 5.2 Rate Limiting

| Tier | Requests/min | Burst | Applied to |
|------|-------------|-------|-----------|
| Free | 60 | 10 | Per API key |
| Standard | 600 | 50 | Per tenant |
| Enterprise | 6000 | 200 | Per tenant |
| Internal | Unlimited | - | Service-to-service (mTLS) |

### 5.3 Multi-Tenant Management (F4)

**Tenant lifecycle:**
```
PROVISIONING → ACTIVE → SUSPENDED → TERMINATED
                 ↑          │
                 └──────────┘ (reinstate)
```

**RLS enforcement pattern:**
```sql
-- Every tenant-scoped table
ALTER TABLE credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE credentials FORCE ROW LEVEL SECURITY;  -- CRITICAL: prevents table owner bypass

CREATE POLICY tenant_isolation ON credentials
    USING (tenant_id = current_setting('app.tenant_id')::text);

-- Set on every request (SET LOCAL for pgBouncer transaction-mode compatibility)
SET LOCAL app.tenant_id = 'TNT_01HXK...';
-- IMPORTANT: Use SET LOCAL (not SET) — scoped to transaction, auto-cleared on COMMIT/ROLLBACK
-- This prevents tenant_id leaking to the next request on a pooled connection
```

**DDL migration strategy:**
- All tables created in F4 migrations (both VaultPass and TrustMark schemas)
- Every table with tenant data has RLS enabled
- All IDs are ULIDs with registered prefixes (Appendix G)
- All timestamps are `timestamptz`
- Every migration is reversible (UP + DOWN)

---

## 6. INFRASTRUCTURE & DEVOPS (F5)

### 6.1 Local Development Stack

```yaml
# docker-compose.yml
services:
  postgres:     # PostgreSQL 16-alpine, port 5432
  redis:        # Redis 7-alpine, port 6379
  mqtt:         # Eclipse Mosquitto 2, ports 1883/9001
```

### 6.2 Production Infrastructure (Terraform)

```
AWS Region: ap-southeast-1 (Singapore — closest to Malaysia)

├── VPC (3 AZs)
│   ├── Public subnets (ALB, NAT Gateway)
│   ├── Private subnets (ECS tasks, RDS)
│   └── Isolated subnets (CloudHSM)
├── ECS Fargate
│   ├── platform-api (2-8 tasks, autoscaled)
│   ├── issuer-service (2-4 tasks)
│   ├── signing-service (2-4 tasks)
│   └── verify-service (2-8 tasks, autoscaled)
├── RDS PostgreSQL 16 (Multi-AZ, encrypted)
├── ElastiCache Redis 7 (cluster mode)
├── CloudHSM (2-node cluster, Multi-AZ)
├── ALB + WAF (OWASP Core Rule Set)
├── S3 (signed documents, encrypted)
├── CloudFront (verification landing page)
├── IoT Core (MQTT for edge verifiers)
├── Secrets Manager (all credentials)
├── ECR (container registry)
└── CloudWatch + OpenTelemetry
```

### 6.3 CI/CD Pipeline

```
Push to main
  → GitHub Actions
    → [Parallel]
      → Rust: fmt check → clippy → test → sqlx prepare check → build
      → TypeScript: lint → type-check → test → build
      → Docker: build multi-stage images → scan → push to ECR
    → [Sequential]
      → Integration tests (testcontainers: Postgres + Redis)
      → Security scan (cargo audit, npm audit, container scan)
      → Deploy to staging (ECS blue-green)
      → Smoke tests against staging
      → [Manual approval gate]
      → Deploy to production (ECS blue-green)
```

### 6.4 Container Strategy

```dockerfile
# Multi-stage Rust build
FROM rust:1.85-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin platform-api

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/platform-api /
CMD ["/platform-api"]
```

- Distroless base images (no shell, no package manager — minimal attack surface)
- Each service is its own container
- Health check endpoints on every service
- Graceful shutdown handling (SIGTERM → drain connections → exit)

---

## 7. TRUST REGISTRY (F6)

### 7.1 Design

The Trust Registry answers: **"Is this DID authorized to issue credentials of type X?"**

```rust
struct TrustRegistry {
    version: u64,
    published: DateTime<Utc>,
    governance_did: String,       // did:web:sahi.my
    entries: Vec<TrustEntry>,
    signature: Signature,         // Ed25519 signature over the registry
}

struct TrustEntry {
    issuer_did: String,           // did:web:issuer.property.sahi.my
    credential_types: Vec<String>,// ["AccessBadge", "VisitorPass"]
    valid_from: DateTime<Utc>,
    valid_until: Option<DateTime<Utc>>,
    status: TrustStatus,          // Active, Suspended, Revoked
    verifier_dids: Vec<String>,   // Which verifiers trust this issuer
    properties: Vec<Ulid>,        // Property IDs this issuer serves
}
```

### 7.2 Distribution

- Published as a signed JSON document at `https://sahi.my/.well-known/trust-registry.json`
- Edge verifiers cache locally with 1-4 hour TTL
- HTTP ETag for efficient conditional polling
- MQTT push notification on registry updates (topic: `sahi/{tenant}/trust-registry/updated`)

**MQTT security:**
- TLS 1.3 mandatory for all MQTT connections
- Client certificate authentication (mTLS) for edge verifiers
- Topic ACLs: each verifier can only subscribe to its own tenant/property topics
- Message payload signed with platform key (Ed25519) to prevent MQTT broker compromise from injecting fake registry updates
- QoS 1 minimum for all critical messages (trust registry, status list, revocation)

### 7.3 DID Architecture

| Entity | DID Method | Example |
|--------|-----------|---------|
| Platform | `did:web` | `did:web:sahi.my` |
| Tenant/Property | `did:web` | `did:web:property.sahi.my` |
| Resident (Phase 1) | `did:key` | `did:key:z6Mkh...` |
| Resident (Phase 2) | `did:peer` | Pairwise, unlinkable |
| Edge Verifier | `did:web` | `did:web:gate-a.property.sahi.my` |

---

## 8. VAULTPASS — CREDENTIAL SYSTEM (VP-1 to VP-3c)

### 8.1 Crypto Engine — VP-1

**Location:** `packages/crypto-engine/`

**Modules:**
```
crypto-engine/src/
├── lib.rs
├── kms.rs           # KmsProvider trait + implementations (re-export from F1)
├── sd_jwt/
│   ├── mod.rs
│   ├── issuer.rs    # SdJwtIssuer: create SD-JWT credentials
│   ├── holder.rs    # SdJwtHolder: derive presentations
│   └── verifier.rs  # SdJwtVerifier: verify presentations
├── bbs_plus/
│   └── mod.rs       # Stub — Phase 2 implementation
├── keys/
│   ├── mod.rs
│   ├── ed25519.rs   # Ed25519 key operations via ring
│   └── ecdsa.rs     # ECDSA P-256 for PAdES/COSE
├── hash/
│   ├── mod.rs
│   ├── blake2.rs    # BLAKE2b for integrity
│   └── sha256.rs    # SHA-256 for Merkle/SD-JWT
└── wasm/
    └── mod.rs       # WASM bindings for browser verification
```

### 8.2 DID Resolution — VP-2

**Location:** `packages/did-resolver/` (TypeScript) + Rust counterpart in `crypto-engine`

**Resolution flow:**
1. Parse DID method (`did:web`, `did:key`, `did:peer`)
2. For `did:web`: HTTPS fetch to `/.well-known/did.json` (or path-based)
3. Cache resolved document in Redis (TTL: 1-24 hours)
4. Verify document integrity (optional: pin hash for known issuers)
5. Extract verification methods for the requested purpose

**Caching layers:**
- L1: In-memory LRU cache (per-service, 100 entries, 5-minute TTL)
- L2: Redis (shared, 1-hour TTL)
- L3: PostgreSQL (persistent, for audit trail)
- Fallback: Local cached copy on edge verifiers (survives network outage)

### 8.3 Credential Format & Status List — VP-3

**Credential types:**

| Type | Claims | TTL | Disclosure |
|------|--------|-----|-----------|
| `ResidentBadge` | name, unit, floor_access, role, clearance | 8 hours | name, unit selective |
| `VisitorPass` | name, host, purpose, valid_from, valid_until, floors | 4 hours | name selective |
| `ContractorBadge` | company, role, floors, valid_dates | 12 hours | company selective |
| `EmergencyAccess` | authority, scope, reason | 1 hour | none (all disclosed) |

**Bitstring Status List:**
- One status list per tenant, per credential type
- 16KB per list (131,072 credential slots)
- Published at `https://sahi.my/api/v1/status/{tenant_id}/{type}`
- Signed as a VC by the platform Status List key

### 8.4 Compliance Enforcement — VP-3b

**Rules engine evaluating before credential issuance:**
- Resident identity verified (eKYC via MyDigital ID)
- Unit ownership/tenancy confirmed by property admin
- No outstanding violations or blacklist entries
- Credential limits per resident not exceeded

### 8.5 Business Logic Rules — VP-3c

**Access decision rules evaluated at the edge verifier:**
```rust
struct AccessRule {
    rule_id: Ulid,
    tenant_id: Ulid,
    property_id: Ulid,
    conditions: Vec<Condition>,  // Time window, floor, role, clearance
    action: Action,              // Allow, Deny, AllowWithLog
    priority: u32,               // Higher priority wins on conflict
}

enum Condition {
    TimeWindow { start: NaiveTime, end: NaiveTime, days: Vec<Weekday> },
    FloorAccess { floors: Vec<u32> },
    Role { roles: Vec<String> },
    Clearance { min_level: String },
    CredentialType { types: Vec<String> },
}
```

---

## 9. VAULTPASS — MOBILE WALLET (VP-4)

### 9.1 Architecture

```
apps/wallet/
├── lib/
│   ├── main.dart
│   ├── app/
│   │   ├── app.dart           # MaterialApp, theme, routing
│   │   └── router.dart        # GoRouter configuration
│   ├── features/
│   │   ├── credentials/       # Credential list, detail, presentation
│   │   ├── scanning/          # BLE/NFC scanning and presentation
│   │   ├── onboarding/        # Registration, eKYC, key generation
│   │   └── settings/          # App settings, security
│   ├── services/
│   │   ├── keystore_service.dart   # Platform channel → Keystore/Secure Enclave
│   │   ├── ble_service.dart        # flutter_blue_plus abstraction
│   │   ├── nfc_service.dart        # nfc_manager + flutter_nfc_hce
│   │   ├── crypto_service.dart     # flutter_rust_bridge → crypto-engine
│   │   └── sync_service.dart       # Background credential/status sync
│   └── core/
│       ├── theme/                  # Sahi dark theme (slate-950)
│       ├── i18n/                   # AppLocalizations (EN/MS)
│       └── storage/               # Secure credential storage
├── rust/                           # flutter_rust_bridge Rust source
│   └── src/api/
│       ├── crypto.rs               # SD-JWT sign/verify
│       ├── ble_payload.rs          # Credential serialization
│       └── revocation.rs           # Status list parsing
└── android/ & ios/                 # Platform-specific code
```

### 9.2 Security Architecture

| Concern | Implementation |
|---------|---------------|
| **Key storage** | Platform channels → Android Keystore (StrongBox preferred) / iOS Secure Enclave |
| **Biometric binding** | `local_auth` → BiometricPrompt (Android) / LAContext (iOS). Key has `userAuthenticationRequired`. |
| **Credential storage** | `flutter_secure_storage` (AES encryption, Keystore-wrapped key) |
| **Crypto operations** | All via `flutter_rust_bridge` FFI to Rust `crypto-engine`. Zero Dart crypto. |
| **Root/jailbreak detection** | `freeRASP`: warn user + log event, do NOT block operations |
| **Transport security** | Certificate pinning for API calls (pin leaf + backup), BLE Security Mode 1 Level 2+ |
| **Credential binding** | Hardware-bound key (Android StrongBox / iOS Secure Enclave) — credential is cryptographically bound to a single device |
| **Secure deletion** | On logout or device wipe: all credential keys destroyed via platform Keystore API, local SQLite `VACUUM`'d |

### 9.3 BLE Credential Presentation (<200ms target)

**GATT service design:**
- Service UUID: Custom 128-bit (e.g., `SAHI0001-...`)
- Challenge characteristic (Read): Verifier publishes fresh nonce
- Presentation characteristic (Write): Wallet writes SD-JWT VP
- Result characteristic (Notify): Verifier sends GRANTED/DENIED

**Optimization techniques:**
1. Request `ConnectionPriority.high` immediately (7.5ms interval)
2. Negotiate MTU 512 (fit presentation in 1-2 packets)
3. Bond devices for fast reconnection (<50ms on subsequent connects)
4. Pre-filter scans by service UUID
5. Background scanning: `ScanMode.lowPower` in background, `ScanMode.lowLatency` on detection
6. Geofencing to toggle scanning near registered properties

**iOS background BLE restrictions:**
- iOS terminates BLE scanning in background after ~30 seconds of inactivity
- Mitigation: Use `CBCentralManager` with `CBCentralManagerScanOptionAllowDuplicatesKey: false` and state restoration (`CBCentralManagerOptionRestoreIdentifierKey`)
- Use Core Bluetooth background modes (`bluetooth-central` capability)
- Geofencing (`CLCircularRegion`) triggers foreground wake to re-enable active scanning
- User MUST grant "Always" location permission for geofencing; gracefully degrade to manual scan if denied

### 9.4 NFC Presentation

**Android:** Host Card Emulation (HCE) via `flutter_nfc_hce` or custom `HostApduService`
- NDEF External Type record: domain `sahi.my`, type `vp`
- Payload: CBOR-encoded SD-JWT (40-60% smaller than JSON)

**iOS:** NFC & SE Platform APIs (requires Apple entitlement)
- **Minimum iOS 18.1** required for NFC credential presentation (Core NFC deprecated; new NFC & SE Platform framework)
- Must apply to Apple for NFC entitlement (review takes 4-8 weeks)
- Fallback: Universal Link URL scheme in NDEF record

**QR fallback:** Display VP as QR code on wallet screen, scanned by verifier camera

### 9.5 Performance Targets

| Target | Budget | How achieved |
|--------|--------|-------------|
| Cold start <1.5s | - | Defer non-essential init, `const` constructors, lazy-load BLE/NFC, pre-warm FFI bridge during splash |
| BLE handshake <200ms | 100ms connect + 50ms challenge + 30ms sign + 20ms write | Bonded devices, high priority, MTU 512 |
| Gate decision <2s | 100ms detect + 150ms present + 50ms verify + 1ms revocation + 20ms result | Cached status list, hardware-accelerated crypto |

---

## 10. VAULTPASS — EDGE VERIFIER (VP-5)

### 10.1 Architecture

```
edge/verifier/src/
├── main.rs              # Tokio async runtime, graceful shutdown
├── config.rs            # Site-specific configuration
├── ble/
│   ├── mod.rs
│   ├── advertiser.rs    # BLE GATT server (bluer crate)
│   ├── protocol.rs      # Presentation exchange protocol
│   └── gatt.rs          # Characteristic definitions
├── nfc/
│   ├── mod.rs
│   ├── reader.rs        # PN532/ACR122U reader (pcsc/libnfc-sys)
│   └── apdu.rs          # ISO 14443-4 APDU handling
├── crypto/
│   ├── mod.rs
│   ├── sd_jwt.rs        # SD-JWT verification
│   └── did_resolver.rs  # DID resolution with local cache
├── policy/
│   ├── mod.rs
│   ├── access_rules.rs  # Who can access what, when
│   ├── offline.rs       # Degradation policy (stale cache behavior)
│   └── evaluator.rs     # Rule evaluation engine
├── revocation/
│   ├── mod.rs
│   ├── status_list.rs   # Bitstring status list cache
│   └── sync.rs          # Background sync via MQTT + HTTP polling
├── hardware/
│   ├── mod.rs
│   ├── gpio.rs          # Door lock relay (rppal)
│   ├── led.rs           # Status LEDs
│   ├── buzzer.rs        # Audio feedback
│   ├── display.rs       # SSD1306 OLED (optional)
│   └── tamper.rs        # Tamper switch monitoring
├── audit/
│   ├── mod.rs
│   └── log.rs           # Append-only local log (rusqlite/sled)
└── mqtt/
    ├── mod.rs
    └── client.rs        # rumqttc async MQTT client
```

### 10.2 Hardware

| Component | Purpose | Interface |
|-----------|---------|-----------|
| Raspberry Pi 4/5 or CM4 | Compute | - |
| PN532 NFC module | NFC reader | SPI or I2C |
| Built-in Bluetooth 5.0 | BLE peripheral | HCI |
| Relay module | Door lock actuation | GPIO |
| Status LEDs (green/red/amber) | Visual feedback | GPIO |
| Buzzer | Audio feedback | GPIO |
| Tamper switch | Enclosure security | GPIO |
| RTC module (optional) | Accurate time without NTP | I2C |
| SSD1306 OLED (optional) | Status display | I2C |

### 10.3 Offline Operation

| Cache | TTL | Stale behavior |
|-------|-----|----------------|
| Trust Registry | 4 hours | Allow-list mode (pre-approved holders only) |
| Status Lists | 15 minutes (900s) | Accept if <4 hours old; deny-all if older |
| Issuer DID docs | 24 hours | Use cached; alert if changed unexpectedly |
| Access rules | 1 hour | Use cached rules |

**Fail-closed:** On ANY error (crypto failure, stale cache beyond threshold, tamper detected, unknown credential format), the decision is **DENY**.

### 10.4 Security Hardening

- Full-disk encryption (LUKS) on SD card
- Read-only root filesystem with writable overlay
- Hardware watchdog (10-second pet interval, auto-reboot on hang)
- Remote attestation: device signs health report with device key, sent via MQTT
- Tamper switch triggers: log event, notify platform, enter lockdown (deny-all)
- **Secure boot:** Verify firmware signature on boot (signed with platform Ed25519 key)
- **OTA updates:** Signed firmware bundles delivered via MQTT, verified before applying, dual-partition A/B scheme for rollback
- **Network isolation:** Verifier communicates ONLY with platform API (MQTT + HTTPS) — no other outbound connections allowed (iptables rules)
- **Key storage:** Device key stored in TPM (if available on RPi) or LUKS-encrypted keyfile

---

## 11. VAULTPASS — WEB APPLICATIONS (VP-6 to VP-9)

### 11.1 Admin Portal — VP-6

**Location:** `apps/admin-portal/` (Next.js 14+)

**Features:**
- Tenant/property management dashboard
- Resident management (CRUD, credential issuance)
- Access log visualization (timeline, heatmaps)
- Verifier device management (health, configuration)
- Analytics (entry counts, peak hours, denial rates)
- Multi-tenant workspace switching
- Credential revocation interface

### 11.2 Guard Tablet — VP-7

**Location:** `apps/guard-tablet/` (Next.js PWA)

**Design principles:**
- Tablet-optimized (10" landscape primary)
- Large touch targets (minimum 48x48dp)
- Glanceable status: current visitor queue, recent entries
- Emergency override button (requires biometric + reason code)
- Offline-capable (service worker + cached data)
- Auto-refresh visitor list every 30 seconds

### 11.3 Guest Web — VP-8

**Location:** `apps/guest-web/` (Next.js)

**Flow:**
1. Guest receives invite link (SMS/email/WhatsApp)
2. Opens link → minimal registration form (name, IC/passport, purpose)
3. Biometric liveness check (optional, P2 assurance)
4. Credential issued as QR code + deep link to wallet
5. On arrival: present via wallet or show QR to guard

**Design:** Maximum 3 steps to completion. Progressive disclosure. EN/MS language toggle.

### 11.4 Onboarding & KYC — VP-9

**eKYC integration:** MyDigital ID (Malaysian government identity platform)
- OAuth 2.0 flow with PKCE
- Request minimum claims (name, IC number hash — never raw IC)
- Map verified identity to wallet DID
- Store verification status, not PII

---

## 12. TRUSTMARK — SIGNING SYSTEM (TM-1 to TM-3b)

### 12.1 Signing Orchestrator — TM-1

**State machine:**
```
CREATED → PREPARING → READY_FOR_SIGNATURES → SIGNING_IN_PROGRESS
    → PARTIALLY_SIGNED → FULLY_SIGNED → TIMESTAMPING
    → AUGMENTING_LTV → COMPLETE

(Any state) → ABORTED
ABORTED → RESUMING → (previous state, with document hash verification)
```

**Rules:**
- Every state transition logged to Merkle tree (F2)
- State persisted in PostgreSQL with optimistic locking (version column)
- Ceremony TTL: 72 hours (configurable), auto-abort on expiry
- Multi-party signing: sequential or parallel order (configurable)
- Each signer slot has sub-state: PENDING → INVITED → AUTHENTICATED → SIGNED | DECLINED

### 12.2 PDF Engine — TM-2

**Custom `trustmark-pades` crate composing:**
- `lopdf` — PDF object manipulation, incremental saves
- `cms` (RustCrypto) — CMS/PKCS#7 SignedData construction
- `x509-cert` (RustCrypto) — Certificate handling
- RFC 3161 TSA client — Timestamp authority requests

**IMPORTANT: No production-ready Rust crate exists for PAdES-B-LTA.** This is a custom build from primitives. The progressive augmentation pipeline (B-T → B-LT → B-LTA) requires careful implementation of:
1. PDF incremental save with signature ByteRange
2. CMS SignedData with signed attributes (signing-time, message-digest, content-type)
3. RFC 3161 timestamp embedding in unsigned attributes
4. DSS (Document Security Store) dictionary for OCSP/CRL responses (PAdES-B-LT)
5. Document timestamp via VRI (Validation Related Information) dictionary (PAdES-B-LTA)
6. **Validation:** Test against Adobe Acrobat, Foxit, and EU DSS validator (dss.esig.europa.eu)

**Progressive augmentation pipeline:**
1. **At signing (real-time, <1s):** Sign + timestamp → PAdES-B-T
2. **Async job (minutes):** Fetch OCSP/CRL → augment to PAdES-B-LT
3. **Batch job (24h):** Add document timestamp + VRI → augment to PAdES-B-LTA
4. **Periodic (yearly):** Re-timestamp for algorithm longevity

### 12.3 Signature Appearance — TM-2b

- Visual signature block embedded in PDF at designated coordinates
- Shows: signer name, timestamp, certification reference, QR code linking to verification page
- Follows Sahi design system (Plus Jakarta Sans, monochrome, minimal)

### 12.4 WebAuthn Integration — TM-3

**Assurance level mapping:**

| TrustMark Level | NIST AAL | Authenticator | Key type | Identity verification |
|----------------|----------|---------------|----------|----------------------|
| P1 (Basic) | AAL1 | Any passkey (synced OK) | User presence | Email/phone verified |
| P2 (Enhanced) | AAL2 | Platform/roaming passkey | User verification (biometric) | eKYC (MyDigital ID) |
| P3 (Qualified) | AAL3 | Hardware-bound roaming key | Non-exportable, attested | CA-issued certificate, in-person |

**Implementation:**
- `userVerification: "required"` for all signing ceremonies
- Enforce attestation for P2/P3 (verify against FIDO MDS)
- Store AAGUID to track authenticator types per signer
- Register multiple passkeys per user (platform + roaming backup)
- Conditional UI (`mediation: "conditional"`) for repeat signing
- **Ceremony-credential binding:** WebAuthn challenge includes `ceremony_id + document_hash` to cryptographically bind the authentication to the specific document being signed. Prevents authentication replay across ceremonies.
- **Cross-origin protection:** `rpId` set to `sahi.my`; all signing UIs hosted on subdomains of `sahi.my`

### 12.5 Assurance Profiles — TM-3b

- P1: Self-asserted identity, email verification, synced passkey OK
- P2: eKYC-verified identity (MyDigital ID), biometric passkey
- P3: CA-issued certificate (from MCMC-licensed CA), hardware security key, in-person proofing

---

## 13. TRUSTMARK — LABELS & VERIFICATION (TM-4 to TM-7)

### 13.1 COSE Token Signer — TM-4

**Token structure (COSE_Sign1):**
```
Protected headers: { algorithm: ES256, content_type: "application/sahi-token+cbor" }
Unprotected headers: { key_id: "KEY_01HXK...", expiry: "2027-03-10T00:00:00Z" }
Payload (CBOR): {
    product_id: "PRD_01HXK...",
    certification_ref: "CERT_01HXK...",
    batch: "BATCH_01HXK...",
    issuer_did: "did:web:issuer.sahi.my",
    issued_at: "2026-03-10T12:00:00Z",
    valid_until: "2027-03-10T00:00:00Z",
    claims: { type: "halal", body: "JAKIM", scope: "food-processing" }
}
Signature: ES256 (ECDSA P-256) via HSM
```

**Size:** ~200-400 bytes in CBOR. Fits in QR (Version 10+) and NFC (NTAG 424 DNA with 256 bytes).

**Encoding for transport:**
- QR: Base45 encoding (proven by EU Digital COVID Certificate)
- NFC: Raw CBOR bytes in NDEF record
- URL: Base64url as query parameter

### 13.2 Physical Label Design

**Dual-layer authentication:**
1. **Layer 1 — COSE_Sign1 token (logical):** Cryptographic proof of claims. Works on any QR code or NFC tag.
2. **Layer 2 — NTAG 424 DNA (hardware, optional):** SUN (Secure Unique NFC) messages with rolling counter and AES-128 CMAC. TagTamper for physical seal detection.

**NTAG 424 SUN limitation:** SUN verification requires an **online** backend call (the CMAC is validated against the tag's server-side symmetric key). This means Layer 2 hardware authentication is NOT available offline — only Layer 1 (COSE_Sign1) works offline. Design the verification flow to clearly distinguish:
- **Offline:** COSE token verified → "Cryptographically valid" (green checkmark)
- **Online + SUN:** COSE token + NTAG 424 SUN verified → "Hardware-authenticated" (green checkmark + shield icon)

**Label layout:**
- QR code (contains COSE token as Base45)
- NFC tap zone (with NTAG 424 DNA for high-assurance products)
- Short URL for manual verification: `verify.sahi.my/v/{short-code}`

### 13.3 Verification Service — TM-5

**Endpoint:** `GET /verify/{token}` → Verification landing page

**Verification flow:**
1. Deserialize COSE_Sign1 token
2. Resolve issuer DID → fetch public key
3. Verify ES256 signature
4. Check token expiry
5. Check revocation via Status List (if online)
6. If NTAG 424 DNA: validate SUN message + check tap counter
7. Return verification result with confidence level

**Landing page (<1.5s first meaningful paint):**
- Green checkmark: Valid, verified, unexpired
- Amber warning: Valid but nearing expiry, or first scan
- Red alert: Invalid, expired, tampered, or revoked
- Collapsible details: Certification body, dates, batch, scan count

### 13.4 Issuer Console — TM-6

**Location:** `apps/trustmark-web/` (Next.js)

**Features:**
- Create signing ceremonies (single and batch)
- Manage signer invitations and roles
- Upload documents for signing
- Track ceremony progress (state machine visualization)
- Issue COSE tokens for physical labels
- Batch label management (generate, track, revoke)
- Certificate lifecycle dashboard

### 13.5 Supply Chain Controls — TM-6b

- Tag provisioning: Unique AES keys per NFC tag, registered in platform
- Binding: Each tag UID linked to specific product/certificate via COSE token
- Chain of custody: Every scan (tap counter increment) logged
- Tamper detection: TagTamper status checked on every scan
- Anomaly detection: Unusual scan patterns (geographic, frequency) flagged

### 13.6 Detached Evidence Bundles — TM-7

For offline or independent verification, package all evidence as a ZIP bundle:
```
evidence-bundle-{ceremony_id}.zip
├── document.pdf                    # Signed PDF (PAdES-B-LTA)
├── ceremony-log.json               # Full ceremony event log
├── merkle-proof.json               # Inclusion proof for ceremony events
├── certificates/                   # Signer certificates
│   ├── signer-1.pem
│   └── signer-2.pem
├── ocsp-responses/                 # OCSP responses for all certs
├── timestamps/                     # TSA receipts
└── manifest.json                   # Bundle metadata + integrity hash
```

---

## 14. UI/UX DESIGN SYSTEM

### 14.1 Design Tokens

**Typography:**
```css
--font-primary: 'Plus Jakarta Sans', system-ui, sans-serif;
--font-mono: 'JetBrains Mono', 'Fira Code', monospace;
--font-size-xs: 0.75rem;   /* 12px */
--font-size-sm: 0.875rem;  /* 14px */
--font-size-base: 1rem;    /* 16px */
--font-size-lg: 1.125rem;  /* 18px */
--font-size-xl: 1.25rem;   /* 20px */
--font-size-2xl: 1.5rem;   /* 24px */
```

**Colors (monochrome-first):**
```css
/* 95% of the UI uses these */
--slate-50: #f8fafc;    --slate-100: #f1f5f9;
--slate-200: #e2e8f0;   --slate-300: #cbd5e1;
--slate-400: #94a3b8;   --slate-500: #64748b;
--slate-600: #475569;   --slate-700: #334155;
--slate-800: #1e293b;   --slate-900: #0f172a;
--slate-950: #020617;

/* Signal colors — ONLY for semantic meaning */
--signal-success: #10B981;  /* green-500: granted, valid, active */
--signal-error: #EF4444;    /* red-500: denied, invalid, revoked */
--signal-warning: #F59E0B;  /* amber-500: expiring, degraded, attention */
```

**Elevation:** Border-only (`1px solid var(--slate-200)`). No drop shadows except modal overlays.

### 14.2 Component Architecture (shadcn/ui)

```
packages/ui/
├── components/
│   ├── credential-card.tsx    # Credential display card
│   ├── verification-badge.tsx # Verification result display
│   ├── status-indicator.tsx   # Active/expired/revoked indicator
│   ├── ceremony-stepper.tsx   # Signing ceremony progress
│   ├── data-table.tsx         # Sortable, filterable table
│   ├── metric-card.tsx        # Dashboard metric display
│   └── ... (shadcn/ui base components)
├── hooks/
│   ├── use-tenant.ts          # Current tenant context
│   ├── use-credentials.ts     # Credential data fetching
│   └── use-realtime.ts        # MQTT/WebSocket subscriptions
└── lib/
    ├── api/                   # Typed fetch functions
    ├── i18n/                  # next-intl configuration
    └── utils.ts               # Shared utilities
```

### 14.3 Application-Specific UX

**Admin Portal:** Dashboard-first. Left sidebar navigation. Cards for key metrics. Data tables for management. Modal forms for CRUD. Workspace switcher for multi-tenant.

**Guard Tablet:** Full-screen, landscape. Split view: visitor queue (left 60%) + current status (right 40%). Large touch targets. Auto-refresh. Emergency override with biometric + reason.

**Guest Web:** Mobile-first, single-column. Maximum 3 steps. Language toggle (EN/MS) at top. Progressive disclosure. No jargon in primary UI.

**Verification Landing:** Center-aligned result. Large icon (checkmark/X). Collapsible technical details. <1.5s first meaningful paint via CloudFront + static generation.

**Wallet (Flutter):** Dark mode only (slate-950). Credential cards with subtle gradients. Presentation flow: detect → biometric → present → result (auto-dismiss 3s). One primary action per screen.

### 14.4 Internationalization

- `next-intl` for web apps (EN/MS JSON files)
- `AppLocalizations` for Flutter wallet
- Language code: `ms` (ISO 639-1), never `bm`
- Both `en` and `ms` keys must be present for every string
- No em dash in any string
- Errors always include: what happened + what to do next

### 14.5 Accessibility (WCAG 2.2 AA)

- Minimum contrast ratio: 4.5:1 (normal text), 3:1 (large text)
- Touch targets: minimum 48x48dp
- Focus management: visible focus ring on all interactive elements
- Screen reader labels on all credential cards and actions
- Keyboard navigation for all flows
- Color-blind safe: never rely on color alone for information
- Reduced motion: respect `prefers-reduced-motion`

---

## 15. SECURITY ARCHITECTURE

### 15.1 STRIDE Threat Model (Per Component)

| Component | S (Spoofing) | T (Tampering) | R (Repudiation) | I (Info Disclosure) | D (Denial of Service) | E (Elevation of Privilege) |
|-----------|---|---|---|---|---|---|
| **KMS** | HSM auth | HSM tamper resistance | Audit log | Key isolation | Rate limiting | RBAC on key ops |
| **API Gateway** | JWT validation | TLS + request signing | Request logging | No PII in responses | Rate limiting per tenant | Tenant isolation (RLS) |
| **Edge Verifier** | Credential verification | Tamper switch + LUKS | Local audit log | Minimal data stored | Watchdog + fail-closed | No privilege escalation (single-purpose) |
| **Wallet** | Biometric + Secure Enclave | App integrity (freeRASP) | Presentation logging | Encrypted storage | BLE timeout | Sandboxed app |
| **Signing Service** | WebAuthn + HSM | PAdES integrity + Merkle | TSA timestamps + audit | Document encryption | Queue-based processing | Role-based access |

### 15.2 Security Hardening Checklist (Per Module)

- [ ] Input validation: all API inputs via Axum typed extractors
- [ ] Authentication: JWT or mTLS for every endpoint
- [ ] Authorization: RLS for data, RBAC for operations
- [ ] No PII in logs: only opaque ULIDs
- [ ] No `.unwrap()` in production code
- [ ] `cargo audit` passes with zero vulnerabilities
- [ ] Rate limiting configured per endpoint
- [ ] Error responses reveal no internal details
- [ ] HTTPS only (HSTS enabled)
- [ ] CORS restricted to known origins
- [ ] CSP headers on all web responses
- [ ] SQL injection: impossible (SQLx compile-time verification)
- [ ] XSS: React auto-escaping + CSP
- [ ] CSRF: SameSite cookies + CSRF tokens

### 15.3 Incident Response

- Credential revocation: immediate via Status List update (propagates within 900s TTL)
- Key compromise: HSM key destruction + new key generation + re-issuance of affected credentials
- Edge verifier compromise: remote lockdown via MQTT + physical tamper alert
- Data breach: PDPA notification within 72 hours + tenant notification
- **Replay attack prevention:** All BLE presentations include challenge nonce + 5-second expiry KB-JWT. Edge verifier maintains a nonce dedup cache (LRU, 1000 entries, 60-second TTL) to reject replayed presentations.
- **Rate limiting at gate:** Max 5 presentation attempts per BLE connection (prevents brute-force scanning). Lockout for 30 seconds after 3 consecutive denials from same device.

### 15.4 Supply Chain Security

- **SBOM generation:** `syft` generates SBOM on every CI build (SPDX format)
- **Dependency pinning:** `Cargo.lock` committed; `pnpm-lock.yaml` committed
- **Container signing:** All production images signed with `cosign` (Sigstore)
- **Third-party audit:** Crypto-engine crate reviewed by external security firm before Phase 2 exit

---

## 16. PERFORMANCE ENGINEERING

### 16.1 Performance Budgets

| Operation | Target | How measured | How achieved |
|-----------|--------|-------------|-------------|
| BLE credential presentation | <200ms | Device to verifier handshake complete | Bonded connections, MTU 512, hardware crypto |
| Gate entry decision | <2s E2E | BLE detect → granted/denied display | Cached revocation, edge-local rules, GPIO direct |
| SD-JWT verification | <50ms | Crypto verify only | ring crate (hardware AES-NI), no network |
| TrustMark signing ceremony | <1s overhead | Platform processing only | Async PAdES augmentation, pre-computed hashes |
| COSE token generation | <100ms | Sign + encode | ES256 via ring, CBOR via ciborium |
| Verification landing page | <1.5s | First meaningful paint | CloudFront CDN, static generation, lazy load details |
| API p99 response | <500ms | Under 100 concurrent | Connection pooling, Redis caching, indexed queries |
| Wallet cold start | <1.5s | To credential home screen | Deferred init, const constructors, lazy BLE/NFC |

### 16.2 Optimization Strategies

**Backend:**
- SQLx connection pool: 20 connections per service
- Redis caching: DID docs, access rules, tenant config (5-minute TTL)
- Prepared statement caching in SQLx
- Streaming responses for large data sets
- Async processing for non-real-time operations (PAdES augmentation)

**Frontend:**
- Next.js static generation for verification pages
- React Query for data fetching with stale-while-revalidate
- Code splitting per route
- Image optimization via Next.js `<Image>`
- Font loading: `font-display: swap` with preload

**Mobile:**
- Flutter Impeller rendering engine
- `ZeroCopyBuffer` for FFI data transfer
- Lazy initialization of BLE/NFC services
- Background isolates for heavy processing

**Edge:**
- In-memory LRU cache for recent verifications
- Persistent sled/rusqlite for status lists
- Pre-computed access rule evaluations
- GPIO direct control (no abstraction overhead)

---

## 17. TESTING STRATEGY

### 17.1 Testing Pyramid

```
        ┌──────────┐
        │   E2E    │  Playwright (web), Flutter integration (wallet)
        │  5-10%   │  Full signing ceremonies, gate entry flows
        ├──────────┤
        │ Integr.  │  testcontainers (Postgres, Redis), HTTP client tests
        │ 20-30%   │  API contract tests, RLS verification, MQTT flows
        ├──────────┤
        │  Unit    │  cargo test, Jest
        │ 60-70%   │  Crypto operations, rule evaluation, state machines
        └──────────┘
```

### 17.2 Testing by Component

| Component | Tool | Key test scenarios |
|-----------|------|-------------------|
| Crypto engine | `cargo test` + proptest | SD-JWT roundtrip, selective disclosure, invalid signatures, edge cases |
| KMS | `cargo test` + mockall | Key generation, signing, rotation, audit events |
| Merkle log | `cargo test` + proptest | Append, inclusion proof, consistency proof, tamper detection |
| API endpoints | `cargo test --test integration` + testcontainers | Happy path, auth failures, RLS isolation, rate limiting |
| RLS policies | SQLx integration tests | Cross-tenant access denied, same-tenant access allowed |
| Edge verifier | `cargo test` + hardware-in-loop (CI optional) | BLE protocol, offline behavior, tamper response |
| Web apps | Jest + Playwright | Form validation, i18n, accessibility, signing flow |
| Wallet | Flutter test + integration test | Credential storage, BLE presentation, biometric mock |
| Performance | k6 (API), criterion (Rust) | p99 latency under load, throughput limits |
| Security | cargo audit + OWASP ZAP | Dependency vulnerabilities, API security scan |
| Chaos/Resilience | Custom scripts | Network partition (verifier offline), HSM unavailable, DB failover |
| Load | k6 + Grafana | 1000 concurrent gate entries, 100 concurrent signing ceremonies |
| Fuzzing | cargo-fuzz (libFuzzer) | SD-JWT parser, COSE deserializer, PDF signature parser |

### 17.3 Property-Based Testing

Use `proptest` for cryptographic code:
```rust
proptest! {
    #[test]
    fn sd_jwt_roundtrip(claims in arbitrary_claims()) {
        let issuer_key = generate_test_key();
        let credential = issue_sd_jwt(&claims, &issuer_key)?;
        let presentation = derive_presentation(&credential, &disclosed_subset)?;
        let verified = verify_presentation(&presentation, &issuer_key.public())?;
        assert_eq!(verified.disclosed_claims, disclosed_subset);
    }
}
```

---

## 18. CI/CD PIPELINE

### 18.1 Pipeline Stages

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]

jobs:
  rust:
    steps:
      - cargo fmt --check
      - cargo clippy -- -D warnings
      - cargo test
      - cargo sqlx prepare --check  # Verify SQL metadata up to date
      - cargo audit                 # Security vulnerability check
      - cargo build --release

  typescript:
    steps:
      - pnpm install --frozen-lockfile
      - turbo run lint
      - turbo run type-check
      - turbo run test
      - turbo run build

  integration:
    needs: [rust, typescript]
    services:
      postgres: { image: postgres:16, ... }
      redis: { image: redis:7, ... }
    steps:
      - cargo test --test integration
      - turbo run test:integration

  security:
    steps:
      - cargo audit
      - pnpm audit
      - trivy image scan
      - SBOM generation (syft)

  docker:
    needs: [integration, security]
    steps:
      - Build multi-stage images
      - Push to ECR
      - Deploy to staging (blue-green)
      - Smoke tests
      - [Manual gate] Deploy to production
```

### 18.2 Quality Gates (Must Pass Before Merge)

- `cargo clippy -- -D warnings` — zero warnings
- `cargo test` — all tests pass
- `cargo sqlx prepare --check` — SQL metadata current
- `cargo audit` — no known vulnerabilities
- TypeScript strict mode — no errors
- All unit + integration tests pass
- No decrease in code coverage on critical paths

---

## 19. OBSERVABILITY & MONITORING

### 19.1 Logging

**Structured JSON logging via `tracing` crate:**
```json
{
  "timestamp": "2026-03-10T12:00:00.000Z",
  "level": "INFO",
  "span": "platform_api::handlers::credentials",
  "message": "credential_issued",
  "tenant_id": "TNT_01HXK...",
  "credential_id": "CRD_01HXK...",
  "request_id": "REQ_01HXK...",
  "duration_ms": 45
}
```

**Rules:** No PII in logs. Use opaque ULIDs only. Log all security events. Log all state transitions.

### 19.2 Metrics (Prometheus)

| Metric | Type | Labels |
|--------|------|--------|
| `sahi_http_requests_total` | Counter | method, path, status, tenant |
| `sahi_http_request_duration_seconds` | Histogram | method, path, tenant |
| `sahi_credential_operations_total` | Counter | operation (issue/verify/revoke), tenant |
| `sahi_gate_decisions_total` | Counter | decision (grant/deny), verifier, tenant |
| `sahi_gate_decision_duration_seconds` | Histogram | verifier |
| `sahi_kms_operations_total` | Counter | operation, success |
| `sahi_merkle_log_entries_total` | Counter | event_type, tenant |
| `sahi_ceremony_state_transitions` | Counter | from_state, to_state, tenant |
| `sahi_status_list_age_seconds` | Gauge | verifier |

### 19.3 Distributed Tracing

OpenTelemetry integration:
- Every API request gets a trace ID (propagated via `traceparent` header)
- Spans for: HTTP handler, database query, Redis cache, KMS operation, MQTT publish
- Trace context propagated through MQTT to edge verifiers
- Export to Jaeger or AWS X-Ray

### 19.4 Grafana Dashboards

1. **Platform Overview:** Request rate, error rate, p99 latency, active tenants
2. **VaultPass Operations:** Gate decisions/min, grant/deny ratio, BLE latency, offline verifier count
3. **TrustMark Operations:** Active ceremonies, signing throughput, augmentation queue depth
4. **Security:** Failed auth attempts, revocation rate, tamper alerts, KMS operations
5. **Infrastructure:** CPU/memory per service, database connections, Redis hit rate, MQTT message rate

### 19.5 Alerting

| Alert | Condition | Severity |
|-------|-----------|----------|
| High error rate | >5% 5xx responses over 5 minutes | Critical |
| Slow API responses | p99 >1s over 10 minutes | Warning |
| Edge verifier offline | No heartbeat for 5 minutes | Critical |
| Status list stale | Cache age >2 hours on any verifier | Warning |
| KMS failure | Any sign/verify failure | Critical |
| Tamper detected | Tamper switch triggered | Critical |
| Database connection exhaustion | >90% pool utilization | Warning |
| Certificate expiry | <30 days until expiry | Warning |

---

## 20. COMPLIANCE & LEGAL

### 20.1 Malaysian Legal Framework

| Law | Requirement | Implementation |
|-----|------------|----------------|
| **Digital Signature Act 1997** | Asymmetric crypto + MCMC-licensed CA for full legal weight | Integrate with MSC Trustgate/Pos Digicert for P3 signatures |
| **Electronic Commerce Act 2006** | Electronic signatures are valid for most transactions | P1/P2 signatures qualify under this act |
| **PDPA 2010** | Consent, purpose limitation, data minimization, security | No PII in logs, 7-year retention, Malaysia-resident processing |
| **Computer Crimes Act 1997** | Unauthorized access is criminal | RLS, audit logs, fail-closed design |

### 20.2 eIDAS 2.0 (EU Market Readiness)

- Accept external qualified certificates from QTSPs
- Prepare for EUDI wallet integration (2026-2027)
- PAdES-B-LTA signatures are eIDAS-recognized AdES
- Do NOT claim to be a QTSP unless certified

### 20.3 Halal Certification Alignment

- JAKIM MYeHALAL integration readiness
- COSE tokens on halal product labels linking to signed certificates
- Phase 4 pilot target: 10,000 products with NFC-authenticated halal labels
- HADIC (Halal Digital Chain) blockchain interoperability planned

### 20.4 Data Residency

- All production data processed in Malaysia or Singapore (ap-southeast-1)
- PostgreSQL RDS in ap-southeast-1
- CloudHSM in ap-southeast-1
- CDN edge caches for verification pages (read-only, no PII)

---

## 21. DEPLOYMENT & INFRASTRUCTURE

### 21.1 Environment Promotion

```
Local (docker-compose)
  → CI (GitHub Actions + testcontainers)
    → Staging (ECS Fargate, separate AWS account)
      → Production (ECS Fargate, production account)
```

### 21.2 Blue-Green Deployment

- ECS services updated with new task definition
- ALB target group switches traffic to new tasks
- Health check: `/health` endpoint (checks DB, Redis, KMS connectivity)
- Rollback: revert to previous task definition if health checks fail
- Zero-downtime migrations: backward-compatible schema changes only

### 21.3 Disaster Recovery

| Component | RPO | RTO | Strategy |
|-----------|-----|-----|----------|
| PostgreSQL | <1 min | <15 min | Multi-AZ RDS, automated failover, daily snapshots, PITR |
| Redis | <5 min | <5 min | ElastiCache Multi-AZ with automatic failover |
| CloudHSM | 0 | <30 min | Multi-AZ cluster, key backup to S3 (encrypted) |
| Application | 0 | <5 min | ECS auto-restart, multi-AZ task placement |
| Edge verifiers | N/A | Self-healing | Watchdog reboot, cached data persists on LUKS volume |
| Signed documents | 0 | <1 hour | S3 with cross-region replication, versioning enabled |

---

## 22. PHASE EXECUTION TIMELINE

### Phase 0 — Shared Foundation (Weeks 1-4)

| Week | Module | Deliverables |
|------|--------|-------------|
| 1 | F5 (Infra) | docker-compose, CI pipeline, Terraform skeleton |
| 1-2 | F1 (KMS) | KmsProvider trait, SoftwareKmsProvider, key generation/signing/rotation, audit events |
| 2 | F2 (Merkle) | Merkle tree, append-only log, inclusion/consistency proofs, publication stubs |
| 2-3 | F4 (Tenants) | Tenant CRUD, RLS policy factory, ALL DDL migrations (VP + TM tables) |
| 3 | F3 (Gateway) | Axum middleware stack (auth, rate limit, metering, RLS setup) |
| 3-4 | F6 (Trust Registry) | Registry data model, DID publication, verifier registration |
| 4 | Cross-cutting | Error codes (Appendix E), ULID factory (Appendix G), i18n infra, STRIDE document |

**Phase 0 exit criteria:** All F1-F6 complete, docker-compose services healthy, CI green, all DDL migrations applied, STRIDE document drafted.

### Phase 1 — VaultPass MVP (Weeks 5-20)

| Weeks | Modules | Focus |
|-------|---------|-------|
| 5-8 | VP-1, VP-2, VP-3, VP-3b, VP-3c | Crypto engine, DID resolution, credential format, compliance rules, business rules |
| 9-12 | VP-4, VP-5 | Wallet app (Flutter + BLE + NFC), edge verifier (Rust + RPi) |
| 13-16 | VP-6, VP-7, VP-8 | Admin portal, guard tablet, guest web |
| 17-20 | VP-9, integration | Onboarding/KYC, full E2E testing, performance tuning |

### Phase 2 — VaultPass Hardening (Weeks 21-26)

External pen test, PDPA audit, performance optimization, BBS+ stub implementation, pilot deployment.

### Phase 3 — TrustMark MVP (Weeks 24-38)

| Weeks | Modules | Focus |
|-------|---------|-------|
| 24-27 | TM-1, TM-3 | Signing orchestrator, WebAuthn integration |
| 28-31 | TM-2, TM-2b, TM-3b | PDF engine (PAdES-B-LTA), signature appearance, assurance profiles |
| 32-35 | TM-4, TM-5, TM-6 | COSE tokens, verification service, issuer console |
| 36-38 | TM-6b, TM-7 | Supply chain controls, detached evidence bundles |

### Phase 4 — TrustMark Hardening (Weeks 39-42)

Pen test, red team (14 scenarios), claims audit, halal pilot, capacity planning.

---

## 23. RISK MITIGATION

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Rust PAdES ecosystem immature | High | High | Build custom crate from `lopdf` + `cms` primitives; validate against EU DSS validator; accept 2-3 week additional development time |
| BLE <200ms on all devices | Medium | High | Bonded connections + MTU negotiation; QR fallback for slow devices |
| Apple NFC entitlement denied | Medium | Medium | BLE primary, NFC secondary; QR tertiary fallback |
| CloudHSM cost | Low | Low | AWS KMS for staging; CloudHSM only for production |
| Rust compilation times | High | Low | Incremental compilation, `cargo-watch`, sccache in CI |
| MCMC-licensed CA integration | Medium | Medium | Start integration early (Phase 1); use self-signed for dev/staging |
| SD-JWT spec changes | Low | Medium | Abstract via `CredentialProof` trait; spec is now RFC-stable |
| Multi-tenant RLS bypass | Low | Critical | Automated RLS tests in CI; penetration test in Phase 2 |
| Edge verifier physical tampering | Medium | High | Tamper switch + LUKS + watchdog + remote attestation |
| Key compromise | Low | Critical | HSM isolation + key rotation + immediate revocation capability |
| NTAG 424 SUN requires online | Medium | Medium | Layer 1 (COSE) works offline; SUN is additive assurance. UX clearly separates verification levels |
| iOS NFC entitlement delay | Medium | Medium | Apply during Phase 1 week 1; BLE is primary transport regardless |
| pgBouncer + RLS interaction | Medium | High | Use SET LOCAL exclusively; integration tests verify tenant isolation through connection pool |
| Certificate Transparency log acceptance | Medium | Low | Use Google Argon or Let's Encrypt Oak logs; fallback to independent TSA only |

---

## 24. CROSS-CUTTING CONCERNS

### 24.1 Error Code Registry

All errors use `SAHI_XXXX` codes from Appendix E. Never invent ad hoc codes. If a new code is needed, add it to the registry first.

### 24.2 ULID Convention

All IDs are ULIDs with registered prefixes from Appendix G:
```
TNT_ — Tenant
USR_ — User
CRD_ — Credential
PRD_ — Product
KEY_ — Key handle
LOG_ — Log entry
EVT_ — Event
REQ_ — Request
SVC_ — Service
CRM_ — Ceremony
VRF_ — Verifier
PRY_ — Property
```

### 24.3 Time Format

- All timestamps: RFC 3339 (`2026-03-10T12:00:00Z`)
- All database columns: `timestamptz` (never `timestamp`)
- All durations: ISO 8601 (`PT15M`, `P1D`)
- Timezone: UTC everywhere internally; local timezone only in UI display layer

### 24.4 Configuration

- All config from environment variables (never hardcoded)
- `.env.example` documents all variables
- `.env` is gitignored
- Production: AWS Secrets Manager
- Staging: AWS SSM Parameter Store
- Local: `.env` file

### 24.5 API Versioning

- Base path: `/api/v1/` for all endpoints
- Breaking changes increment version: `/api/v2/`
- Old versions supported for minimum 6 months after deprecation notice
- Version negotiation via URL path (not headers) for simplicity
- OpenAPI spec generated per version, published at `/api/v1/openapi.json`

### 24.6 Database Migration Safety

- All migrations are **reversible** (UP + DOWN scripts)
- **Expand-contract pattern** for breaking schema changes:
  1. Expand: add new column/table alongside old
  2. Migrate data: background job copies data
  3. Contract: remove old column/table after all services updated
- **No lock-contention migrations:** Never `ALTER TABLE ... ADD COLUMN ... DEFAULT ...` on large tables (locks entire table). Use `ADD COLUMN` then backfill.
- **SQLx offline mode:** `cargo sqlx prepare` generates query metadata for CI (no live database required for compilation)
- Migration naming: `YYYYMMDDHHMMSS_description.sql`

### 24.7 Graceful Degradation

| Dependency down | Behavior |
|-----------------|----------|
| PostgreSQL | Platform API returns 503; edge verifiers continue with cached data |
| Redis | Fall through to PostgreSQL queries (slower but functional) |
| CloudHSM | Queue signing requests; reject new key generation; existing verifications continue (software verify) |
| MQTT broker | Edge verifiers poll HTTPS endpoints for status list/trust registry updates |
| Internet (edge) | Full offline mode — cached trust material used, events queued locally |

### 24.8 Documentation Structure

```
docs/
├── MASTER_PLAN.md         ← this file
├── stride-threat-model.md ← per-component STRIDE analysis
├── api-reference/         ← OpenAPI specs per service
├── architecture/          ← Architecture Decision Records (ADRs)
└── runbooks/              ← Operational procedures
```

---

## APPENDIX A: CRATE DEPENDENCY MAP

```
crypto-engine
  ├── ring 0.17+          (TLS, ECDSA, Ed25519, SHA-256, random)
  ├── argon2               (password hashing)
  ├── blake2               (integrity hashing)
  ├── sd-jwt-rs            (SD-JWT operations)
  ├── bbs [Phase 2]        (BBS+ signatures)
  ├── coset                (COSE token operations)
  ├── ciborium             (CBOR serialization)
  ├── thiserror            (typed errors)
  ├── serde + serde_json   (serialization)
  ├── tokio                (async runtime)
  └── tracing              (structured logging)

platform-api
  ├── axum 0.8+            (web framework)
  ├── sqlx 0.8+            (database, compile-time SQL)
  ├── tower + tower-http   (middleware)
  ├── redis                (caching)
  ├── crypto-engine        (workspace dependency)
  └── [standard: serde, tracing, tokio, thiserror, chrono, ulid]

edge-verifier
  ├── bluer                (BLE GATT server)
  ├── pcsc / libnfc-sys    (NFC reader)
  ├── rppal                (GPIO)
  ├── ssd1306              (OLED display)
  ├── rumqttc              (MQTT client)
  ├── rusqlite / sled      (embedded database)
  ├── crypto-engine        (workspace dependency)
  └── [standard: tokio, serde, tracing, thiserror]
```

## APPENDIX B: FLUTTER PACKAGE MAP

```
wallet (Flutter)
  ├── flutter_rust_bridge 2+  (FFI to Rust crypto-engine)
  ├── flutter_blue_plus       (BLE central)
  ├── nfc_manager             (NFC read)
  ├── flutter_nfc_hce         (NFC HCE — Android)
  ├── flutter_secure_storage  (encrypted credential storage)
  ├── local_auth              (biometric prompt)
  ├── freerasp                (jailbreak/root detection)
  ├── sqflite                 (local SQLite for queryable data)
  ├── go_router               (navigation)
  ├── riverpod / bloc         (state management)
  └── flutter_foreground_task (background BLE — Android)
```

---

## APPENDIX E: ERROR CODE REGISTRY

All errors follow the pattern `SAHI_XXXX` where X is a 4-digit code.

| Range | Domain | Example |
|-------|--------|---------|
| 1000-1099 | Authentication | `SAHI_1001` JWT expired, `SAHI_1002` Invalid signature, `SAHI_1003` WebAuthn failed |
| 1100-1199 | Authorization | `SAHI_1100` Tenant mismatch, `SAHI_1101` Insufficient role, `SAHI_1102` RLS violation |
| 2000-2099 | KMS | `SAHI_2001` Key not found, `SAHI_2002` Sign failed, `SAHI_2003` HSM unreachable |
| 2100-2199 | Credential | `SAHI_2100` Invalid SD-JWT, `SAHI_2101` Revoked, `SAHI_2102` Expired |
| 2200-2299 | Merkle Log | `SAHI_2200` Sequence gap, `SAHI_2201` Root mismatch, `SAHI_2202` Append failed |
| 3000-3099 | Signing Ceremony | `SAHI_3001` Ceremony expired, `SAHI_3002` Invalid transition, `SAHI_3003` Document hash mismatch |
| 3100-3199 | PAdES | `SAHI_3100` PDF parse error, `SAHI_3101` TSA unreachable, `SAHI_3102` Augmentation failed |
| 4000-4099 | COSE / Labels | `SAHI_4001` Token expired, `SAHI_4002` SUN validation failed, `SAHI_4003` Tag tampered |
| 5000-5099 | Gate / Verifier | `SAHI_5001` Credential denied, `SAHI_5002` Offline stale cache, `SAHI_5003` Tamper detected |
| 9000-9099 | Internal | `SAHI_9001` Database error, `SAHI_9002` Redis error, `SAHI_9003` Configuration error |

**Error response format:**
```json
{
  "error": {
    "code": "SAHI_2101",
    "message": "Credential has been revoked",
    "action": "Request a new credential from your property administrator",
    "request_id": "REQ_01HXK..."
  }
}
```

## APPENDIX G: ULID PREFIX REGISTRY

| Prefix | Entity | Example |
|--------|--------|---------|
| `TNT_` | Tenant | `TNT_01HXK4Y5J6P8M2N3Q7R9S0T1` |
| `USR_` | User | `USR_01HXK...` |
| `CRD_` | Credential | `CRD_01HXK...` |
| `PRD_` | Product | `PRD_01HXK...` |
| `KEY_` | Key Handle | `KEY_01HXK...` |
| `LOG_` | Log Entry | `LOG_01HXK...` |
| `EVT_` | Event | `EVT_01HXK...` |
| `REQ_` | Request | `REQ_01HXK...` |
| `SVC_` | Service | `SVC_01HXK...` |
| `CRM_` | Ceremony | `CRM_01HXK...` |
| `VRF_` | Verifier | `VRF_01HXK...` |
| `PRY_` | Property | `PRY_01HXK...` |
| `BTH_` | Batch | `BTH_01HXK...` |
| `CRT_` | Certificate | `CRT_01HXK...` |
| `SIG_` | Signature | `SIG_01HXK...` |
| `TAG_` | NFC Tag | `TAG_01HXK...` |
| `INV_` | Invitation | `INV_01HXK...` |

**ULID factory implementation:**
```rust
pub fn generate_ulid(prefix: &str) -> String {
    format!("{}_{}", prefix, ulid::Ulid::new())
}
```

---

*This master plan is the single source of truth for Sahi platform development. All implementation decisions, architectural patterns, and quality standards flow from this document. Spec files in `specs/` provide additional detail; in case of conflict, defer to the spec hierarchy defined in CLAUDE.md.*
