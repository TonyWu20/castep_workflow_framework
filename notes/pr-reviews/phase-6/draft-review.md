# Draft PR Review: `phase-6` -> `main`

**Rating:** Approve

**Summary:** Phase 6 implements all five plan goals — CollectFailurePolicy, root_dir, retry stdin, multi-parameter sweeps, and documentation accuracy. The `process_finished()` rewrite fixing the D.7 production correctness bug is the most significant change and is correct. No blocking issues remain after prior fix rounds.

## Axis Scores

- **Plan & Spec: Pass** — All five goals (TASK-1 through TASK-6) implemented as commissioned. Integration tests cover both CollectFailurePolicy modes.
- **Architecture: Pass** — DAG-centric design preserved. Builder patterns correct. Crate boundaries respected. `CollectFailurePolicy` stays in `workflow_core`, sweep logic stays in Layer 3. `root_dir` resolution only on relative paths, preserving `dry_run()` output.
- **Rust Style: Pass** — No unnecessary clone/unwrap/expect. Error handling consistent. Builder pattern used throughout. No dead code.
- **Test Coverage: Pass** — Two integration tests for CollectFailurePolicy modes. Updated `hook_recording` test with explicit `WarnOnly`. Unit tests for `read_task_ids`. `parse_empty_string` assertion strengthened.

## Issues Found

### [Improvement] `read_task_ids` logic gap: empty input returns silent no-op

- **File:** `workflow-cli/src/main.rs`, `read_task_ids` (lines 31-55)

The function's docstring describes three behaviors:

```
- Non-empty `task_ids` with first element != "-" → use as-is
- `["-"]` or empty + piped input → read stdin (one ID per line)
- Empty + TTY → usage error
```

The second and third bullets are NOT implemented. The function has no branch for the empty case. The only condition is:

```rust
if task_ids.first().map(|s| s.as_str()) == Some("-")
```

If a user passes `--task-ids ""` (clap produces `vec![""]`), the function returns `Ok([""])` and the downstream `cmd_retry` silently iterates once over the empty string. This is not a crash, but it produces misleading behavior — no error is surfaced to the user.

The current test coverage does not exercise this path. The `default_value = "-"` in clap ensures the user-facing path never produces an empty vec, so this is latent, but the docstring's claim about handling stdin on empty input is misleading.

### [Improvement] `read_task_ids`: unreachable `task_ids.is_empty()` branch

- **File:** `workflow-cli/src/main.rs`, `read_task_ids`, line 32

The `task_ids.is_empty()` condition is dead code. The clap attribute `#[arg(required = false, default_value = "-")]` guarantees `task_ids` always has at least one element when the function is called. Remove the dead branch or refactor to a cleaner conditional.

---

## Notes

### Strengths
- `process_finished()` rewrite (workflow.rs:389-457) is the most complex change and is well-structured. The collect-before-status-decision ordering is correct, and the state re-read pattern after `mark_failed` handles the collect-overrides-exit-code case properly.
- `InFlightTask::workdir` holding the resolved path (not the original) means hooks and collect closures see the correct path. This is the intended behavior.
- Integration test stubs (`StubRunner`, `StubHandle`, `StubHookExecutor`) in both `collect_failure_policy.rs` and `workflow.rs` follow consistent patterns and are well-implemented.
- `StubHandle::wait` taking ownership via `.take()` ensures `wait()` is called at most once.
- Single-mode task IDs preserved in original format (e.g., `scf_U3.0` — the `_default` suffix was removed in fix-plan TASK-2).

### Observations
- `examples/hubbard_u_sweep_slurm/src/main.rs:119-133`: The `build_chain` DOS task is a functional stub (no setup/collect closures). This is acceptable per the plan scope (dry-run validation).
- `examples/hubbard_u_sweep_slurm/src/main.rs:150-173`: Minor duplication of `second_values` extraction in both match arms (~4 lines each). The match arms have different iteration patterns, so extraction is marginal.
- `workflow_core/src/lib.rs:17-31`: The `init_default_logging` function returns `Box<dyn Error>` with a documented reason. This is a justified exception from the "anyhow only in binaries" convention per CLAUDE.md.

### Notes on the per-file analysis

The per-file analysis (in `per-file-analysis.md`) contains a factual error in the `prelude.rs` section: the notes line states "Trailing newline present (confirmed via manifest)" — but the manifest explicitly shows `has_trailing_newline: false` for this file. This is a self-contradiction in the review materials that should be corrected.
