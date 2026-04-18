## PR Review: `pre-phase-4` → `main`

**Rating:** Request Changes

**Summary:** Excellent preparatory work: the `run()` extraction into five free functions is clean and well-motivated, TaskClosure ergonomics are improved, and new test coverage (timeout, setup/collect failure, hook firing, chain-skip propagation) fills real gaps. Two issues remain: `TaskClosure` lost its `Send + Sync` bounds during the widening, and one test closure has broken indentation.

**Axis Scores:**

- Plan & Spec: Pass — All 12 pre-phase-4 tasks implemented: dependency hygiene, signal isolation test, StateStore docs, TaskClosure widening, run() extraction, test fixtures
- Architecture: Pass — Crate boundaries respected, no anyhow in libs, free functions avoid borrow conflicts, DAG-centric design preserved
- Rust Style: Partial — TaskClosure type alias dropped Send+Sync (public API regression); one test has severely misaligned indentation
- Test Coverage: Pass — Good new coverage: timeout, setup/collect failure, hook lifecycle, chain-skip propagation, resume; per-task hook ordering assertions are sound

---

## Fix Document for Author

### Issue 1: `TaskClosure` type alias missing `Send + Sync` bounds

**File:** `workflow_core/src/task.rs`
**Severity:** Blocking
**Problem:** The old type was `Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>`. The new type is `Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + 'static>` — the `+ Send + Sync` on the outer `dyn Fn` trait object was dropped. The builder methods (`setup`, `collect`) still enforce `F: Send + Sync + 'static`, so currently-compiled code works, but the type alias itself no longer guarantees `Send + Sync`. Anyone constructing a `TaskClosure` directly (bypassing the builder) could create a non-Send+Sync closure, and if `Task` is later required to be `Send` (e.g., moved into a thread pool), compilation will fail. This is a public API regression.
**Fix:** Add `+ Send + Sync` back to the type alias:
```rust
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static>;
```

### Issue 2: Misaligned `.setup()` closure in `chain_respects_order` test

**File:** `workflow_core/src/workflow.rs`
**Severity:** Minor
**Problem:** The `.setup(move |_| -> Result<(), std::io::Error> { ... })` closure for task "a" in the `chain_respects_order` test starts at column 0 and uses inconsistent 6-space/4-space indentation, while the identical pattern for task "b" is correctly indented. This appears to be a merge/rebase artifact.
**Fix:** Re-indent the closure to match the "b" task's `.setup()` style (12-space indent for `.setup(`, 16-space for body).
