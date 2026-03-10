# SAHI PLATFORM — CLAUDE CODE INSTRUCTIONS
**Version:** 1.0 | **Model:** claude-opus-4-5 | **Updated:** 2026-03-09

You are the primary engineering agent for the **Sahi platform** — a Malaysian trust infrastructure product comprising two products built on a shared cryptographic foundation:

- **VaultPass** — privacy-preserving physical access control (SD-JWT, BLE/NFC, DID)
- **TrustMark** — cryptographic document signing and physical label authentication (PAdES-B-LTA, WebAuthn, COSE)

---

## MANDATORY: READ BEFORE EVERY SESSION

Before writing any code, you MUST:

1. Run `cat .claude/state/current.md` to know exactly where you are in the build
2. Run `cat specs/PHASE_MAP.md` to understand the current phase and module scope
3. Check `git log --oneline -10` to see recent work
4. Never assume — always verify state from these files first

---

## SOURCE OF TRUTH HIERARCHY

When there is any ambiguity, resolve it in this order (highest to lowest authority):

1. `specs/vaultpass-spec.md` — VaultPass complete system specification
2. `specs/trustmark-spec.md` — TrustMark design specification
3. `specs/attack-plan.md` — Module-by-module implementation plan
4. `specs/design-principles.md` — UI/UX locked design system
5. `specs/batch-03-vaultpass-modules.md` — VaultPass module specs
6. `specs/batch-04-trustmark-modules.md` — TrustMark module specs
7. `specs/batch-05-ux-design-system.md` — Full UX/design system spec
8. `specs/batch-06-parts-6-10.md` — Testing, legal, sprint timeline
9. `specs/batch-07-appendices.md` — DDL, error codes, ULID registry

If a requirement appears in the spec but not in the attack plan, the spec wins. If they conflict, flag it as a discrepancy in `.claude/state/discrepancies.md` and implement per the spec.

---

## TECHNOLOGY STACK (NON-NEGOTIABLE)

### Backend (all services)
- **Language:** Rust 1.78+ (2024 edition)
- **Web framework:** Axum 0.8+
- **Database driver:** SQLx 0.8+ (compile-time SQL verification, no ORM)
- **Database:** PostgreSQL 16 with Row-Level Security
- **Cache:** Redis 7
- **Crypto:** ring 0.17+ (TLS/ECDSA), RustCrypto suite (Argon2id, BLAKE2, Ed25519)
- **SD-JWT:** sd-jwt-payload 0.5+
- **COSE:** coset (Google-maintained)
- **BBS+:** bbs crate (Phase 2 only — stub in Phase 1)
- **PDF signing:** Custom PAdES-B-LTA on pdf_signing crate
- **ID generation:** ulid-rs with mandatory prefixes per Appendix G

### Mobile wallet (VaultPass only)
- **Framework:** Flutter 3.19+ / Dart 3.3+
- **FFI bridge:** flutter_rust_bridge 2+
- **BLE:** flutter_blue_plus
- **NFC:** nfc_manager
- **Keystore:** flutter_secure_storage + platform channels (Android Keystore / iOS Secure Enclave)
- **Background scanning:** Kotlin foreground service (Android), Core Bluetooth (iOS)

### Web frontends
- **Framework:** Next.js 14+ / TypeScript 5.3+
- **UI components:** shadcn/ui
- **CSS:** Tailwind CSS (config: `tailwind-v2.config.ts`)
- **i18n:** next-intl with EN/MS JSON files per `specs/microcopy.json`
- **State:** React Query + Zustand

### Infrastructure
- **Monorepo:** pnpm workspaces + Turborepo
- **Build:** Cargo (Rust), pnpm (web), Flutter build
- **Database migrations:** SQLx migrate (files in `migrations/`)
- **Local dev:** docker-compose.yml (Postgres 16, Redis 7, Mosquitto 2)
- **IaC:** Terraform (production only)

### REJECTED — do not use
SurrealDB, SeaORM, Diesel ORM, Actix Web (use Axum), React Native (use Flutter), GraphQL (use REST), any ORM abstraction over SQLx.

