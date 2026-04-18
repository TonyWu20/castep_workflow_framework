## PR Review: `pre-phase-4` → `main` (v4)

**Rating:** Request Changes

**Summary:** Solid pre-phase-4 cleanup. The `run()` decomposition into 5 helper functions (`fire_hooks`, `process_finished`, `propagate_skips`, `build_summary`, `poll_finished`) significantly improves readability. The `TaskClosure` widening to accept generic error types is the right ergonomic call. Test coverage is meaningfully improved with `RecordingExecutor` and new integration tests. One blocking compilation error in the example and three minor cleanup items remain.

**Axis Scores:**

- Plan & Spec: Pass — All changes are appropriate pre-phase-4 cleanup (dep normalization, TaskClosure ergonomics, run() decomposition, dead code removal, new tests)
- Architecture: Pass — DAG-centric design preserved, extracted helpers are free functions, crate boundaries respected
- Rust Style: Partial — Blocking compilation error in example, duplicated test helper, unconditional disk writes in hot loop
- Test Coverage: Pass — Meaningful new integration tests for setup failure, collect failure, hook firing, timeout, 3-task skip propagation

---

## Fix Document for Author

### Issue 1: E0283 compilation error in hubbard_u_sweep example

**File:** `examples/hubbard_u_sweep/src/main.rs`
**Severity:** Blocking
**Problem:** The `setup()` method is now generic over `<F, E>` where `E: std::error::Error + Send + Sync + 'static`. The closure uses `?` with mixed error types (`WorkflowError` and I/O errors) so the compiler cannot infer the concrete `E`. This is the only workspace compilation error.
**Fix:** Add explicit return type annotation: `.setup(move |workdir| -> Result<(), WorkflowError> {`

### Issue 2: dead_code warnings in shared test helper module

**File:** `workflow_core/tests/common/mod.rs`
**Severity:** Minor
**Problem:** `RecordingExecutor` and `direct()` trigger dead_code warnings because each integration test binary compiles `common` independently and not every test file uses all items.
**Fix:** Add `#[allow(dead_code)]` on `RecordingExecutor` struct and `direct()` fn. (Note: `#![allow(dead_code)]` is invalid in non-crate-root modules.)

### Issue 3: Duplicated `direct()` helper in hook_recording.rs

**File:** `workflow_core/tests/hook_recording.rs`
**Severity:** Minor
**Problem:** Local `direct()` function is identical to `common::direct()`. The file already imports from `common`. Removing the local copy also makes `HashMap` and `ExecutionMode` imports unused.
**Fix:** Remove local `fn direct()`, remove `HashMap` and `task::ExecutionMode` from imports, add `use common::direct;`.

### Issue 4: `propagate_skips` writes to disk unconditionally on every poll cycle

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor
**Problem:** `propagate_skips()` calls `state.save()` regardless of whether any tasks were actually skipped. Since the main loop calls this every 50ms, it causes unnecessary disk writes.
**Fix:** Track an `any_skipped` flag; only call `state.save()` when true.
