# Pre-Phase-4 Fix Plan v3

**Branch:** `pre-phase-4`
**Review source:** `notes/pr-reviews/pre-phase-4/review.md`
**Date:** 2026-04-18

---

## Summary

Two issues found in v3 review: `TaskClosure` type alias dropped `Send + Sync` bounds (public API regression), and a test closure has broken indentation.

---

## Parallel Groups

- **Group A (parallel):** TASK-1, TASK-2 — fully independent, different files.

---

### TASK-1: Add `Send + Sync` bounds to `TaskClosure` type alias

**File:** `workflow_core/src/task.rs`
**Severity:** Blocking

**Context:** The old `TaskClosure` was `Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>`. During the error-type widening, `+ Send + Sync` was dropped from the outer trait object, leaving only `+ 'static`. The builder methods still enforce `F: Send + Sync`, but the alias itself no longer guarantees it.

**Before** (line 8, the `TaskClosure` type alias):
```rust
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + 'static>;
```

**After:**
```rust
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>;
```

**Verification:**
```bash
cd /Users/tony/programming/castep_workflow_framework && cargo check -p workflow_core && cargo test -p workflow_core
```

**Depends on:** None
**Enables:** None

---

### TASK-2: Fix indentation of `.setup()` closure in `chain_respects_order` test

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor

**Context:** The `.setup()` closure for task "a" in the `chain_respects_order` test has broken indentation (starts at column 0 instead of 12-space indent). The "b" task's `.setup()` is correctly indented.

**Before** (in `chain_respects_order` test, the `.setup()` call for task "a"):
```rust
            )
.setup(move |_| -> Result<(), std::io::Error> {
      let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_for_a)?;
      writeln!(f, "a")?;
      Ok(())
    }),
```

**After:**
```rust
            )
            .setup(move |_| -> Result<(), std::io::Error> {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_for_a)?;
                writeln!(f, "a")?;
                Ok(())
            }),
```

**Verification:**
```bash
cd /Users/tony/programming/castep_workflow_framework && cargo test -p workflow_core chain_respects_order
```

**Depends on:** None
**Enables:** None

---

## Dependency Graph

```mermaid
graph TD
  TASK-1
  TASK-2
```

Both tasks are fully independent — different files.

## Execution Phases

| Phase | Tasks | Notes |
|-------|-------|-------|
| Phase 1 (parallel) | TASK-1, TASK-2 | Fully independent |

## Final Verification

```bash
cd /Users/tony/programming/castep_workflow_framework && cargo check --workspace && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
