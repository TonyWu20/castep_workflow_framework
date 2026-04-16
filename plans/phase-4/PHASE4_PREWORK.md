# Phase 4 Prework: Code Quality & Test Coverage

**Goal:** Address the issues identified in the Phase 3 post-mortem before implementing Phase 4 features. These are correctness, ergonomics, and maintainability fixes that will make Phase 4 implementation cleaner.

---

## Pre-Implementation Gates

These must be completed and verified **before any feature code is written**:

### Gate 1: Dependency audit

Audit every `Cargo.toml` in the workspace:
- All lib crates must use `thiserror` and `time` via `{ workspace = true }`
- `anyhow` must appear only in binary crates
- No unused or misplaced deps

```
cargo tree --workspace | grep anyhow
```

Flag and fix any violations before proceeding.

### Gate 2: Multi-instance integration test

Write a test that creates a `Workflow`, runs it, then creates a **second** `Workflow` and runs it again — verifying signal registration works correctly for both instances. This must pass before any changes to `workflow.rs`.

### Gate 3: `StateStore` use-case enumeration

Before touching the `StateStore` trait, explicitly list all use cases it must serve in Phase 4 (read-only inspection, crash recovery, migration, export, etc.). Document this as a comment block above the trait definition. Avoids the "add `load_raw()` after the fact" pattern.

---

## Scope

1. Fix `TaskClosure` error type — remove forced `WorkflowError` from user closures
2. Add timeout test — the only unverified production code path
3. Add setup/collect failure and hook firing tests
4. Extract and clean up `Workflow::run`

---

## Item 1: Fix `TaskClosure` error type

**File:** `workflow_core/src/task.rs`

Current:
```rust
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>;
```

Problem: forces `.map_err(WorkflowError::Io)` at every call site. The run loop discards the concrete error via `.to_string()` anyway.

Fix:
```rust
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;
```

Builder methods (`setup`, `collect`) become generic over `E: Error + Send + Sync + 'static` and box the error internally:
```rust
pub fn setup<F, E>(mut self, f: F) -> Self
where
    F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    self.setup = Some(Box::new(move |p| f(p).map_err(|e| Box::new(e) as _)));
    self
}
```

Same pattern for `collect`. No changes needed in `workflow.rs` — it already calls `.to_string()` on the error.

---

## Item 2: Add timeout test

**File:** `workflow_core/src/workflow.rs` (tests module)

Add a test that:
- Creates a task with `command: "sleep"`, `args: ["10"]`, `timeout: Some(Duration::from_millis(100))`
- Runs the workflow
- Asserts the task is `Failed` (not `Completed`)
- Asserts the summary has 1 failed task

This covers the `task_timeouts` map path which currently has zero test coverage.

---

## Item 3: Add setup/collect failure and hook firing tests

**File:** `workflow_core/src/workflow.rs` (tests module)

Three new tests:

**setup_failure_marks_task_failed**: setup closure returns `Err`, task should be `Failed`, dependent tasks `SkippedDueToDependencyFailure`.

**collect_failure_is_warned_not_fatal**: collect closure returns `Err`, task should still be `Completed` (collect failure is non-fatal by design).

**hooks_fire_at_correct_lifecycle**: Use a recording `HookExecutor` (stores fired hook names + triggers in a `Arc<Mutex<Vec<...>>>`). Assert `OnComplete` fires on success, `OnFailure` fires on failure, `OnStart` fires on dispatch.

---

## Item 4: Extract and clean up `Workflow::run`

**File:** `workflow_core/src/workflow.rs`

`Workflow::run` is 318 lines with 6 concerns in one loop body. Changes:

**4a. Merge the two upfront validation loops** (lines 90–109) into a single pass over `&self.tasks`.

**4b. Remove the dead `Queued` `unreachable!()` arm** in the dispatch block — `Queued` is already rejected upfront; the arm is dead weight.

**4c. Unify duplicated hook-firing** — OnStart (dispatch block) and OnComplete/OnFailure (finished-task block) share identical iteration + match + warn logic. Extract to:
```rust
fn fire_hooks(&self, trigger: HookTrigger, task_id: &str, hook_executor: &dyn HookExecutor)
```

**4d. Extract loop-body concerns into private methods:**

- `fn poll_finished(handles: &mut HashMap<String, InFlightTask>, state: &mut dyn StateStore, task_timeouts: &HashMap<String, Duration>) -> Vec<String>` — returns IDs of finished/timed-out tasks
- `fn process_finished(id: &str, t: InFlightTask, state: &mut dyn StateStore, hook_executor: &dyn HookExecutor)` — handles wait(), mark_completed/failed, hook firing
- `fn propagate_skips(tasks: &HashMap<String, Task>, dag: &Dag, state: &mut dyn StateStore)` — the skip propagation while loop
- `fn build_summary(state: &dyn StateStore) -> WorkflowSummary` — constructs the final summary

The main loop body becomes a readable orchestration of these calls.

**4e. Add a test fixture builder** to eliminate the ~15× repeated `ExecutionMode::Direct { command, args: vec![], env: HashMap::new(), timeout: None }` boilerplate in tests:
```rust
fn direct(command: &str) -> ExecutionMode { ... }
```

---

## Sequencing

```
Item 1 (TaskClosure fix)   — do first; affects builder API used in all tests
Item 2 (timeout test)      — isolated
Item 3 (hook/setup tests)  — isolated; write recording executor once, reuse
Item 4 (extract run())     — last; pure refactor, no behavior change
```

---

## Verification

After each item:
```
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

All existing tests must continue to pass. No new public API surface is added.
