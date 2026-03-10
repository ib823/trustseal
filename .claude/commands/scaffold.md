# /scaffold — Create the initial monorepo structure

Usage: `/scaffold` (run only once at the very start)

When this command is run:

1. Verify this is a clean repository (check git status)
2. Create the full directory structure per CLAUDE.md > Repository Structure
3. Create `Cargo.toml` workspace root with all packages
4. Create `pnpm-workspace.yaml`
5. Create `turbo.json` with build pipeline
6. Create `.env.example` from the VaultPass spec Section 6.3
7. Create `docker-compose.yml` from VaultPass spec Section 6.4
8. Create `rust-toolchain.toml` pinning Rust 1.78+
9. Create `.nvmrc` pinning Node.js 20 LTS
10. Create `migrations/` directory with `.gitkeep`
11. Create stub `Cargo.toml` for each Rust package
12. Create stub `package.json` for each TypeScript package
13. Initialize git with an initial commit

After scaffold is complete:
- Update `.claude/state/current.md` to reflect scaffold done
- Print the full directory tree
- Confirm Phase 0 is ready to begin

Do not create any application logic during scaffold — only structure and config files.
