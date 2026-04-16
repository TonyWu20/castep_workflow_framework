# Phase 3 Fix Plan — v7 (2026-04-16)

This plan addresses 2 issues found in the v7 review: a stale `anyhow` dependency in `workflow_utils` (violates "anyhow only in binaries" rule), and two crates (`thiserror`, `time`) that bypass workspace dependency management in `workflow_core`. Both issues are independent and can run fully in parallel.

---

## Execution Phases

| Phase              | Tasks        | Notes                                                                 |
| ------------------ | ------------ | --------------------------------------------------------------------- |
| Phase 1 (parallel) | TASK-1, TASK-2 | Independent — different files, no overlapping edits                   |
| Final              | —            | `cargo check --workspace && cargo clippy --workspace`                |

---

### TASK-1: Remove stale `anyhow` dependency from `workflow_utils`

**File:** `workflow_utils/Cargo.toml`
**Severity:** Minor
**Depends on:** None
**Enables:** None
**Can run in parallel with:** TASK-2
**Problem:** `anyhow = { workspace = true }` is declared in `[dependencies]` but is not used anywhere in `workflow_utils/src/`. The design principle is `anyhow` only in binary crates.
**Fix:** Remove the `anyhow` line from `[dependencies]`.

Before (inside `[dependencies]`, the full block):
```toml
[dependencies]
anyhow = { workspace = true }
serde = { workspace = true }
workflow_core = { path = "../workflow_core" }
```

After:
```toml
[dependencies]
serde = { workspace = true }
workflow_core = { path = "../workflow_core" }
```

**Verification:** `cargo check --workspace && cargo clippy --workspace`

---

### TASK-2: Promote `thiserror` and `time` to workspace-managed dependencies

**File:** `Cargo.toml` (workspace root), `workflow_core/Cargo.toml`
**Severity:** Minor
**Depends on:** None
**Enables:** None
**Can run in parallel with:** TASK-1
**Problem:** `thiserror = "1"` and `time = { version = "0.3", features = ["formatting"] }` are direct deps in `workflow_core/Cargo.toml` instead of workspace-managed. All other external crates use `workspace = true`.

**IMPORTANT:** Both sub-steps below MUST be applied together in a single atomic operation. Applying only one will break the build.

**Step 1 — Add to workspace root `Cargo.toml`.**

Find the `[workspace.dependencies]` section. Locate the last entry `signal-hook = "0.3"`.

Before (last entry in `[workspace.dependencies]`):
```toml
signal-hook = "0.3"
```

After (append two new lines immediately after `signal-hook`):
```toml
signal-hook = "0.3"
thiserror = "1"
time = { version = "0.3", features = ["formatting"] }
```

**Step 2 — Update `workflow_core/Cargo.toml`.**

Find the `[dependencies]` section. Locate the two direct version declarations.

Before (inside `[dependencies]`):
```toml
thiserror = "1"
time = { version = "0.3", features = ["formatting"] }
```

After:
```toml
thiserror = { workspace = true }
time = { workspace = true }
```

**Verification:** `cargo check --workspace`
