# /quality — Run all quality gates for current module

Usage: `/quality` or `/quality [module-id]`

Run the full quality gate sequence for the current module (or specified module):

```bash
# Rust quality gates
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo sqlx prepare --check   # verify SQL is up to date
cargo audit                   # check for security advisories

# TypeScript quality gates (if applicable)
pnpm typecheck
pnpm lint
pnpm test

# Flutter quality gates (if applicable)
flutter analyze
flutter test

# Integration check
docker-compose up -d
cargo test --test integration
docker-compose down
```

Report results:
```
## Quality Gate Report — [module] — [timestamp]

| Gate | Status | Details |
|------|--------|---------|
| Rust fmt | PASS/FAIL | |
| Rust clippy | PASS/FAIL | N warnings |
| Rust tests | PASS/FAIL | N passed, N failed |
| SQL offline | PASS/FAIL | |
| Cargo audit | PASS/FAIL | N advisories |
| TS typecheck | PASS/FAIL | |
| TS lint | PASS/FAIL | |
| TS tests | PASS/FAIL | |
| Flutter analyze | PASS/FAIL | |
| Integration | PASS/FAIL | |

### Overall: [PASS / FAIL]
[If FAIL: list what must be fixed before module can be marked complete]
```

Only mark a module complete in `.claude/state/completed.md` when all gates are PASS.
