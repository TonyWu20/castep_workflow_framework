## v2 (2026-04-15)

# Phase 3 Fix Plan — Post-Final-Fixes Review

This plan addresses residual issues found after the 11-task final fix round. All items below were confirmed present in the actual source on the `phase-3` branch.

**Dependencies:** All 5 tasks are independent and can be applied in parallel.

---

### TASK-1: Fix tautological assertion in `test_terminate_long_running_process`

**File:** `workflow_utils/tests/process_tests.rs`
**Target function:** `test_terminate_long_running_process`
**Depends on:** none
**Can run in parallel with:** TASK-2, TASK-3, TASK-4, TASK-5

**Before** (line 55):

```rust
    assert!(result.exit_code.is_some() || result.exit_code.is_none());  // Either has code or was killed by signal
```

**After:**

```rust
    assert_ne!(result.exit_code, Some(0), "terminated process should not exit successfully");
```

**Why:** The existing assertion is `P || !P` — always true regardless of the actual value. It verifies nothing about termination behavior. The replacement checks what actually matters: a killed process must not report success.

**Verification:**

```bash
cargo test -p workflow_utils --test process_tests test_terminate_long_running_process
```

Expected: test passes.

---

### TASK-2: Fix copy-paste error in `add_edge` second-lookup error and strengthen test

**File:** `workflow_core/src/dag.rs`
**Target function:** `add_edge`
**Depends on:** none
**Can run in parallel with:** TASK-1, TASK-3, TASK-4, TASK-5

**Problem:** Both the `from` and `to` node lookups in `add_edge` produce identical error values. When the `to` node is missing, the error is indistinguishable from a missing `from` node.

**Step 1** — Fix the second `ok_or_else` block. Locate it by the surrounding context:

**Before** (the second lookup, lines 43–49):

```rust
        let &t = self
            .node_map
            .get(to)
            .ok_or_else(|| WorkflowError::UnknownDependency {
                task: to.to_string(),
                dependency: from.to_string(),
            })?;
```

**After:**

```rust
        let &t = self
            .node_map
            .get(to)
            .ok_or_else(|| WorkflowError::UnknownDependency {
                task: to.to_string(),
                dependency: to.to_string(),
            })?;
```