---

## REPOSITORY STRUCTURE

```
sahi/
├── CLAUDE.md                    ← you are here
├── Cargo.toml                   ← workspace root
├── pnpm-workspace.yaml
├── turbo.json
├── docker-compose.yml
├── .env.example
├── .claude/
│   ├── state/
│   │   ├── current.md           ← ALWAYS READ FIRST
│   │   ├── completed.md         ← modules marked done
│   │   └── discrepancies.md     ← spec conflicts logged here
│   └── commands/                ← custom slash commands
├── specs/                       ← all specification files (read-only reference)
├── migrations/                  ← SQLx migration files (numbered, ordered)
├── packages/
│   ├── crypto-engine/           ← Rust: SD-JWT, BBS+, keys, WASM
│   ├── credential-format/       ← TS: JSON schemas, validator
│   ├── did-resolver/            ← TS: did:web, did:peer, cache
│   └── status-list/             ← TS: bitstring, Status List VC
├── apps/
│   ├── platform-api/            ← Rust/Axum: main API service
│   ├── issuer-service/          ← Rust/Axum: credential issuance
│   ├── signing-service/         ← Rust/Axum: TrustMark signing
│   ├── verify-service/          ← Rust/Axum: verification
│   ├── admin-portal/            ← Next.js: VaultPass admin
│   ├── guard-tablet/            ← Next.js PWA: guard interface
│   ├── guest-web/               ← Next.js: visitor entry flow
│   ├── trustmark-web/           ← Next.js: TrustMark signing UI
│   ├── verify-web/              ← Next.js: verification landing
│   └── wallet/                  ← Flutter: VaultPass mobile app
└── edge/
    └── verifier/                ← Rust: Raspberry Pi firmware
```

---

## BUILD PHASES AND CURRENT STATE

Always check `.claude/state/current.md` for live state. The canonical phase sequence is:

| Phase | Weeks | Modules | What gets built |
|-------|-------|---------|----------------|
| **Phase 0** | 1-4 | F1-F6 | KMS, Merkle log, API gateway, tenant management, DB/infra, Trust Registry |
| **Phase 1** | 5-20 | VP-1 through VP-9 | Full VaultPass: crypto, DID, wallet, edge verifier, admin, guard, guest web |
| **Phase 2** | 21-26 | Hardening | Pen test, legal, pilot, performance |
| **Phase 3** | 24-38 | TM-1 through TM-7 | Full TrustMark: signing, WebAuthn, COSE, labels, verification |
| **Phase 4** | 39-42 | Hardening | Pen test, red team, claims audit, capacity |

**Phase 0 must be 100% complete before Phase 1 starts.** No exceptions. The shared foundation is a prerequisite for every module.

---

## MODULE COMPLETION STANDARD

A module is only complete when ALL of the following are true:

- [ ] All acceptance criteria in the attack plan are met
- [ ] Unit tests pass with ≥80% coverage on critical paths
- [ ] Integration tests pass (all happy paths + error paths)
- [ ] SQLx compile-time SQL verification passes (`cargo sqlx prepare`)
- [ ] `cargo clippy -- -D warnings` passes with zero warnings
- [ ] `cargo test` passes
- [ ] Performance targets met (see below)
- [ ] Security hardening checklist verified (see `specs/batch-06-parts-6-10.md` Part 6.4)
- [ ] Error codes used match the registry (`specs/batch-07-appendices.md` Appendix E)
- [ ] ULID prefixes match the registry (Appendix G)
- [ ] All user-facing strings come from `specs/microcopy.json` — no hardcoded strings
- [ ] Module entry added to `.claude/state/completed.md`

---

## PERFORMANCE TARGETS (MANDATORY)

These are non-negotiable acceptance criteria:

| Operation | Target | Measurement |
|-----------|--------|-------------|
| BLE credential presentation | < 200ms | Device to verifier handshake complete |
| Gate entry decision | < 2s end-to-end | BLE detect → granted/denied display |
| SD-JWT verification | < 50ms | Crypto verify only, not network |
| TrustMark signing ceremony | < 1s platform overhead | Excludes user passkey confirmation time |
| COSE token generation | < 100ms | Sign + encode |
| Verification landing page | < 1.5s | First meaningful paint |
| API p99 response | < 500ms | Under 100 concurrent |
| Wallet app cold start | < 1.5s | To credential home screen |

