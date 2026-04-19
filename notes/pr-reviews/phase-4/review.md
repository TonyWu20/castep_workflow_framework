## PR Review: `phase-4` → `main`

**Rating:** Request Changes

**Summary:** Phase 4 lays solid groundwork for HPC queued execution — `OutputLocation`, `TaskPhase`, `QueuedSubmitter`, periodic hooks, and file-backed logging are well-layered. However, TASK-8 (graph-aware retry) code was never committed to source files despite checkpoint claiming completion, and TASK-12's integration test is untracked/failing. Code quality issues (dead variables, duplicated logic) need cleanup before merge.

**Cross-Round Patterns:** None

**Axis Scores:**

- Plan & Spec: Partial — TASK-8 code missing from source; TASK-12 untracked/failing
- Architecture: Pass — Clean layering, QueuedSubmitter justified at I/O boundary, crate boundaries respected
- Rust Style: Partial — Duplicated dead code in process_finished, missing Copy/Default derives, unused destructured fields
- Test Coverage: Partial — Good new tests but PATH mutation is unsound, no Workflow-level queued integration test

## Fix Document for Author

### Issue 1: TASK-8 code not in source files

**File:** `workflow_core/src/state.rs`
**Severity:** Blocking
**Problem:** The `task_successors` field, `set_task_graph()` trait method, and graph-aware `cmd_retry` in `workflow-cli/src/main.rs` exist only in the plan document. Commit 4357370 only updated checkpoint/report files. The feature is entirely unimplemented.
**Fix:** Implement TASK-8 as specified in `plans/phase-4/PHASE4_IMPLEMENTATION_PLAN.md` — add `task_successors` field to `JsonStateStore`, `set_task_graph()` to `StateStore` trait, `downstream_tasks()` BFS helper and graph-aware `cmd_retry` to `workflow-cli/src/main.rs`.

### Issue 2: TASK-12 integration test untracked and failing

**File:** `workflow_utils/tests/queued_integration.rs`
**Severity:** Blocking
**Problem:** The file is untracked (never committed). The execution report shows `submit_with_mock_sbatch_returns_on_disk_handle` panicking with `Io(Os { code: 2, kind: NotFound })`. Additionally, no test exercises `Workflow::run()` with a Queued task through the full dispatch loop.
**Fix:** Commit the test file. Debug the mock sbatch test (likely the mock PATH isn't being inherited by the `sh -c` subprocess). Add a Workflow-level integration test that uses a mock `QueuedSubmitter` to verify the dispatch path in `workflow.rs:223-244`.

### Issue 3: Duplicated dead code in `process_finished`

**File:** `workflow_core/src/workflow.rs`
**Severity:** Major
**Problem:** Lines 357-395 compute three representations of the same completed/failed boolean: (1) `final_state: &str` at line 357 (shadowed, dead), (2) `final_state: TaskPhase` at line 385 (dead), (3) `task_phase: TaskPhase` at line 391 (used). Two are dead variables.
**Fix:** Remove the string-based `final_state` binding (rename the let-binding at line 357 to just capture `exit_code`). Remove the duplicate `final_state` at line 385. Keep only `task_phase` computation at line 391.

### Issue 4: `ExecutionMode::Queued` fields unused in dispatch

**File:** `workflow_core/src/workflow.rs`
**Severity:** Major
**Problem:** At line 223, `ExecutionMode::Queued { submit_cmd, poll_cmd, cancel_cmd }` destructures three fields that are never read — dispatch delegates to `self.queued_submitter` which builds its own commands from `SchedulerKind`. This is a design mismatch: either pass these fields to the submitter, or simplify the enum.
**Fix:** Either (a) change `QueuedSubmitter::submit()` to accept the command strings from the enum, or (b) simplify `ExecutionMode::Queued` to a unit variant (or just a marker) since the commands come from `QueuedRunner`'s `SchedulerKind`. Option (b) is cleaner — the submitter owns its command logic.

### Issue 5: PATH mutation without `#[serial]` in tests

**File:** `workflow_utils/tests/queued_integration.rs`
**Severity:** Major
**Problem:** `std::env::set_var("PATH", ...)` is process-global and unsound in parallel test execution. Comment mentions `#[serial]` but doesn't use it. As of Rust 1.83, `set_var` is `unsafe`.
**Fix:** Add `serial_test` dev-dependency and `#[serial]` attribute to tests that mutate PATH. Alternatively, restructure tests to spawn a subprocess with a modified env (preferred for soundness).

### Issue 6: `TaskPhase` missing `Copy`, `PartialEq`, `Eq` derives

**File:** `workflow_core/src/monitoring.rs`
**Severity:** Minor
**Problem:** `TaskPhase` is a fieldless enum but only derives `Debug, Clone, Serialize, Deserialize`. This forces unnecessary `.clone()` calls (e.g., `phase.clone()` in `fire_hooks` match).
**Fix:** Add `Copy, PartialEq, Eq` to the derive list. Remove `.clone()` calls on `TaskPhase` values.

### Issue 7: `QueuedProcessHandle::workdir` is dead field

**File:** `workflow_utils/src/queued.rs`
**Severity:** Minor
**Problem:** The `workdir: PathBuf` field is stored during construction but never read by any method.
**Fix:** Remove the field, or document its intended future use with a TODO comment.

### Issue 8: `pub use queued::*` glob re-export

**File:** `workflow_utils/src/lib.rs`
**Severity:** Minor
**Problem:** Glob re-export leaks `QueuedProcessHandle` which should be opaque (callers use `Box<dyn ProcessHandle>`).
**Fix:** Replace with explicit re-exports: `pub use queued::{QueuedRunner, SchedulerKind};`

### Issue 9: `SystemProcessRunner` should derive `Default`

**File:** `workflow_utils/src/executor.rs`
**Severity:** Minor
**Problem:** `new()` returns `Self { log_dir: None }` — this is exactly what `Default` would produce.
**Fix:** Add `#[derive(Default)]` and keep `new()` as a convenience alias (or remove it in favor of `Default::default()`).

### Issue 10: Periodic hook test sleeps 8 seconds

**File:** `workflow_core/tests/hook_recording.rs`
**Severity:** Minor
**Problem:** `periodic_hook_fires_during_long_task` uses `sleep 8` — unnecessarily slow for CI.
**Fix:** Reduce to `sleep 2` (still sufficient with `interval_secs: 1` to fire at least once).

### Issue 11: `build_submit_cmd` doesn't shell-escape paths

**File:** `workflow_utils/src/queued.rs`
**Severity:** Minor
**Problem:** Paths are interpolated directly into the shell command string via `format!`. Paths with spaces or special characters will break.
**Fix:** Wrap paths in single-quotes with internal quote escaping, or use `shell_escape` crate.
