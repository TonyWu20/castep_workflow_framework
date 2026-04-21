## PR Review: `phase-4` → `main`

**Rating:** Approve

**Summary:** Phase 4 completes all 7 fix-plan tasks cleanly. The branch introduces `TaskSuccessors` as a proper newtype, moves `set_task_graph` from the `StateStore` trait to an inherent method on `JsonStateStore`, stores the successor graph in `Workflow` via `run()`, adds graph-aware CLI retry with proper `None`/`Some` fallback, and exercises the queued polling loop with a `DelayedHandle` test. All tests pass and clippy is clean. Four minor API hygiene items remain (none blocking).

**Cross-Round Patterns:** None. Shell injection (Round 2 Major) confirmed fixed in Round 3 and remains fixed. No regressions from prior rounds detected.

**Axis Scores:**
- Plan & Spec: Pass — all 7 fix-plan tasks completed, no out-of-scope additions
- Architecture: Pass — DAG-centric design, `TaskSuccessors` newtype, `set_task_graph` correctly scoped to `JsonStateStore`, crate boundaries respected
- Rust Style: Pass — meaningful error types, `#[must_use]` applied to getter, no dead code warnings, no clippy warnings
- Test Coverage: Pass — 6 BFS unit tests, serialization round-trip, `DelayedHandle` polling test, queued workflow integration test

---

## Fix Document for Author

### Issue 1: `TaskSuccessors::inner()` exposes raw `HashMap`, undermining newtype

**File:** `workflow_core/src/state.rs`
**Severity:** Minor
**Problem:** `TaskSuccessors` wraps `HashMap<String, Vec<String>>` to encapsulate the adjacency representation. The `pub fn inner()` method returns `&HashMap<String, Vec<String>>` directly, giving callers full access to the raw backing type. This defeats newtype encapsulation — any caller using `inner()` couples to the concrete representation and will break if the backing type ever changes. Grep confirms `inner()` has zero call sites in the workspace; it is dead API surface.
**Fix:** Remove `pub fn inner()` from `TaskSuccessors`. The existing `get()` and `is_empty()` methods cover all current usage. If a future use case arises that those cannot serve, add a purpose-specific method at that time.

### Issue 2: `TaskSuccessors` missing from `workflow_core` root re-exports

**File:** `workflow_core/src/lib.rs`
**Severity:** Minor
**Problem:** The re-export line (`pub use state::{JsonStateStore, StateStore, StateStoreExt, StateSummary, TaskStatus}`) includes every primary public type from the `state` module except `TaskSuccessors`. The CLI works around this by importing via the full path `workflow_core::state::TaskSuccessors`, but this breaks the crate's own convention where all primary public types are accessible at the crate root. Users relying on autocomplete or docs will miss `TaskSuccessors`.
**Fix:** Add `TaskSuccessors` to the re-export: `pub use state::{JsonStateStore, StateStore, StateStoreExt, StateSummary, TaskStatus, TaskSuccessors};`

### Issue 3: `downstream_tasks` BFS belongs in `workflow_core`, not the CLI binary

**File:** `workflow-cli/src/main.rs`
**Severity:** Minor
**Problem:** `downstream_tasks` takes `&TaskSuccessors` (a `workflow_core` type) and returns `HashSet<String>`. It has zero CLI-specific dependencies — no `clap`, no I/O, no formatting. BFS traversal over a domain graph type is domain logic, not presentation logic. Its 6 unit tests are also self-contained. Library consumers wanting graph-aware retry cannot reuse this function since it lives in a binary crate.
**Fix:** Move `downstream_tasks` to `workflow_core` — either as a free function in `state.rs` or as a method on `TaskSuccessors` (e.g., `TaskSuccessors::downstream_of(start: &[String]) -> HashSet<String>`). Re-export it and update the CLI import. Move or replicate the 6 unit tests alongside it.

### Issue 4: `QueuedProcessHandle::wait()` exit-code semantics are underdocumented

**File:** `workflow_utils/src/queued.rs`
**Severity:** Minor
**Problem:** `wait()` returns `finished_exit_code` which can be: `Some(0)` (job left the queue, assumed success), `Some(-1)` (scheduler query command itself failed), or `None` (called before `is_running()` transitions to finished). The `-1` sentinel conflates "cannot reach scheduler" with "process killed by signal" (Unix signal termination also conventionally maps to a negative code). Additionally, a comment at line 141 says "accounting query in `wait()` may refine" but `wait()` performs no such refinement.
**Fix:** Add a doc comment on `wait()` clarifying the three states and the approximate nature of `finished_exit_code`. Remove or fix the stale "accounting query" comment. No behavioral change needed — the caller in `workflow.rs` already handles all three cases defensively.
