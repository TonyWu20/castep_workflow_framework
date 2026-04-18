# Phase 4 Prework Implementation Plan

## Context

Phase 3 required 9 fix rounds before merge. Post-mortem identified three recurring themes: dependency hygiene (anyhow/thiserror/time pinning), `workflow.rs` instability from incremental growth, and state loading semantics conflating mutation with inspection. This prework phase resolves all three before Phase 4 features land, using pre-implementation gates as forcing functions.

---

## Pre-Implementation Gates

Must all pass before any Item 1–4 work proceeds.

### Gate 1: Dependency audit
- `workflow_core/Cargo.toml`: `thiserror` and `time` must become `{ workspace = true }`
- `workflow_utils/Cargo.toml`: confirm no `anyhow` (lib crate violation)
- `workflow-cli/`, `examples/hubbard_u_sweep/`: `anyhow` permitted (binary crates)
- Verify: `cargo tree -p workflow_core` and `cargo tree -p workflow_utils` show no `anyhow` node

### Gate 2: Multi-instance signal test
- Write test: run Workflow 1, set its `interrupt` flag to `true` **before** calling `run()`, run Workflow 2, assert it completes normally
- Expose `pub fn interrupt_handle(&self) -> Arc<AtomicBool>` accessor — do not promote the field itself to `pub`
- Verify `interrupt` field visibility with LSP hover before writing

### Gate 3: StateStore use-case comment
- Add comment block above `pub trait StateStore` in `workflow_core/src/state.rs`
- List: task status persistence, crash recovery/resume, read-only inspection, workflow summary export
- Clarify that `load`/`load_raw` are on `JsonStateStore`, not this trait
- No trait methods changed

---

## Task Breakdown

### TASK-1: Fix workflow_core workspace dep pins
**File:** `workflow_core/Cargo.toml`
**Depends on:** none
**Parallel with:** TASK-2, TASK-3, TASK-4

Change:
- `thiserror = "1"` → `thiserror = { workspace = true }`
- `time = { version = "0.3", ... }` → `time = { workspace = true }` (no `features` key — workspace declaration already carries it)

Acceptance: `cargo check -p workflow_core` passes; `cargo tree -p workflow_core | grep thiserror` shows single version.

---

### TASK-2: Audit remaining crates for anyhow placement
**Files:** `workflow_utils/Cargo.toml`, `workflow-cli/Cargo.toml`, `examples/hubbard_u_sweep/Cargo.toml`
**Depends on:** none
**Parallel with:** TASK-1, TASK-3, TASK-4

Fix any `anyhow` in lib crates. Confirm `workflow-cli` and `examples/hubbard_u_sweep` still have `anyhow` (positive confirmation that binary crates are correct).

Acceptance: `cargo tree -p workflow_core` and `cargo tree -p workflow_utils` show no `anyhow` node; binary crates retain `anyhow`.

---

### TASK-3: Multi-instance signal integration test
**Files:** `workflow_core/src/workflow.rs` (accessor), `workflow_core/tests/signal_isolation.rs` (new)
**Depends on:** none
**Parallel with:** TASK-1, TASK-2, TASK-4

Add `pub fn interrupt_handle(&self) -> Arc<AtomicBool>` to `Workflow` returning `Arc::clone(&self.interrupt)`. Doc comment must state it is the intended signal-injection point for testing, not a general-purpose pause mechanism.

Test `interrupt_flag_is_per_instance`:
1. Construct Workflow 1, call `interrupt_handle()`, set flag to `true` **before** calling `run()`
2. Call `run()` on Workflow 1 — assert `Err(WorkflowError::Interrupted)` without executing any tasks
3. Construct Workflow 2 with a fresh `Workflow::new(...)`, run to completion, assert `Ok`
4. Include a comment explaining why sending a real `SIGINT` in a test process would affect all previously-registered handlers

Note: Integration test lives in `workflow_core/tests/` and cannot access `pub(crate)` fields — must use the new accessor. Existing unit test `interrupt_before_run_dispatches_nothing` is unchanged.

Acceptance: test passes; LSP diagnostics on `workflow.rs` show no new errors.

---

### TASK-4: Enumerate StateStore use cases
**File:** `workflow_core/src/state.rs`
**Depends on:** none
**Parallel with:** TASK-1, TASK-2, TASK-3

