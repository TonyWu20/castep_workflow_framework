# Phase 6 Per-File Analysis

## File: workflow_core/src/task.rs

**Intent:** Added `CollectFailurePolicy` enum with `FailTask`/`WarnOnly` variants, field on `Task`, builder method, and default initialization.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: meaningful types or stringly-typed? N/A — this is a policy enum, not error handling
- Dead code or unused imports? No
- New public API: tests present? No — `CollectFailurePolicy` itself has no unit tests; integration tests exist in `collect_failure_policy.rs`
- Change appears within plan scope? Yes — TASK-1

**Notes:** Enum derives `Copy` which is appropriate for a small policy marker. The field is `pub(crate)` which is correct — internal to the workflow execution path, not part of the public Layer 3 API. Doc comments are thorough.

---

## File: workflow_core/src/lib.rs

**Intent:** Re-exported `CollectFailurePolicy` from crate root.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A
- Dead code or unused imports? No
- New public API: tests present? N/A — re-export
- Change appears within plan scope? Yes — TASK-1

**Notes:** Single-line change. Correct placement in the existing re-export chain.

---

## File: workflow_core/src/prelude.rs

**Intent:** Re-exported `CollectFailurePolicy` in prelude module.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A
- Dead code or unused imports? No
- New public API: tests present? N/A — re-export
- Change appears within plan scope? Yes — TASK-1

**Notes:** Note: file still missing trailing newline (the CLAUDE.md rule says to always add trailing newlines). This was listed as TASK-6 item 6 but the fix appears to have not landed here (or the diff stat shows only 2 lines changed for this file). Confirmed: the file ends without a newline.

---

## File: workflow_core/src/workflow.rs

**Intent:** Added `root_dir` field and builder to `Workflow`; added `collect_failure_policy` field to `InFlightTask`; rewrote `process_finished()` to run collect before final status decision; resolved workdir and log_dir against root_dir at dispatch time.

**Checklist:**
- Unnecessary clone/unwrap/expect? `root.join(&task.workdir)` clones `task_workdir` before passing to `InFlightTask` — this is intentional since `task` is consumed by `self.tasks.remove()`, so the clone is necessary, not unnecessary.
- Error handling: meaningful types or stringly-typed? `process_finished` uses `e.to_string()` for error propagation into state — consistent with existing pattern in the file
- Dead code or unused imports? No
- New public API: tests present? Yes — inline tests in `workflow.rs` cover the workflow behavior; separate integration test file covers `collect_failure_policy`
- Change appears within plan scope? Yes — TASK-1, TASK-2, TASK-3

**Notes:**
- `process_finished()` rewrite is the most complex change. The logic now: (1) wait for process, (2) if exit != 0, mark failed immediately, (3) if exit == 0, run collect, (4) if collect fails with FailTask, mark failed, (5) re-read state to decide phase. The re-read of state (`state.get_status(id)`) after potential `mark_failed` is the correct pattern to handle the collect-overrides-exit-code case.
- `resolved_log_dir` is computed once at the top of `run()` and reused. The QueuedSubmitter path uses `resolved_log_dir.as_deref().unwrap_or(resolved_workdir.as_path())` which is correct — if no log_dir is configured, falls back to the resolved workdir.
- `root_dir` is `Option<std::path::PathBuf>` on the struct, set via builder. Resolution only applies to relative paths, preserving absolute paths unchanged. This matches the plan specification.
- The `InFlightTask::workdir` field now holds the resolved path instead of the original task workdir. This means hooks and collect closures see the resolved path, which is the intended behavior.

---

## File: workflow-cli/src/main.rs

**Intent:** Added `read_task_ids()` function for stdin-based task ID input to `workflow-cli retry` command.

**Checklist:**
- Unnecessary clone/unwrap/expect? No. `task_ids.to_vec()` on the non-stdin branch is a defensive copy — reasonable for a public-facing function result.
- Error handling: meaningful types or stringly-typed? Uses `anyhow::bail!` with descriptive messages for three error conditions (TTY, read failure, empty stdin).
- Dead code or unused imports? No
- New public API: tests present? Yes — two new tests for `read_task_ids`
- Change appears within plan scope? Yes — TASK-4

