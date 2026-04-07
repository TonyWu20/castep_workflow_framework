# Fix Plan: qwopus3.5 → main

## Issue 1: Rename `_monitors` and `_task_workdirs` in `run()`

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor

**Before:**

```rust
let _monitors: HashMap<String, Vec<workflow_utils::MonitoringHook>> = ...
let _task_workdirs: HashMap<String, std::path::PathBuf> = ...
```

**After:** Rename both declarations and all usages:

- `_monitors` → `monitors`
- `_task_workdirs` → `task_workdirs`

**Verification:** `cargo build -p workflow_core` with no unused-variable warnings.

---

## Issue 2: Remove misleading comment in `resume()`

**File:** `workflow_core/src/workflow.rs`

**Severity:** Minor

**Before:**

```rust
pub fn resume(name: impl Into<String>, state_dir: impl Into<PathBuf>) -> Result<Self> {
    // Just create a new workflow with the given name
    // State will be loaded from file in run() if available
    Self::builder().name(name.into()).state_dir(state_dir.into()).build()
}
```

**After:**

```rust
/// Resume a workflow by name, loading prior state from `state_dir` when `run()` is called.
pub fn resume(name: impl Into<String>, state_dir: impl Into<PathBuf>) -> Result<Self> {
    Self::builder().name(name.into()).state_dir(state_dir.into()).build()
}
```

**Verification:** `cargo test -p workflow_core -- resume_loads_existing_state` passes.

---

## Issue 3: Restore convenience re-exports in `lib.rs`

**File:** `workflow_core/src/lib.rs`

**Severity:** Major

**Before:**

```rust
pub mod dag;
pub mod state;
pub mod task;
pub mod workflow;
```

**After:**

```rust
pub mod dag;
pub mod state;
pub mod task;
pub mod workflow;

pub use task::Task;
pub use workflow::Workflow;
pub use state::{TaskStatus, WorkflowState};
```

**Verification:** `cargo build -p workflow_core` passes.

---

## Issue 4: Move `bon` and `serde_json` to workspace dependencies

**Files:** root `Cargo.toml`, `workflow_core/Cargo.toml`

**Severity:** Minor

**Step 1** — Add to root `Cargo.toml` `[workspace.dependencies]`:

```toml
bon = "3.9.1"
serde_json = "1"
```

**Step 2** — In `workflow_core/Cargo.toml` replace:

```toml
bon = "3.9.1"
serde_json = "1"
```

with:

```toml
bon = { workspace = true }
serde_json = { workspace = true }
```

**Verification:** `cargo build -p workflow_core` passes.

---

## Issue 5: Add `workdir()` builder method to `Task`

**File:** `workflow_core/src/task.rs`

**Severity:** Major

**Before:** The `add_monitor` method is the last builder method in `task.rs`:

```rust
    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self {
        self.monitors.push(hook);
        self
    }
}
```

No `workdir` setter exists after it.

**After:** Insert the new method between `add_monitor` and the closing `}` of `impl Task`:

```rust
    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self {
        self.monitors.push(hook);
        self
    }

    pub fn workdir(mut self, path: impl Into<PathBuf>) -> Self {
        self.workdir = path.into();
        self
    }
}
```

**Verification:** `cargo test -p workflow_core` passes. The following compiles without error:

```rust
Task::new("id", || Ok(())).workdir("/tmp/my_workdir")
```

---

## Recurring Issues (flagged in previous review — not yet fixed)

These two issues appeared in the prior review of this branch and were not addressed. They must not recur in future PRs.

### R1: Underscore-prefixed names on live variables

**Pattern to avoid:** Naming an actively-used variable `_foo`. In Rust, `_foo` means "intentionally unused — suppress the dead-code warning." Applying it to variables that are read throughout the function is misleading to readers and linters.

**Rule:** Only use the `_` prefix on variables that are genuinely unused (e.g., a binding kept alive for its `Drop` side-effect, or a placeholder in a destructure). If a variable is read anywhere, name it without the prefix.

**How to catch it before PR:** Run `cargo clippy -p workflow_core` — clippy will warn if a `_`-prefixed variable is actually used.

### R2: Removing `pub use` re-exports from `lib.rs` without replacement

**Pattern to avoid:** Deleting top-level re-exports (`pub use task::Task` etc.) during a refactor without either restoring them or explicitly documenting the breaking API change.

**Rule:** `lib.rs` re-exports are the public API contract. Any removal is a breaking change. Before removing a `pub use`, check whether any downstream crate (including future `castep_adapter`) uses the short path. If yes, keep the re-export. If the removal is intentional, document it in the PR description and update all call sites in the same commit.

**How to catch it before PR:** After any `lib.rs` edit, run `cargo build --workspace` to surface broken downstream imports immediately.
