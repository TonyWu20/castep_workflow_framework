## PR Review: `pre-phase-4` → `main` (v5)

**Rating:** Approve

**Summary:** Clean pre-Phase-4 quality branch. The `run()` decomposition into 5 free functions (`fire_hooks`, `process_finished`, `propagate_skips`, `build_summary`, `poll_finished`) significantly improves readability. TaskClosure widening to generic error types is the right ergonomic call. Test coverage meaningfully improved with RecordingExecutor, setup/collect failure tests, timeout test, and chain skip propagation. Three minor style issues remain — all deferred to Phase 4 where they align with planned work.

**Cross-Round Patterns:** None

**Axis Scores:**

- Plan & Spec: Pass — All planned items delivered: dep cleanup, TaskClosure widening, run() decomposition, merged validation, new tests, doc improvements
- Architecture: Pass — DAG-centric flow preserved, extracted helpers are free functions, crate boundaries clean, anyhow removed from libs
- Rust Style: Partial — fire_hooks uses stringly-typed state matching and takes PathBuf by value unnecessarily; misleading comment in hook test
- Test Coverage: Pass — Strong new coverage: setup failure, collect-doesn't-fail, hook recording, timeout, 3-task chain skip, signal isolation

---

## Fix Document for Author

All issues below are **deferred to Phase 4** (documented in `plans/phase-4/PHASE4_PLAN.md` under "Deferred from pre-phase-4 review").

### Issue 1: fire_hooks uses stringly-typed state matching

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor
**Problem:** `fire_hooks` accepts `final_state: &str` and matches against raw strings `"running"`, `"completed"`, `"failed"` (lines 276-280). A typo at any call site silently breaks hook dispatch. Root cause: `HookContext.state` in `monitoring.rs` is `String`.
**Fix:** When adding `HookTrigger::Periodic` in Phase 4 Part 2, refactor `HookContext.state` to an enum and update `fire_hooks` to accept it.

### Issue 2: fire_hooks takes PathBuf by value causing unnecessary clone

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor
**Problem:** `fire_hooks` takes `workdir: std::path::PathBuf`. The OnStart call site clones `task.workdir` (line 193) solely for this call, since `task.workdir` is later moved into `InFlightTask` (line 217).
**Fix:** Change to `workdir: &std::path::Path`; do `workdir.to_path_buf()` inside `HookContext` construction. Do alongside Phase 4 Part 2 touch of `fire_hooks`.

### Issue 3: Misleading ordering comment in hook test

**File:** `workflow_core/tests/hook_recording.rs`
**Severity:** Minor
**Problem:** Line 103 says "Expected order: success OnStart, failure OnStart, success OnComplete, failure OnFailure" implying cross-task ordering matters. Assertions are per-task filtered and safe, but the comment is misleading.
**Fix:** Change to: `// 4 hook calls total: 2 per task (cross-task order is non-deterministic)`
