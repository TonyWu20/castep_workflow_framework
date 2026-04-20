## PR Review: `phase-4` → `main`

**Rating:** Request Changes

**Summary:** Phase 4 delivers all planned queued execution support with clean architecture. Prior round's 11 fix tasks all applied successfully. Two new issues surfaced: shell injection risk in `QueuedRunner::submit` and `wait()` semantics that rely on undocumented caller contract. Four minor items remain.

**Cross-Round Patterns:** None

**Axis Scores:**

- Plan & Spec: Pass — All 7 deliverables implemented; graph-aware retry explicitly deferred
- Architecture: Pass — QueuedSubmitter in core, impl in utils, crate boundaries respected
- Rust Style: Partial — Shell injection surface in queued.rs, one clippy warning
- Test Coverage: Partial — Happy/error paths tested; PBS parsing and is_running/terminate untested

## Fix Document for Author

### Issue 1: Unused import `ProcessHandle` in queued integration test

**File:** `workflow_utils/tests/queued_integration.rs`
**Severity:** Minor
**Problem:** `use workflow_core::process::{OutputLocation, ProcessHandle}` imports `ProcessHandle` which is unused (clippy warning). The trait method `wait()` is available through the return type without this import.
**Fix:** Change the import to `use workflow_core::process::OutputLocation;` only.

### Issue 2: Shell injection in `QueuedRunner::submit`

**File:** `workflow_utils/src/queued.rs`
**Severity:** Major
**Problem:** `build_submit_cmd` interpolates `script_path`, `task_id`, and `log_dir` paths into a shell command string via `format!`, then `submit` passes this to `Command::new("sh").args(["-c", &submit_cmd])`. If any path or task ID contains shell metacharacters (spaces, semicolons, backticks), this enables command injection.
**Fix:** Replace `sh -c` with direct `Command::new("sbatch")` / `Command::new("qsub")` using `.args()` for each argument. This avoids shell interpretation entirely.

### Issue 3: `QueuedProcessHandle::wait()` non-blocking semantics undocumented

**File:** `workflow_utils/src/queued.rs`
**Severity:** Major
**Problem:** `wait()` returns immediately with `self.finished_exit_code` which is `None` until `is_running()` detects completion. The `ProcessHandle` trait's `wait()` doc doesn't specify blocking semantics, so callers may assume it blocks. The workflow's `process_finished` does call `wait()` only after `is_running()` returns false (correct), but the contract is implicit.
**Fix:** Add a doc comment to the `ProcessHandle` trait's `wait()` method in `workflow_core/src/process.rs` specifying: "Callers must ensure `is_running()` has returned `false` before calling `wait()`. Behavior when called on a still-running process is implementation-defined." Also add a doc comment on `QueuedProcessHandle::wait()` noting it returns immediately with cached state.

### Issue 4: `log_dir` defaults to `"."` for Queued tasks

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor
**Problem:** In the Queued dispatch branch, `self.log_dir.as_deref().unwrap_or_else(|| std::path::Path::new("."))` defaults to the process CWD, which may differ from `task.workdir`. Log files could end up in unexpected locations.
**Fix:** Default to `&task.workdir` instead of `"."` when `self.log_dir` is None.

### Issue 5: PBS job ID parsing untested

**File:** `workflow_utils/src/queued.rs`
**Severity:** Minor
**Problem:** `parse_job_id` handles both SLURM and PBS formats, but only SLURM is exercised in integration tests. The PBS path (trim + empty check) is untested.
**Fix:** Add unit tests for `parse_job_id` by either making it `pub(crate)` or adding a `#[cfg(test)]` module with direct tests. Test cases: valid PBS output (`"12345.server\n"`), empty PBS output, valid SLURM output.

### Issue 6: Missing doc comments on `QueuedRunner` public API

**File:** `workflow_utils/src/queued.rs`
**Severity:** Minor
**Problem:** `QueuedRunner`, `SchedulerKind`, and the `QueuedSubmitter` impl lack doc comments. These are public types re-exported from `workflow_utils`.
**Fix:** Add brief `///` doc comments to `QueuedRunner`, `SchedulerKind`, and the `submit` method.

### Issue 7: PATH mutation race condition in queued integration tests

**File:** `workflow_utils/tests/queued_integration.rs`
**Severity:** Blocking
**Problem:** `submit_returns_err_when_sbatch_unavailable` and `submit_with_mock_sbatch_returns_on_disk_handle` both call `std::env::set_var("PATH", ...)` which is process-global. By default Rust runs tests in parallel within the same binary, so the two tests race on PATH — one sets it to an empty dir, the other to mock_bin. This causes intermittent `NotFound` failures. The comment on line 44 acknowledges the need for `#[serial]` but it was never added. The `serial_test` crate is in Cargo.toml but unused.
**Fix:** Add `use serial_test::serial;` and `#[serial]` attribute to both PATH-mutating tests.
