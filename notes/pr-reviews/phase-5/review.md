# Phase 5A PR Review (v2 — 2026-04-23)

## PR Review: `phase-5` → `main`

**Rating:** Request Changes

**Summary:** The v1 fix round resolved Issue 2 completely (`parse_u_values` now returns `Result` and the call site propagates the error correctly). Issue 1 is partially resolved: `JOB_SCRIPT_NAME` was defined in `queued.rs` and re-exported from `workflow_utils`, but the two direct consumers — `main.rs` (setup closure) and the integration tests in `queued_integration.rs` — still hardcode `"job.sh"`. The invisible contract remains in effect at both call sites. Two small mechanical fixes are needed before merge.

**Cross-Round Patterns:** None (only one prior fix-plan version exists)

**Deferred Improvements:** 8 items carried forward → `notes/pr-reviews/phase-5/deferred.md`

**Axis Scores:**

- Plan & Spec: Partial — Issue 1 constant defined but not adopted by consumers; Issue 2 fully resolved
- Architecture: Pass — layer boundaries respected; `anyhow` only in binary; const placement in `workflow_utils` appropriate
- Rust Style: Partial — two minor style nits (double `s.trim()`, `anyhow::anyhow!(e)` vs `anyhow::Error::msg`) deferred as improvements
- Test Coverage: Partial — no unit tests for `parse_u_values` error path; integration tests still use hardcoded literal (addressed in fix)

---

## Fix Document for Author

### Issue 1: Consumer and integration tests still hardcode `"job.sh"` despite `JOB_SCRIPT_NAME` being exported

**Classification:** Defect
**File:** `examples/hubbard_u_sweep_slurm/src/main.rs`
**Severity:** Major
**Problem:** The setup closure in `main.rs` writes `write_file(workdir.join("job.sh"), &job_script)?` using a hardcoded literal. `JOB_SCRIPT_NAME` was exported from `workflow_utils` specifically so consumers would reference the same constant as `QueuedRunner::submit()`. Using a hardcoded string here means the consumer and the runner can silently diverge if the constant is ever changed — the exact problem the constant was introduced to prevent.
**Fix:** Add `JOB_SCRIPT_NAME` to the `use workflow_utils::{...}` import block and replace the `"job.sh"` literal in the `write_file` call with `JOB_SCRIPT_NAME`.

### Issue 2: Integration tests hardcode `"job.sh"` instead of using `JOB_SCRIPT_NAME`

**Classification:** Defect
**File:** `workflow_utils/tests/queued_integration.rs`
**Severity:** Minor
**Problem:** Both integration tests (`submit_returns_err_when_sbatch_unavailable` at line 39 and `submit_with_mock_sbatch_returns_on_disk_handle` at line 80) create `"job.sh"` with a hardcoded literal. This creates the same drift risk in the test layer: if `JOB_SCRIPT_NAME` is ever updated, the tests pass against the old name while the runtime uses the new one.
**Fix:** Add `JOB_SCRIPT_NAME` to the `use workflow_utils::{...}` import on line 9 and replace both `workdir.join("job.sh")` calls with `workdir.join(JOB_SCRIPT_NAME)`.