Use `LSP hover` on `StateStore` to confirm the current doc comment before editing. Replace the one-liner above `pub trait StateStore` with a multi-paragraph comment covering:
- Task status persistence during a live run (the trait's runtime mutation role)
- Crash recovery / resume (`JsonStateStore::load` resets Running/Failed/Skipped to Pending)
- Read-only inspection for CLI status display (`JsonStateStore::load_raw`)
- Workflow summary export

The comment must explicitly state that `load` and `load_raw` are methods on `JsonStateStore`, not on this trait, and direct readers there for persistence semantics. No trait or impl changes.

Acceptance: `cargo doc -p workflow_core` renders without warnings on `state.rs`.

---

### TASK-5: Gate verification checkpoint
**Depends on:** TASK-1, TASK-2, TASK-3, TASK-4

Run: `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`. All must pass. Block if any gate fails.

Gate checklist:
- [ ] Gate 1: `cargo tree -p workflow_core` and `cargo tree -p workflow_utils` show no `anyhow`; `thiserror`/`time` use `{ workspace = true }` in `workflow_core`
- [ ] Gate 2: Multi-instance signal test passes
- [ ] Gate 3: `StateStore` use-case comment block present

---

### TASK-6: Widen TaskClosure error type
**Files:** `workflow_core/src/task.rs`, `workflow_core/src/workflow.rs`, `workflow_core/tests/integration.rs`, `workflow_core/tests/hubbard_u_sweep.rs`
**Depends on:** TASK-5
**Parallel with:** TASK-7, TASK-10

**Status note:** `TaskClosure` alias and builder generics are already implemented. The remaining work is updating call sites to use concrete error types.

Before editing: use `LSP references` on `TaskClosure` to find all usages. Confirm which call sites construct a `TaskClosure` directly (bypassing the builder methods).

**Already done:**
- `TaskClosure` alias uses `Box<dyn std::error::Error + Send + Sync>` as the error type.
- `setup` and `collect` builder methods are generic over `F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static` where `E: std::error::Error + Send + Sync + 'static`. Both builders wrap the user closure in an outer closure that maps the error via boxing.

**Remaining work — update call sites to remove `Box<dyn ...>` annotations:**

Root cause: `()` does not implement `std::error::Error`, so infallible closures cannot infer `E`, forcing a turbofish. Fix by using concrete error types or `std::convert::Infallible`.

`workflow_core/tests/integration.rs` — infallible closures:
```rust
// before
.setup(move |_| { ...; Ok::<_, Box<dyn std::error::Error + Send + Sync>>(()) })
// after
.setup(move |_| -> Result<(), std::convert::Infallible> { ...; Ok(()) })
```

`workflow_core/tests/hubbard_u_sweep.rs` — closures returning `WorkflowError`:
```rust
// before
.setup(move |_| -> Result<(), Box<dyn std::error::Error + Send + Sync>> { ...; Ok::<_, Box<dyn std::error::Error + Send + Sync>>(()) })
// after
.setup(move |_| -> Result<(), WorkflowError> { ...; Ok(()) })
```

`workflow_core/src/workflow.rs` — closures with `std::io::Error` paths: use `-> Result<(), std::io::Error>` and drop manual `Box::from` wrapping; `?` works directly.

`workflow_core/src/workflow.rs` — infallible closures: use `-> Result<(), std::convert::Infallible>`.

Acceptance: `cargo test -p workflow_core` passes; no `Box<dyn std::error::Error + Send + Sync>` annotations remain in closure return types; LSP diagnostics on `task.rs`, `workflow.rs`, and test files show no errors.

---

### TASK-7: Add timeout test
**File:** `workflow_core/tests/timeout_integration.rs`
**Depends on:** TASK-5
**Parallel with:** TASK-6, TASK-8, TASK-10

Add test `task_timeout_marks_failed`:
- Task: `command: "sleep"`, `args: ["10"]`, `timeout: Some(Duration::from_millis(100))`
- Assert: task status is `TaskStatus::Failed { error }` where `error` matches `WorkflowError::TaskTimeout(id.to_string()).to_string()`
- Must complete in under 1 second (confirms terminate-then-skip-wait path)
- Use `workflow_utils::{ShellHookExecutor, SystemProcessRunner}` consistent with existing tests in that file

Acceptance: `cargo test -p workflow_core task_timeout_marks_failed` passes in under 1 second.

---

### TASK-8: Add RecordingExecutor and setup/collect failure tests
**Files:** `workflow_core/tests/common/mod.rs` (new), `workflow_core/tests/hook_recording.rs` (new)
**Depends on:** TASK-5, TASK-6
**Parallel with:** TASK-11

Add `RecordingExecutor` in `tests/common/mod.rs`:
- Implements `HookExecutor`
- Stores calls as `Arc<Mutex<Vec<(String, String)>>>` (hook name, task id)
- Exposes `calls(&self) -> Vec<(String, String)>` method
- Share the `Arc` outside the executor so the test body can read recorded calls

Three tests in `hook_recording.rs`:

`setup_failure_skips_dependent`: task "a" setup closure returns error → `TaskStatus::Failed`; task "b" (depends on "a") → `SkippedDueToDependencyFailure`.

`collect_failure_does_not_fail_task`: task "a" collect closure returns error → `TaskStatus::Completed` (not Failed; `workflow.rs` uses `tracing::warn!` and does not mark the task failed).

`hooks_fire_on_start_complete_failure`: use `RecordingExecutor`; assert `calls()` contains the expected `(hook_name, task_id)` pairs for OnStart+OnComplete on success and OnStart+OnFailure on failure. The `calls()` assertion is required — `RecordingExecutor` must not be dead infrastructure.

Note: After TASK-6, `.setup(|_| Err("setup failed".into()))` works without `map_err`.

Acceptance: all three tests pass; LSP diagnostics on new files show no errors.

---

### TASK-9: Add `direct()` test fixture builder
**Files:** `workflow_core/tests/common/mod.rs`, `workflow_core/tests/integration.rs`, `workflow_core/tests/timeout_integration.rs`, `workflow_core/tests/resume.rs`
**Depends on:** TASK-8

Use `LSP references` on `ExecutionMode::Direct` within test files to find all occurrences before editing.

Add `pub fn direct(cmd: &str) -> ExecutionMode` inside `tests/common/mod.rs`. Remove the local `fn direct` from `integration.rs`. Replace all inline `ExecutionMode::Direct { command: ..., args: vec![], env: HashMap::new(), timeout: None }` constructions in test files with `common::direct(...)`.

Each test file needs `mod common;` at its root (Rust integration test files each have their own module root).

Acceptance: `cargo test -p workflow_core` passes; no production code changed.

---

### TASK-10: Merge validation loops + remove dead Queued arm
**File:** `workflow_core/src/workflow.rs` (production)
**Depends on:** TASK-5
**Parallel with:** TASK-6, TASK-7

Use `LSP documentSymbol` to locate the `run` function body before editing.

Merge the two upfront validation loops (Queued-rejection and Periodic-hook-rejection) into a single pass over `&self.tasks`. The merged loop must appear before `build_dag()`. Delete `Queued => unreachable!(...)` arm from the dispatch `match` — after the upfront check, this arm is genuinely dead, and removing it means a new `ExecutionMode` variant will produce a compile error rather than a silent `unreachable!` at runtime.

Run `LSP diagnostics` after edit before proceeding.

Acceptance: `cargo test -p workflow_core` passes; zero LSP errors.

---

### TASK-11: Extract fire_hooks as free function
**File:** `workflow_core/src/workflow.rs` (production)
**Depends on:** TASK-10
**Parallel with:** TASK-8

Use `LSP documentSymbol` to locate the hook-firing block within `run()` before extracting.

Extract duplicated hook-firing logic as a free function — NOT a `&self` method (borrow conflict: `t.monitors`/`t.workdir` from the consumed `InFlightTask` vs `state: &mut dyn StateStore`). The function takes: `monitors: &[MonitoringHook]`, `workdir: &Path`, `final_state: &str`, `exit_code: Option<i32>`, `task_id: &str`, `hook_executor: &dyn HookExecutor`, `state: &mut dyn StateStore`. Returns `Result<(), WorkflowError>`.

`tracing::warn!` calls for hook failures remain inside the function. `state.save()` after hook firing remains at the `run()` call site — the function returns `Result` so the caller propagates the save error.

Replace both call sites (dispatch block OnStart, finished-task block OnComplete/OnFailure).

Do NOT fix the stringly-typed `final_state: &str` pattern — extract existing logic faithfully.

Acceptance: TASK-8 hook tests still pass; `cargo test -p workflow_core` passes; LSP diagnostics show no new errors.

---

### TASK-12: Extract poll_finished, process_finished, propagate_skips, build_summary
**File:** `workflow_core/src/workflow.rs` (production)
**Depends on:** TASK-11

Use `LSP documentSymbol` on `workflow.rs` to identify block boundaries before extracting. Run `LSP diagnostics` after each function extraction, not just at the end.

**Pre-condition:** Write a three-task chain test (A→B→C, A fails) to confirm current skip-propagation behavior before extracting.

Extract one function at a time:

`poll_finished` takes `handles: &mut HashMap<String, InFlightTask>`, `task_timeouts: &HashMap<String, Duration>`, `state: &mut dyn StateStore`. Returns `Vec<String>` of finished task IDs. Iterates handles, checks elapsed time against `task_timeouts`, calls `handle.terminate()` and `state.mark_failed()` on timeout, then checks `handle.is_running()` for natural completion. Calls `state.save()?` after any timeout-triggered mark.

`process_finished` takes `id: &str`, `t: InFlightTask` (owned, already removed from handles by the caller), `state: &mut dyn StateStore`, `hook_executor: &Arc<dyn HookExecutor>`. Returns `Result<(), WorkflowError>`. The very first check must be: if the task is already `TaskStatus::Failed { .. }` (timed out by `poll_finished`), return `Ok(())` immediately without calling `wait()` or `state.save()` — the timeout path already saved. On the non-early-return path: call `t.handle.wait()`, update state to completed or failed, run the collect closure on success, fire hooks via `hook_executor`, call `state.save()?` once at the end.

`propagate_skips` takes `dag: &Dag`, `state: &mut dyn StateStore`, `tasks: &HashMap<String, Task>`. Returns `Result<(), WorkflowError>`. Runs the fixpoint loop: repeatedly finds Pending tasks whose dependencies include a Failed/Skipped entry, marks them `SkippedDueToDependencyFailure`, repeats until stable. Calls `state.save()?` once after the loop exits.

`build_summary` takes `state: &mut dyn StateStore`, `workflow_start: Instant`. Returns `WorkflowSummary` (infallible — no `state.save()` call; this function is read-only). Iterates `state.all_tasks()`, partitions into succeeded/failed/skipped, constructs `WorkflowSummary { succeeded, failed, skipped, duration: workflow_start.elapsed() }`.

None are `pub`. None have `&self`. Dispatch block is NOT extracted.

Final `run` loop body reads: interrupt check → `poll_finished` → `process_finished` per id → `propagate_skips` → dispatch → termination check.

Acceptance: all existing tests pass; LSP diagnostics zero errors; `run()` is under 100 lines.

---

## Execution Phases

| Phase | Tasks | Notes |
|-------|-------|-------|
| 1 (parallel) | TASK-1, TASK-2, TASK-3, TASK-4 | All independent |
| 2 (sequential) | TASK-5 | Gate checkpoint; blocks all Item work |
| 3 (parallel) | TASK-6, TASK-7, TASK-10 | All depend only on TASK-5 |
| 4 (parallel) | TASK-8, TASK-11 | TASK-8 depends on TASK-6; TASK-11 depends on TASK-10 |
| 5 (parallel) | TASK-9, TASK-12 | TASK-9 depends on TASK-8; TASK-12 depends on TASK-11 |

---

## Verification

After each task:
```
cargo check --workspace
cargo clippy --workspace
cargo test --workspace
```

All existing tests must remain green throughout. No new public API surface added except `interrupt_handle()`.

---

## Risk Flags

- **TASK-12 (`process_finished` early-return guard):** The guard `if matches!(state.get_status(&id), Some(TaskStatus::Failed { .. })) { return Ok(()); }` must be the first check in `process_finished`. Misplacing it causes a double-fault on timed-out tasks (calling `wait()` on an already-terminated handle). The existing `timeout_task_fails_and_dependent_skips` test will catch a regression, but only indirectly.
- **TASK-12 (`build_summary` is read-only):** `build_summary` must not call `state.save()`. It is a pure read of `state.all_tasks()`.
- **TASK-11 (`fire_hooks` must be a free function):** Making it a `&self` method reintroduces the borrow conflict. The function must have no `&self` receiver.
- **TASK-11 (`state.save()` placement):** The save must remain at the `run()` call site after `fire_hooks` returns, not inside `fire_hooks`.
- **TASK-6 (breaking public API):** `TaskClosure` is re-exported from `lib.rs`. Downstream users who construct `TaskClosure` directly (not via `.setup()`/`.collect()`) will need to update. Callers using the builder methods are unaffected.
- **TASK-3 (`interrupt` flag timing):** The flag must be set before `run()` is called. Setting it after `run()` returns is a no-op and would produce a test that passes vacuously.