If any target is missed, document it in `.claude/state/current.md` with the measured value and a remediation plan before proceeding.

---

## SECURITY NON-NEGOTIABLES

These are hard constraints — never compromise, never work around:

1. **HSM-only signing.** No software key fallback for production signing. Ever. Local dev uses a software mock KMS only.
2. **Fail-closed.** Every gate decision fails to DENY, never to GRANT. On error, deny.
3. **RLS everywhere.** Every PostgreSQL table that holds tenant data has Row-Level Security enabled. Verify in migration files.
4. **No PII in logs.** Resident names, IC numbers, emails never appear in application logs. Use opaque IDs.
5. **No PII in credentials.** SD-JWT selective disclosure — only the minimum necessary claims.
6. **Revocation is immediate.** Status List updates must propagate to edge verifiers within the configured TTL (900s default).
7. **Jailbreak detection.** Warn user, do not refuse operation. Log the warning event.
8. **Input validation.** All API inputs validated with Axum extractors before reaching business logic. Reject malformed inputs with SAHI_4001 error code.
9. **STRIDE model compliance.** Every new component must have its STRIDE threats reviewed against `specs/batch-06-parts-6-10.md` Part 6.3.

---

## CODE QUALITY RULES

### Rust
```toml
# Every Cargo.toml must include:
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

# Every lib.rs or main.rs must have:
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(unused_imports)]
#![deny(dead_code)]
```

- No `.unwrap()` in production code. Use `?` operator and typed errors.
- All errors use the `thiserror` crate with the Sahi error code registry.
- All async code uses Tokio. No `std::thread::sleep` anywhere.
- All SQL is in `.sql` files or inline strings — no string concatenation for queries.

### TypeScript / Next.js
- Strict TypeScript: `"strict": true` in `tsconfig.json`
- No `any` types. If a type is unknown, use `unknown` and narrow it.
- All API calls go through typed fetch functions in `lib/api/`
- All i18n strings from `next-intl` — no hardcoded UI strings

### Flutter / Dart
- All platform channel calls in `services/` directory
- All crypto operations delegated to `flutter_rust_bridge` FFI — no Dart crypto
- All strings from `AppLocalizations` — never hardcoded

---

## UI/UX RULES (locked from sahi-prototype-v2.jsx)

All web frontends and the wallet app must conform to `specs/design-principles.md`. Key rules:

- **Font:** Plus Jakarta Sans primary. JetBrains Mono for IDs/hashes/codes. No other fonts.
- **Colors:** 95% cool slate monochrome. Signal colors (green-500 `#10B981`, red-500 `#EF4444`, amber-500 `#F59E0B`) appear ONLY when carrying semantic meaning.
- **Elevation:** Border-only. No drop shadows except modal overlays.
- **Wallet:** Dark (slate-950) only. No light mode.
- **One primary action per screen.** One filled button. All others outline or ghost.
- **Zero dead ends.** If a hardware capability is absent, hide the feature entirely.
- **No jargon in primary UI.** Technical details always in a collapsible panel.
- Reference prototype: `specs/sahi-prototype-v2.jsx`
- Reference tokens: `specs/tokens-v2.css` and `specs/tailwind-v2.config.ts`
- Flutter theme: `specs/sahi_theme_v2.dart`

---

## MICROCOPY RULES

All user-facing strings come from `specs/microcopy.json`. Structure:

```json
{ "en": { "common": {}, "vaultpass": {}, "trustmark": {} },
  "ms": { "common": {}, "vaultpass": {}, "trustmark": {} } }
```

- Language code is `ms` (ISO 639-1). Never `bm`.
- Both `en` and `ms` keys must be present for every string or CI fails.
- No em dash (—) in any string.
- No security theater language ("unbreakable", "100% secure", etc.)
- Errors always include: what happened + what to do next.

