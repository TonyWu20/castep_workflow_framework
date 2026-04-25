# Draft PR Review: `phase-6` -> `main`

**Rating:** Request Changes

**Summary:** Phase 6 implements all five plan goals correctly. The CollectFailurePolicy fix and root_dir support are solid. The multi-parameter sweep example is functional but introduces a behavioral change in single-mode task IDs that warrants documentation or a fix. The per-file analysis document contains factual inaccuracies regarding trailing newlines that should be corrected.

**Axis Scores:**
- Plan & Spec: Pass — All 5 goals (CollectFailurePolicy, root_dir, stdin, multi-param sweep, docs sweep) are implemented as commissioned.
- Architecture: Pass — DAG-centric design preserved, builder patterns correct, crate boundaries respected, sync-by-default with tokio-ready design.
- Rust Style: Partial — Dead code branch in `read_task_ids`, single-mode task ID behavioral change, one file missing trailing newline.
- Test Coverage: Pass — Integration tests for both collect policies, updated hook_recording test, new unit tests for `read_task_ids`.

---

## Issues Found

- [Correctness] Dead code: `task_ids.is_empty()` branch unreachable — file: workflow-cli/src/main.rs:32 — The `#[arg(required = false, default_value = "-")]` attribute ensures `task_ids` always has at least one element. The `task_ids.is_empty()` branch on line 32 can never execute. Remove the dead branch or remove the clap default and handle the empty case properly.

- [Improvement] Single-mode task ID behavioral change — file: examples/hubbard_u_sweep_slurm/src/main.rs:180 — Single-mode now appends `_default` to task IDs (e.g., `scf_U3.0` becomes `scf_U3.0_default`). This is a behavioral change that existing workflow state files would not recognize. Document this or use a different sentinel value (e.g., empty string that does not produce a suffix).

- [Improvement] Missing trailing newline — file: examples/hubbard_u_sweep_slurm/Cargo.toml — File ends without trailing newline. CLAUDE.md rule requires trailing newlines on all source files.

- [Improvement] Per-file analysis factual inaccuracies — file: notes/pr-reviews/phase-6/per-file-analysis.md — The analysis claims `workflow_core/src/prelude.rs` and `workflow_core/tests/collect_failure_policy.rs` are missing trailing newlines. Both files were verified via hex dump to have trailing newlines (`0a` at end). These false claims should be removed from the analysis.

---

## Notes

### Strengths
- `process_finished()` rewrite (workflow.rs:389-457) is the most complex change and is well-structured. The collect-before-status-decision ordering is correct, and the state re-read pattern after `mark_failed` handles the collect-overrides-exit-code case properly.
- `InFlightTask::workdir` holding the resolved path (not the original) means hooks and collect closures see the correct path. This is the intended behavior.
- Integration test stubs (`StubRunner`, `StubHandle`, `StubHookExecutor`) in both `collect_failure_policy.rs` and `workflow.rs` follow consistent patterns and are well-implemented.
- `StubHandle::wait` taking ownership via `.take()` ensures `wait()` is called at most once.

### Observations
- `examples/hubbard_u_sweep_slurm/src/main.rs:119-133`: The `build_chain` DOS task is a functional stub (no setup/collect closures). This is acceptable per the plan scope (dry-run validation), but the comment noting this is sufficient.
- `examples/hubbard_u_sweep_slurm/src/main.rs:150-173`: Minor duplication of `second_values` extraction in both match arms (4 lines each). The match arms have different iteration patterns, so extraction is marginal.
- `workflow_core/src/lib.rs:17-31`: The `init_default_logging` function returns `Box<dyn Error>` with a documented reason in a comment. This is a justified exception from the "anyhow only in binaries" convention per CLAUDE.md.
