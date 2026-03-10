# Sahi Platform — Codespace Setup Guide

## What this package contains

```
sahi-claude-code-setup/
├── CLAUDE.md                        ← Primary Claude Code instructions (upload to repo root)
├── README.md                        ← This file (do not upload to repo)
├── .claude/
│   ├── settings.json                ← Claude Code model + permissions config
│   ├── commands/
│   │   ├── module.md                ← /module slash command
│   │   ├── checkpoint.md            ← /checkpoint slash command
│   │   ├── scaffold.md              ← /scaffold slash command
│   │   ├── quality.md               ← /quality slash command
│   │   └── spec.md                  ← /spec slash command
│   └── state/
│       ├── current.md               ← Live build state (update every session)
│       ├── completed.md             ← Module completion log
│       └── discrepancies.md         ← Spec conflict log
├── .devcontainer/
│   ├── devcontainer.json            ← Codespace configuration
│   └── setup.sh                     ← One-time environment setup script
└── specs/
    └── PHASE_MAP.md                 ← Phase/module quick reference
```

---

## Step 1 — Create the GitHub repository

```bash
# Create a new private repo on GitHub named "sahi"
# Clone it locally or open in Codespace directly
```

---

## Step 2 — Upload this setup package to the repo root

Upload these files/folders maintaining the exact directory structure:
- `CLAUDE.md` → repo root
- `.claude/` → repo root (the entire directory)
- `.devcontainer/` → repo root (the entire directory)
- `specs/PHASE_MAP.md` → `specs/PHASE_MAP.md`

---

## Step 3 — Upload all Sahi spec files into specs/

Upload every file from your Claude project knowledge into `specs/`:

| Upload as | Rename to |
|-----------|-----------|
| `VaultPass_Complete_System_Specification_v1.md` | `specs/vaultpass-spec.md` |
| `TrustMark_DSD_v3_Consolidated.docx` | `specs/trustmark-spec.docx` (or convert to .md first — recommended) |
| `sahi-attack-plan-v2.md` | `specs/attack-plan.md` |
| `batch-01-strategic-overview.md` | `specs/batch-01-strategic-overview.md` |
| `batch-03-vaultpass-modules-v2.md` | `specs/batch-03-vaultpass-modules.md` |
| `batch-04-trustmark-modules-v2.md` | `specs/batch-04-trustmark-modules.md` |
| `batch-05-ux-design-system-v2.md` | `specs/batch-05-ux-design-system.md` |
| `batch-06-parts-6-10.md` | `specs/batch-06-parts-6-10.md` |
| `batch-07-appendices.md` | `specs/batch-07-appendices.md` |
| `sahi-design-principles.md` | `specs/design-principles.md` |
| `sahi-prototype-v2.jsx` | `specs/sahi-prototype-v2.jsx` |
| `tokens-v2.css` | `specs/tokens-v2.css` |
| `tailwind-v2_config.ts` | `specs/tailwind-v2.config.ts` |
| `sahi_theme_v2.dart` | `specs/sahi_theme_v2.dart` |
| `sahi-microcopy2.json` | `specs/microcopy.json` |
| `sahi-ui-masterplan.md` | `specs/ui-masterplan.md` |

> **Note on TrustMark spec:** The `.docx` file is harder for Claude Code to read inline. Recommended: convert it to Markdown using `pandoc TrustMark_DSD_v3_Consolidated.docx -o specs/trustmark-spec.md` before uploading. If you can't convert, upload the `.docx` and Claude Code will read it when needed.

---

## Step 4 — Create the Codespace

1. Go to the GitHub repo
2. Click **Code → Codespaces → Create codespace on main**
3. Choose machine type: **8-core, 32GB RAM** (minimum for Rust + Flutter + Postgres)
4. Wait for the devcontainer to build (~5-10 minutes on first run)

The `setup.sh` will automatically:
- Install Rust components (clippy, rustfmt, wasm32 target)
- Install cargo tools (cargo-audit, sqlx-cli, wasm-pack, cargo-watch)
- Install pnpm and turbo globally
- Install system dependencies (libssl, libpq, etc.)
- Copy `.env.example` to `.env`
- Install git pre-commit hooks

---

## Step 5 — Start Claude Code CLI

```bash
# In the Codespace terminal:
claude

# Claude Code will read CLAUDE.md automatically on startup.
# First session commands:
/checkpoint          # verify environment state
/scaffold            # create monorepo structure (first time only)
/module F1           # begin Phase 0, Module F1
```

---

## Step 6 — Configure environment variables

Edit `.env` and fill in:
- `HSM_*` credentials (use AWS CloudHSM for production, software mock for dev)
- `MYDIGITAL_ID_*` credentials (from MyDigital ID developer portal)
- Any other secrets marked `<placeholder>` in `.env.example`

**Never commit `.env` to the repository.** It is in `.gitignore`.

---

## Recommended session pattern

Every Claude Code session:
```
/checkpoint                    # always start here
/module [next module]          # focus on one module at a time
[work...]
/quality                       # run gates before ending
git add -A && git commit -m "feat(MODULE): description"
```

---

## Codespace machine size recommendation

| Phase | Recommended size | Why |
|-------|-----------------|-----|
| Phase 0 (backend only) | 4-core, 16GB | Rust compilation is memory-hungry |
| Phase 1 (wallet + edge) | 8-core, 32GB | Flutter + Rust simultaneously |
| Phase 3 (TrustMark) | 8-core, 32GB | PDF engine + COSE + signing tests |

---

## Troubleshooting

**Rust compilation OOM:**
```bash
export CARGO_BUILD_JOBS=2   # reduce parallel jobs
```

**SQLx offline mode errors:**
```bash
cargo sqlx prepare          # regenerate query metadata
# or set SQLX_OFFLINE=true in .env for faster iteration
```

**Postgres connection refused:**
```bash
docker-compose up -d postgres
sleep 3
cargo sqlx migrate run
```

**Flutter FFI bridge not generating:**
```bash
cd packages/crypto-engine
cargo build --release
cd ../../apps/wallet
flutter_rust_bridge_codegen generate
```