**Why:** When `to` is missing, it is the unknown node. Setting `dependency: to` makes the two error cases distinguishable. (The `UnknownDependency` variant doesn't perfectly model "target task not found", but using `dependency: to` is the most accurate fit given the existing variants, and makes both error arms distinguishable without adding a new error variant.)

**Step 2** — Strengthen the `unknown_dep_errors` test to assert field values. Locate the test by its name `unknown_dep_errors`:

**Before:**

```rust
    #[test]
    fn unknown_dep_errors() {
        let mut dag = Dag::new();
        dag.add_node("b".to_owned()).unwrap();
        assert!(matches!(
            dag.add_edge("missing", "b").unwrap_err(),
            WorkflowError::UnknownDependency { task: _, dependency: _ }
        ));
    }
```

**After:**

```rust
    #[test]
    fn unknown_dep_errors() {
        let mut dag = Dag::new();
        dag.add_node("b".to_owned()).unwrap();
        // from="missing" is absent — first lookup fails.
        // Expected: task="b" (the dependent), dependency="missing" (the unknown dep).
        assert!(matches!(
            dag.add_edge("missing", "b").unwrap_err(),
            WorkflowError::UnknownDependency { ref task, ref dependency }
            if task == "b" && dependency == "missing"
        ));
        // to="missing" is absent — second lookup fails.
        // After the Step 1 fix, both fields equal "missing" (the missing node's id).
        // This is distinguishable from the first case (where task="b", dependency="missing").
        dag.add_node("a".to_owned()).unwrap();
        assert!(matches!(
            dag.add_edge("a", "missing").unwrap_err(),
            WorkflowError::UnknownDependency { ref task, ref dependency }
            if task == "missing" && dependency == "missing"
        ));
    }
```

**Verification:**

```bash
cargo test -p workflow_core --lib -- dag::tests::unknown_dep_errors --exact
```

Expected: test passes.

---

### TASK-3: Change `cmd_status` to accept `&dyn StateStore`

**File:** `workflow-cli/src/main.rs`
**Target function:** `cmd_status`
**Depends on:** none
**Can run in parallel with:** TASK-1, TASK-2, TASK-4, TASK-5

**Before:**

```rust
fn cmd_status(state: &JsonStateStore) -> String {
```

**After:**

```rust
fn cmd_status(state: &dyn StateStore) -> String {
```

No changes are needed at the call sites in `main()` — `&JsonStateStore` coerces to `&dyn StateStore` automatically. No import changes are needed — `StateStore` and `StateStoreExt` are already imported at line 2. Note: `dyn StateStore` exposes `StateStoreExt::summary()` because `StateStoreExt` is blanket-implemented for all `T: ?Sized + StateStore` (including trait objects) via `impl<T: ?Sized + StateStore> StateStoreExt for T {}` in `state.rs`, so the `state.summary()` call inside the function body compiles unchanged.

**Why:** `cmd_status` only calls `all_tasks()` and `summary()`, both available on `&dyn StateStore` via `StateStoreExt`. The current concrete type makes it inconsistent with `cmd_inspect` and `cmd_retry`.

**Verification:**

```bash
cargo check -p workflow-cli
cargo test -p workflow-cli
```

Expected: no errors, all tests pass.

---

### TASK-4: Remove `mut` from unused-mut variable in CLI test

**File:** `workflow-cli/src/main.rs`
**Target function:** `status_shows_failed_after_load_raw` (inside `#[cfg(test)] mod tests`)
**Depends on:** none
**Can run in parallel with:** TASK-1, TASK-2, TASK-3, TASK-5

Locate the variable by its unique surrounding context in `status_shows_failed_after_load_raw`:

**Before:**

```rust
        let mut s = make_state(dir.path());
        s.save().unwrap();
```

**After:**

```rust
        let s = make_state(dir.path());
        s.save().unwrap();
```

**Why:** `save()` takes `&self`, so `mut` is unnecessary. The compiler reports `#[warn(unused_mut)]` for this binding.

**Verification:**

```bash
cargo clippy -p workflow-cli --tests 2>&1 | rg unused_mut
```

Expected: no output (warning resolved).

---

### TASK-5: Remove dead `WorkflowState` alias

**File:** `workflow_core/src/state.rs`
**Target:** `WorkflowState` type alias and its test
**Depends on:** none
**Can run in parallel with:** TASK-1, TASK-2, TASK-3, TASK-4

**Step 1** — Delete the alias. Locate by surrounding comment:

**Before:**

```rust
/// Alias for backward compatibility with existing code.
pub type WorkflowState = JsonStateStore;
```

**After:** (delete both lines entirely)

**Step 2** — Delete the `alias_workflow_state` test. Locate by the function name `alias_workflow_state`:

**Before:**

```rust
    #[test]
    fn alias_workflow_state() {
        let mut s: WorkflowState = JsonStateStore::new("alias", PathBuf::from("/tmp"));
        s.mark_completed("x");
        assert!(s.is_completed("x"));
    }
```

**After:** (delete the entire test function)

**Why:** `WorkflowState` is documented as a backward-compatibility alias, but it is not re-exported from `workflow_core/src/lib.rs` (confirmed via LSP — only 2 references: the definition and its own test). External crates cannot access it as `workflow_core::WorkflowState`. In its current form it provides no backward-compat value and is dead code.

**Verification:**

```bash
cargo test -p workflow_core --lib state::tests
rg WorkflowState workflow_core/src/
```

Expected: all tests pass; `rg` returns no matches (confirming no remaining references to `WorkflowState` in source).

---

## Dependency Graph

All 5 tasks are independent — they touch different files and have no shared prerequisites.

```
TASK-1  (workflow_utils/tests/process_tests.rs)    ─┐
TASK-2  (workflow_core/src/dag.rs)                  ├── all parallel
TASK-3  (workflow-cli/src/main.rs — fn signature)  ─┤
TASK-4  (workflow-cli/src/main.rs — test mut)      ─┤
TASK-5  (workflow_core/src/state.rs — alias)       ─┘
```

**Final verification after all tasks:**

```bash
cargo test --workspace
cargo clippy --workspace --all-targets 2>&1 | grep -E "^error|^warning.*unused"
```