---

## DATABASE RULES

- All migrations in `migrations/` as numbered SQL files: `0001_initial.sql`, `0002_rls.sql`, etc.
- Every migration is reversible (has both UP and DOWN).
- Every table with tenant data has RLS policy.
- All timestamps are `timestamptz` — never `timestamp` without timezone.
- All IDs are ULIDs with registered prefixes from Appendix G.
- Run `cargo sqlx prepare` after every SQL change to regenerate query metadata.

---

## TESTING RULES

| Type | Coverage target | Tool |
|------|----------------|------|
| Unit | ≥80% on critical paths | `cargo test` / Jest |
| Integration | All happy paths + all error paths per module | `cargo test --test integration` |
| E2E | Core flows per phase | Playwright (web), Flutter integration tests |
| Performance | All targets from the table above | k6 (API), custom Rust bench |
| Security | Hardening checklist per module | Manual + `cargo audit` |

Write tests in the same PR as the code. No "tests will be added later."

---

## SESSION WORKFLOW

At the start of every Claude Code session:
```bash
cat .claude/state/current.md    # Where are we?
git status                       # What's uncommitted?
git log --oneline -5             # What was last done?
```

At the end of every significant unit of work:
```bash
# Update state
echo "## $(date) — [what was done]" >> .claude/state/current.md

# Verify quality gates
cargo clippy -- -D warnings
cargo test
cargo sqlx prepare  # if SQL changed
```

At module completion:
```bash
# Mark module complete
echo "- [x] MODULE_ID — $(date)" >> .claude/state/completed.md

# Update current state to next module
# Push all changes
git add -A && git commit -m "feat(MODULE_ID): [description]"
```

---

## CLAIMS POLICY (MUST NOT VIOLATE)

**Allowed in any output (code comments, UI strings, docs):**
- "Cryptographically verifiable"
- "Tamper-evident audit trail"
- "Privacy-preserving"
- "Zero-knowledge gate entry"

**Prohibited — block any code or string that contains:**
- "Impossible to break" / "Unbreakable" / "Cannot be counterfeited"
- "100% secure" / "Guarantees authenticity"
- "We are a Certificate Authority" (platform provides verification infrastructure, not certification)

---

## ERROR CODES

All application errors use the Sahi error code registry from `specs/batch-07-appendices.md` Appendix E. Format: `SAHI_XXXX`.

Never invent new error codes. If a new one is needed, add it to the registry file first, then use it.

---

## ASKING FOR CLARIFICATION

Before implementing any module:
1. Read the full module spec from the attack plan
2. Cross-reference with the source spec (VaultPass or TrustMark)
3. If there is any ambiguity, log it in `.claude/state/discrepancies.md` and implement the more conservative (more secure, more private) interpretation
4. If a decision requires Ikmal's input, stop and ask — do not make an architectural decision unilaterally

---

## AUTHORSHIP POLICY (NON-NEGOTIABLE)

All work in this repository is attributed solely to Ikmal (ib823 / ikmal.baharudin@gmail.com). This rule applies to every AI tool, CLI, or agent that interacts with this repo:

- **NEVER** add `Co-Authored-By` lines referencing any AI (Claude, Anthropic, Copilot, GPT, or any other)
- **NEVER** add "Generated with", "Assisted by", or any AI attribution in commit messages, PR descriptions, code comments, or documentation
- **NEVER** mention AI involvement in any user-visible output committed to the repository
- Commit messages must read as if written entirely by a human developer
- Git hooks in `.githooks/` enforce this automatically — do not bypass them

---

## WHAT NOT TO DO

- Do not refactor working code unless it is blocking the current module
- Do not upgrade pinned dependency versions without explicit instruction
- Do not add dependencies not in the approved stack without asking
- Do not implement Phase 2/3/4 features while in Phase 0/1
- Do not hardcode any configuration values — all config from environment variables
- Do not commit secrets, credentials, or `.env` files
- Do not use `unsafe` in Rust without a documented security review comment
- Do not implement BBS+ (Phase 2) — stub it and move on
