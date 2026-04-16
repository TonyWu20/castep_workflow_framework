# Phase 3 Fix Plan — v8 (2026-04-16)

This plan addresses 1 issue found in the v8 review: `workflow_core/Cargo.toml` still declares `thiserror` and `time` as direct pinned versions instead of consuming the workspace entries added in v7. Single task, no dependencies.

---

### TASK-1: Replace pinned `thiserror` and `time` with workspace references in `workflow_core`

**File:** `workflow_core/Cargo.toml`

**Before:**
```toml
thiserror = "1"
time = { version = "0.3", features = ["formatting"] }
```

**After:**
```toml
thiserror = { workspace = true }
time = { workspace = true }
```

**Note:** The `features = ["formatting"]` annotation is dropped because the root `Cargo.toml` workspace entry already declares `time = { version = "0.3", features = ["formatting"] }` — the feature is inherited automatically.

**Verification:**
```
cargo check -p workflow_core
```

**Dependencies:** None. Self-contained.
