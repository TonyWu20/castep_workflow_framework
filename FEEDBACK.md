# PR Review: `qwopus3.5` → `main`

**Rating:** Request Changes

**Summary:** The branch successfully completes the Phase 1.2 migration — removing the async executor layer, adapter crates, and TOML-driven schema in favour of the closure-based `workflow_utils` integration. The `bon` builder for `Workflow`, monitoring hook wiring, and `Default` impl for `Dag` are all correct. However, the branch leaves `workflow_cli` in a broken state that prevents the workspace from compiling, and has a few API and style issues that need addressing before merge.

**Axis Scores:**
- Plan & Spec: Partial — Phase 1.2 core goal achieved, but `workflow_cli` is left broken (stated as "to be refactored later" in a commit message, not resolved)
- Architecture: Pass — DAG-centric design preserved; sync-over-thread model consistent with Phase 1.1 pattern; `workflow_utils` correctly integrated
- Rust Style: Partial — underscore-prefixed variables that are actively used; `pub use` re-exports removed without replacement
- Test Coverage: Partial — new builder paths tested; `duplicate_task_id_errors` test conflates two conditions; `resume()` correctness not tested

---

## Fix Document for Author

### Issue 1: `workflow_cli` references deleted modules — workspace does not compile

**File:** `workflow_cli/src/main.rs:16`, `workflow_cli/src/main.rs:49`
**Severity:** Blocking
**Problem:** `workflow_cli` imports `workflow_core::executor::Executor` and `workflow_core::schema::WorkflowDef`, both of which were deleted in this branch. The workspace will not compile.
**Fix:** Either remove `workflow_cli` from the workspace `members` list in `Cargo.toml` (it was already removed — verify it is also excluded from `Cargo.lock` resolution), or stub/rewrite `workflow_cli/src/main.rs` to not reference the deleted modules. The commit message acknowledges this ("workflow_cli to be refactored later") — that refactor must land before merge, or the crate must be fully removed from the workspace.

---

### Issue 2: `_monitors` and `_task_workdirs` use misleading underscore prefix

**File:** `workflow_core/src/workflow.rs:83–89`
**Severity:** Minor
**Problem:** Both variables are named with a leading `_` but are actively used throughout `run()`. In Rust, `_foo` conventionally means "intentionally unused — suppress the warning." Using it on live variables is misleading to readers and linters.
**Fix:** Rename to `monitors` and `task_workdirs`.

---

### Issue 3: `pub use` re-exports removed — breaking downstream API

**File:** `workflow_core/src/lib.rs`
**Severity:** Major
**Problem:** The branch removes:
```rust
pub use task::Task;
pub use workflow::Workflow;
pub use state::{TaskStatus, WorkflowState};
```
Any downstream code using `workflow_core::Task`, `workflow_core::Workflow`, or `workflow_core::TaskStatus` directly now fails to compile. The modules are still `pub mod`, so callers must now use the full path (`workflow_core::task::Task`), which is an unannounced breaking change.
**Fix:** Restore the three `pub use` lines in `lib.rs`, or document the intentional API break and update all call sites.

---

### Issue 4: `resume()` silently ignores state if CWD differs

**File:** `workflow_core/src/workflow.rs:60–63`
**Severity:** Major
**Problem:** `resume(name)` calls `Self::builder().name(name.into()).build()` with no `state_dir` parameter, so `state_path` always defaults to `./<name>.workflow.json` relative to the current working directory. If the caller's CWD differs from where the workflow was originally run, `run()` silently starts fresh instead of resuming. There is no error or warning.
**Fix:** Add a `state_dir` parameter to `resume()`:
```rust
pub fn resume(name: impl Into<String>, state_dir: impl Into<PathBuf>) -> Result<Self> {
    Self::builder().name(name.into()).state_dir(state_dir.into()).build()
}
```
And add a test that verifies `resume()` actually loads existing state from a non-default path.

---

### Issue 5: `duplicate_task_id_errors` test no longer tests its stated condition cleanly

**File:** `workflow_core/src/workflow.rs:349–362`
**Severity:** Minor
**Problem:** The test adds "a", then "b" (depends_on "a"), then tries to add "a" again (depends_on "b"). The third call fails at the duplicate-ID check in `add_task` before `build_dag()` is ever called — so the cycle (a→b→a) is never exercised. The test name says "duplicate task id errors" but the setup is unnecessarily complex for that purpose.
**Fix:** Simplify back to the original intent — add "a" twice, assert the second returns `Err`. Keep a separate test for cycle detection if desired.
