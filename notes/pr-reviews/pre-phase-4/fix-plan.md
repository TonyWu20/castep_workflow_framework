# Fix Plan: `pre-phase-4` v4

**Branch:** `pre-phase-4`
**Date:** 2026-04-18

---

## Summary

Four issues from v4 review: one blocking compilation error (E0283 in hubbard_u_sweep example), three minor (dead_code warnings, duplicated test helper, unconditional disk write).

---

## Parallel Groups

- **Group A (parallel):** TASK-1, TASK-2, TASK-3, TASK-4 — all fully independent.

---

### TASK-1: Add return type annotation to setup closure in hubbard_u_sweep

**File:** `examples/hubbard_u_sweep/src/main.rs`
**Severity:** Blocking

**Context:** The `setup()` method is now generic over `<F, E>` where `E: std::error::Error + Send + Sync + 'static`. The closure uses `?` with mixed error types (`WorkflowError` and I/O errors) so the compiler cannot infer `E`.

**Before:**
```rust
        .setup(move |workdir| {
```

**After:**
```rust
        .setup(move |workdir| -> Result<(), WorkflowError> {
```

**Verification:**
```bash
cargo check -p hubbard_u_sweep
```

**Depends on:** None

---

### TASK-2: Suppress dead_code warnings in test helper module

**File:** `workflow_core/tests/common/mod.rs`
**Severity:** Minor

**Context:** `RecordingExecutor` and `direct()` trigger dead_code warnings because each test binary compiles `common` independently. Note: `#![allow(dead_code)]` is invalid in non-crate-root modules, so use item-level `#[allow(dead_code)]`.

**Edit A — Before:**
```rust
pub struct RecordingExecutor {
```

**After:**
```rust
#[allow(dead_code)]
pub struct RecordingExecutor {
```

**Edit B — Before:**
```rust
pub fn direct(cmd: &str) -> ExecutionMode {
```

**After:**
```rust
#[allow(dead_code)]
pub fn direct(cmd: &str) -> ExecutionMode {
```

**Verification:**
```bash
cargo test -p workflow_core 2>&1 | grep dead_code
```
(Should produce no output.)

**Depends on:** None

---

### TASK-3: Remove duplicated `direct()` helper from hook_recording.rs

**File:** `workflow_core/tests/hook_recording.rs`
**Severity:** Minor

**Context:** Lines 11-13 define a local `direct()` identical to `common::direct()`. Removing it also makes `HashMap` and `ExecutionMode` imports unused.

**Edit 1 — Remove unused imports. Before:**
```rust
use std::collections::HashMap;
use std::sync::Arc;

use workflow_core::{HookExecutor, process::ProcessRunner, state::{JsonStateStore, StateStore, TaskStatus}, task::ExecutionMode, Workflow, Task};
```

**After:**
```rust
use std::sync::Arc;

use workflow_core::{HookExecutor, process::ProcessRunner, state::{JsonStateStore, StateStore, TaskStatus}, Workflow, Task};
```

**Edit 2 — Import `direct` from common. Before:**
```rust
use common::RecordingExecutor;
```

**After:**
```rust
use common::{RecordingExecutor, direct};
```

**Edit 3 — Remove local `direct()`. Before:**
```rust
fn runner() -> Arc<dyn ProcessRunner> { Arc::new(SystemProcessRunner) }
fn direct(cmd: &str) -> ExecutionMode {
    ExecutionMode::Direct { command: cmd.into(), args: vec![], env: HashMap::new(), timeout: None }
}
```

**After:**
```rust
fn runner() -> Arc<dyn ProcessRunner> { Arc::new(SystemProcessRunner) }
```

**Verification:**
```bash
cargo test -p workflow_core
```

**Depends on:** None

---

### TASK-4: Guard `state.save()` in `propagate_skips` with a dirty flag

**File:** `workflow_core/src/workflow.rs`
**Function:** `propagate_skips`
**Severity:** Minor

**Context:** `propagate_skips()` calls `state.save()` every time, even when no tasks were skipped. Since the main loop calls this every 50ms, it causes unnecessary disk writes.

**Edit A — Add `any_skipped` flag. Before:**
```rust
) -> Result<(), WorkflowError> {
    let mut changed = true;
    while changed {
```

**After:**
```rust
) -> Result<(), WorkflowError> {
    let mut any_skipped = false;
    let mut changed = true;
    while changed {
```

**Edit B — Set flag when skipping. Before:**
```rust
        if !to_skip.is_empty() {
            changed = true;
            for id in to_skip.iter() {
                state.mark_skipped_due_to_dep_failure(id);
            }
        }
```

**After:**
```rust
        if !to_skip.is_empty() {
            changed = true;
            any_skipped = true;
            for id in to_skip.iter() {
                state.mark_skipped_due_to_dep_failure(id);
            }
        }
```

**Edit C — Guard save (last `state.save()` in `propagate_skips`). Before:**
```rust
    state.save()?;
    Ok(())
}
```

**After:**
```rust
    if any_skipped {
        state.save()?;
    }
    Ok(())
}
```

**Verification:**
```bash
cargo test -p workflow_core
```

**Depends on:** None

---

## Dependency Graph

```mermaid
graph TD
  TASK-1
  TASK-2
  TASK-3
  TASK-4
```

All four tasks are fully independent — no edges.

## Execution Phases

| Phase | Tasks | Notes |
|-------|-------|-------|
| Phase 1 (parallel) | TASK-1, TASK-2, TASK-3, TASK-4 | All independent |

## Final Verification

```bash
cargo check --workspace --all-targets && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```