**Notes:**
- The `#[arg(required = false, default_value = "-")]` clap attribute means `task_ids` will always be non-empty when clap parses — either the user provides values, or the default `"-"` is used. This means `task_ids.is_empty()` can never be true in practice. The empty check in `read_task_ids` is therefore dead code. This is a minor redundancy, not a correctness bug.
- `io::stdin().read_to_string(&mut input)` on an empty pipe returns Ok with empty string (no bytes). The test comment correctly notes this behavior and the "no task IDs found" bail fires as expected.
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
- All three new fields are `String` / `Option<String>`. `sweep_mode` and `workdir` use clap defaults. `second_values` is optional — when absent in product/pairwise mode, the example defaults to `vec!["kpt8x8x8"]`.
- The test assertion change from `!err.is_empty()` to `err.contains("invalid")` is an improvement in assertion specificity.

---

## File: examples/hubbard_u_sweep_slurm/src/main.rs

**Intent:** Extended to support multi-parameter sweeps (product/pairwise modes), added `build_chain` for SCF→DOS dependent task chains, added `--workdir` root_dir wiring.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: Uses `WorkflowError` consistently in closures; `build_sweep_tasks` returns `anyhow::Error` (consistent with binary crate convention per CLAUDE.md)
- Dead code or unused imports? No
- New public API: tests present? No — the binary example has no tests. The `build_chain` DOS task is a partial implementation (no setup/collect closures) with a comment noting this.
- Change appears within plan scope? Yes — TASK-5

**Notes:**
- `build_chain` creates a DOS task with no setup or collect closures. The comment explains this is sufficient for dry-run validation. This is a reasonable stub.
- The "single" mode passes `"default"` as the second parameter string to `build_one_task`, which appends `_default` to the task ID. This means single-mode task IDs change format from `scf_U3.0` to `scf_U3.0_default`. This is a behavioral change that existing workflow state files would not recognize.
- `parse_second_values` is a simple split+trim, consistent with the existing `parse_u_values` pattern but without f64 conversion.
- Duplicated `second_values` extraction logic in both "product" and "pairwise" arms could be extracted into a local binding before the match, but the duplication is minimal (4 lines each) and the match arms have different iteration patterns.

---

## File: workflow_core/tests/collect_failure_policy.rs

**Intent:** New integration test file verifying both `FailTask` and `WarnOnly` policies in `process_finished`.

**Checklist:**
- Unnecessary clone/unwrap/expect? `tempfile::tempdir().unwrap()` and `.unwrap()` on `add_task` are standard test patterns.
- Error handling: Test doubles (`StubRunner`, `StubHandle`, `StubHookExecutor`) are correct and complete.
- Dead code or unused imports? No
- New public API: tests present? Yes — this is the test file itself
- Change appears within plan scope? Yes — TASK-3

**Notes:**
- Two tests cover the two policy modes. Both use the same pattern: create workflow, add task with failing collect, run, verify state.
- `StubHandle::wait` takes ownership of the child via `.take()`, which is correct — ensures `wait()` is called at most once.
- File ends without trailing newline (same issue as `prelude.rs`).

---

## File: workflow_core/tests/hook_recording.rs

**Intent:** Added explicit `.collect_failure_policy(CollectFailurePolicy::WarnOnly)` to the `collect_failure_does_not_fail_task` test, and imported `CollectFailurePolicy`.

**Checklist:**
- Unnecessary clone/unwrap/expect? No
- Error handling: N/A
- Dead code or unused imports? No
- New public API: tests present? N/A — existing test updated
- Change appears within plan scope? Yes — TASK-3

**Notes:** This change is necessary because the default `CollectFailurePolicy` is now `FailTask`. Without the explicit `WarnOnly`, the test would fail (task would be marked Failed instead of Completed). This is correct behavior — the test's intent is to verify `WarnOnly` semantics.

---

## File: Cargo.toml

**Intent:** Added `itertools = "0.14"` to workspace dependencies.

**Checklist:**
- No issues

---

## File: examples/hubbard_u_sweep_slurm/Cargo.toml

**Intent:** Added `itertools` workspace dependency. Removed trailing newline.

**Checklist:**
- Trailing newline missing — minor code hygiene issue.

---

## File: workflow_core/src/prelude.rs

**Intent:** Re-exported `CollectFailurePolicy`.

**Checklist:**
- File missing trailing newline (already noted above).

---
