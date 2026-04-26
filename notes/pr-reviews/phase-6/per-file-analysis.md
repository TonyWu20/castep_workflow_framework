# Per-File Analysis: phase-6 -> main

## File: workflow_core/src/task.rs

**Intent:** Added `CollectFailurePolicy` enum (FailTask/WarnOnly), `collect_failure_policy` field on `Task`, and builder method. Generic `<F, E>` signatures for `setup` and `collect` already existed.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A (new enum + field)
- Dead code or unused imports? No
- New public API: tests present? No — the enum's behavior is tested through integration tests in `collect_failure_policy.rs`
- Change appears within plan scope? Yes — TASK-1

**Notes:** The enum is correctly `#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]`. Default is `FailTask`. `pub(crate)` visibility on the struct field correctly limits external mutation. The `<F, E>` generic signatures on `setup` and `collect` in task.rs already matched ARCHITECTURE.md after the Phase 5B diff — documentation accuracy confirmed.

---

## File: workflow_core/src/workflow.rs

**Intent:** Added `root_dir` field + builder; added `collect_failure_policy` to `InFlightTask`; rewrote `process_finished()` to run collect before final status decision; resolved workdir and log_dir against root_dir at dispatch.

**Checklist:**
- Unnecessary clone/unwrap/expect? The `root.join(&task.workdir)` clone is intentional — `task` is consumed by `self.tasks.remove()`, so the clone is necessary.
- Error handling: stringly-typed `e.to_string()` for collect errors in state — consistent with existing pattern throughout the file
- Dead code or unused imports? No
- New public API: tests present? Inline tests verify workflow behavior; separate integration test file covers collect_failure_policy
- Change appears within plan scope? Yes — TASK-1, TASK-2, TASK-3

**Notes:**
- The `process_finished()` rewrite (lines 389-457) is the most complex change. The collect-before-status ordering is correct. The state re-read pattern after `mark_failed` handles the collect-overrides-exit-code case properly.
- `InFlightTask::workdir` now holds the resolved path — hooks and collect closures see the correct path, which is the intended behavior.
- `resolved_log_dir` is computed once at the top of `run()` and reused. The `queued.rs` path uses `resolved_log_dir.as_deref().unwrap_or(resolved_workdir.as_path())`, which is correct — if no log_dir is configured, falls back to the resolved workdir.
- `root_dir` resolution only applies to relative paths, preserving absolute paths unchanged. This matches the plan specification.

---

## File: workflow_core/src/lib.rs

**Intent:** Re-exported `CollectFailurePolicy` from crate root.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A
- Dead code or unused imports? No
- New public API: tests present? N/A — single-line re-export
- Change appears within plan scope? Yes — TASK-1

**Notes:** Correct placement in the existing re-export chain.

---

## File: workflow_core/src/prelude.rs

**Intent:** Re-exported `CollectFailurePolicy` in prelude module.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A
- Dead code or unused imports? No
- New public API: tests present? N/A — re-export
- Change appears within plan scope? Yes — TASK-1

**Notes:** Trailing newline present (confirmed via manifest). No issues.

---

## File: workflow_core/tests/collect_failure_policy.rs

**Intent:** New integration test file verifying both `FailTask` and `WarnOnly` policies.

**Checklist:**
- Unnecessary clone/unwrap/expect? `tempfile::tempdir().unwrap()` and `.unwrap()` on `add_task` — standard test patterns
- Error handling: N/A (tests)
- Dead code or unused imports? No
- New public API: tests present? Yes — this is the test file itself
- Change appears within plan scope? Yes — TASK-3

**Notes:**
- Two tests cover the two policy modes. Both use the same pattern: create workflow, add task with failing collect, run, verify state.
- `StubHandle::wait` takes ownership of the child via `.take()`, correct.
- Test doubles (`StubRunner`, `StubHandle`, `StubHookExecutor`) are correct and complete.
- Trailing newline missing (confirmed via manifest).

---

## File: workflow_core/tests/hook_recording.rs

**Intent:** Added explicit `.collect_failure_policy(CollectFailurePolicy::WarnOnly)` to the existing `collect_failure_does_not_fail_task` test, and imported `CollectFailurePolicy`.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A
- Dead code or unused imports? No
- New public API: N/A — existing test updated
- Change appears within plan scope? Yes — TASK-3

**Notes:** This change is necessary because the default `CollectFailurePolicy` is now `FailTask`. Without the explicit `WarnOnly`, the test would fail. The test's intent is to verify `WarnOnly` semantics, so this is correct.

---

## File: workflow-cli/src/main.rs

**Intent:** Added `read_task_ids()` function for stdin-based task ID input to `workflow-cli retry` command.

**Checklist:**
- Unnecessary clone/unwrap/expect? `task_ids.to_vec()` is a defensive copy on the non-stdin branch — reasonable for a public-facing function result.
- Error handling: `anyhow::bail!` with descriptive messages for three error conditions (TTY, read failure, empty stdin)
- Dead code or unused imports? No
- New public API: tests present? Yes — two new tests for `read_task_ids`
- Change appears within plan scope? Yes — TASK-4

