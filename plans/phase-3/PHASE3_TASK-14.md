# TASK-14: Remove stale workspace dependencies

- **Scope**: Remove `workflow_cli` from workspace members and prune dead deps from `[workspace.dependencies]`.
- **Crate/Module**: `Cargo.toml` (workspace root)
- **Depends On**: None
- **Enables**: TASK-18
- **Can Run In Parallel With**: TASK-12, TASK-13, TASK-15

### Acceptance Criteria

- `workflow_cli` removed from `[workspace.members]` FIRST (before any dep removal).
- `tokio`, `async-trait`, `tokio-rusqlite`, `tokio-util`, `toml` removed from `[workspace.dependencies]`.
- `bon` removed only after `rg '#\[bon\]|bon::' --type rust` returns zero matches.
- `nix` removed only if present.
- `serde`, `serde_json`, `petgraph`, `anyhow`, `tracing`, `tracing-subscriber` remain.
- `cargo check --workspace` and `cargo test --workspace` pass.

### Implementation

Steps in order:

1. Remove `"workflow_cli"` from `[workspace.members]`
2. Run `rg '#\[bon\]|bon::' --type rust` — if zero matches, remove `bon`
3. Remove from `[workspace.dependencies]`: `tokio`, `async-trait`, `tokio-rusqlite`, `tokio-util`, `toml`, `nix` (if present)

**Verify**: `cargo check --workspace && cargo test --workspace`
