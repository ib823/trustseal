# /checkpoint — Verify current build state

Usage: `/checkpoint`

When this command is run, perform a full state verification:

1. Run `cat .claude/state/current.md`
2. Run `cat .claude/state/completed.md`
3. Run `git log --oneline -10`
4. Run `git status`
5. Run `cargo test 2>&1 | tail -20` (if Rust code exists)
6. Run `cargo clippy -- -D warnings 2>&1 | tail -20` (if Rust code exists)
7. Run `pnpm test 2>&1 | tail -20` (if web code exists)
8. Check that docker-compose services are healthy: `docker-compose ps`

Then produce a checkpoint report:

```
## Checkpoint Report — [timestamp]

### Build State
- Current phase: [phase]
- Current module: [module]
- Last completed: [module + date]

### Code Quality
- Rust clippy: [PASS / FAIL — N warnings]
- Rust tests: [PASS / FAIL — N passed, N failed]
- TS tests: [PASS / FAIL]

### Infrastructure
- Postgres: [running/stopped]
- Redis: [running/stopped]
- Services: [list]

### Next action
[exactly what should be done next]

### Blockers
[anything blocking progress]
```