**Notes:**
- The clap default `#[arg(required = false, default_value = "-")]` ensures `task_ids` is always non-empty when clap parses. The dead `task_ids.is_empty()` branch from the initial draft review was already removed in fix-plan TASK-1.
- `io::stdin().read_to_string(&mut input)` on an empty pipe returns Ok with empty string. The test comment correctly notes this behavior and the "no task IDs found" bail fires as expected.
- The function is `fn` (private), not `pub fn`, so it is not a new public API.

---

## File: examples/hubbard_u_sweep_slurm/src/config.rs

**Intent:** Added `sweep_mode`, `second_values`, and `workdir` CLI fields to `SweepConfig`. Updated `parse_empty_string` test assertion.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A (CLI config fields)
- Dead code or unused imports? No
- New public API: tests present? Yes — test assertion updated for consistency
- Change appears within plan scope? Yes — TASK-5, TASK-6

**Notes:**
- `sweep_mode` and `workdir` have clap defaults; `second_values` is optional (defaults to `vec!["kpt8x8x8"]` in product/pairwise mode).
- Test assertion change from `!err.is_empty()` to `err.contains("invalid")` improves assertion specificity.

---

## File: examples/hubbard_u_sweep_slurm/src/main.rs

**Intent:** Extended to support multi-parameter sweeps (product/pairwise modes), added `build_chain` for SCF-DOS dependent task chains, added `--workdir` root_dir wiring.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: Uses `WorkflowError` consistently in closures; `build_sweep_tasks` returns `anyhow::Error` (binary crate convention per CLAUDE.md)
- Dead code or unused imports? No
- New public API: tests present? No — the binary example has no tests
- Change appears within plan scope? Yes — TASK-5

**Notes:**
- `build_chain` creates a DOS task with no setup/collect closures. The comment explains this is sufficient for dry-run validation per plan scope.
- Single-mode mode calls `build_one_task(config, u, None, ...)` — task IDs remain in the original format (e.g., `scf_U3.0`). No behavioral change. (This was previously incorrectly flagged as introducing a `_default` suffix, which was the state before fix-plan TASK-2.)
- `parse_second_values` is a simple split+trim, consistent with the existing `parse_u_values` pattern but without f64 conversion.
- Minor duplication of `second_values` extraction in both "product" and "pairwise" arms (~4 lines each) — the match arms have different iteration patterns so extraction would be marginal.

---

## File: examples/hubbard_u_sweep_slurm/Cargo.toml

**Intent:** Added `itertools` workspace dependency.

**Checklist:**
- No substantive issues.

**Notes:** Trailing newline missing (confirmed via manifest).

---

## File: Cargo.toml

**Intent:** Added `itertools = "0.14"` to workspace dependencies.

**Checklist:**
- No issues.

---

## File: Cargo.lock

**Intent:** Added `itertools` 0.14.0 and `either` 1.15.0.

**Checklist:**
- No issues.

---

## File: ARCHITECTURE.md

**Intent:** Updated `setup`/`collect` builder signatures and `JsonStateStore::load`/`load_raw` to match actual implementation.

**Checklist:**
- No issues.

**Notes:** `setup<F, E>` and `collect<F, E>` now match actual signatures. `load`/`load_raw` corrected from instance methods to static constructors.

---

## File: ARCHITECTURE_STATUS.md

**Intent:** Updated Phase 5 description, added Phase 6 section, updated Next Steps.

**Checklist:**
- No issues.

**Notes:** `CollectFailurePolicy` re-export entry added. `TaskClosure` description updated. Next Steps updated to reflect Phase 6 status.

---

## File: flake.nix

**Intent:** Updated LLM model endpoint URLs and model names for apex models.

**Checklist:**
- No issues (not source code).

---

## File: plans/phase-6/PHASE_PLAN.md

**Intent:** New phase plan document for Phase 6.

**Checklist:**
- No issues.

**Notes:** Contains 5 goals with detailed design, critical files, and sequencing notes.

---

## File: plans/phase-6/phase6_implementation.toml

**Intent:** Task-level implementation breakdown for Phase 6.

**Checklist:**
- No issues.

**Notes:** 6 tasks with dependency chain.

---

## File: notes/pr-reviews/phase-6/draft-fix-document.md

**Intent:** Draft document identifying the dead code issue.

**Checklist:**
- Not code (review artifact).

---

## File: notes/pr-reviews/phase-6/fix-plan.toml

**Intent:** Fix plan for review issues.

**Checklist:**
- Not code (review artifact).

---

## File: notes/plan-reviews/PHASE_PLAN/decisions.md

**Intent:** Architectural review decisions for Phase 6 plan.

**Checklist:**
- Not code (review artifact).

---

## File: notes/pr-reviews/phase-6/deferred.md

**Intent:** Deferred items carried forward from prior phases.

**Checklist:**
- Not code (review artifact).

---
