## PR Review: `phase-6` → `main`

**Rating:** Request Changes

**Summary:** Phase 6 implements all five plan goals correctly. The CollectFailurePolicy fix and root_dir support are solid and well-tested. One blocking correctness issue must be resolved: single-mode task IDs silently gained a `_default` suffix, breaking state file continuity for existing workflows. Three minor trailing-newline violations and one dead code branch round out the required fixes.

**Cross-Round Patterns:** None — first review round.

**Deferred Improvements:** None

**Axis Scores:**

- Plan & Spec: Pass — All 5 goals (CollectFailurePolicy, root_dir, stdin, multi-param sweep, docs sweep) implemented as commissioned.
- Architecture: Pass — DAG-centric design preserved, builder patterns correct, crate boundaries respected.
- Rust Style: Partial — Dead code branch in `read_task_ids`; single-mode task ID regression; three files missing trailing newlines.
- Test Coverage: Pass — Integration tests for both collect policies, updated hook_recording test, new unit tests for `read_task_ids`.

---

## Fix Document for Author

### Issue 1: Dead `task_ids.is_empty()` branch in `read_task_ids`

**Classification:** Correctness
**File:** `workflow-cli/src/main.rs`
**Severity:** Minor
**Problem:** The `#[arg(required = false, default_value = "-")]` clap attribute ensures `task_ids` always contains at least one element. The `|| task_ids.is_empty()` branch on the stdin-detection condition can never be true and misleads readers about when stdin is triggered.
**Fix:** Remove the `|| task_ids.is_empty()` clause from the condition.

---

### Issue 2: Single-mode task ID `_default` suffix regression

**Classification:** Correctness
**File:** `examples/hubbard_u_sweep_slurm/src/main.rs`
**Severity:** Blocking
**Problem:** Single-mode passes `"default"` as the `second` parameter to `build_one_task`, which formats task IDs as `scf_U3.0_default` instead of the previous `scf_U3.0`. Existing workflow state files keyed on the old format will not match, causing tasks to be re-run or lost.
**Fix:** Change `second: &str` to `second: Option<&str>` in both `build_one_task` and `build_chain`. Single mode passes `None` (restoring the original `scf_U{u:.1}` format); product/pairwise modes pass `Some(&second)`. Update `task_id`, `workdir`, `dos_id`, and `dos_workdir` formations to match on `Some`/`None`.

---

### Issue 3: Missing trailing newline — `examples/hubbard_u_sweep_slurm/Cargo.toml`

**Classification:** Correctness
**File:** `examples/hubbard_u_sweep_slurm/Cargo.toml`
**Severity:** Minor
**Problem:** File ends without a trailing newline, violating the CLAUDE.md rule requiring trailing newlines on all source files.
**Fix:** Add a trailing newline at end of file.

---

### Issue 4: Missing trailing newline — `workflow_core/tests/collect_failure_policy.rs`

**Classification:** Correctness
**File:** `workflow_core/tests/collect_failure_policy.rs`
**Severity:** Minor
**Problem:** File ends without a trailing newline, violating the CLAUDE.md trailing-newline rule.
**Fix:** Add a trailing newline at end of file.

---

### Issue 5: Missing trailing newline — `workflow_core/src/prelude.rs`

**Classification:** Correctness
**File:** `workflow_core/src/prelude.rs`
**Severity:** Minor
**Problem:** File ends without a trailing newline, violating the CLAUDE.md trailing-newline rule.
**Fix:** Add a trailing newline at end of file.
