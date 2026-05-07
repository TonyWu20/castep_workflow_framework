# Deferred Items — Phase 6 Fix

Items flagged during the make-judgement review as worth doing but out of scope for this phase.

## Previously Documented Deferred Items

These items were carried forward from the planning phase and remain deferred:

- **D.1 (portable SLURM config)**: Make SBATCH directives configurable rather than hardcoded. Preconditions unmet.
- **D.3 (expanded unit tests for generate_job_script)**: Edge case coverage beyond the 6 ported tests. Preconditions unmet.
- **read_task_ids edge case**: Not fixed. Preconditions unmet.

## New Deferred Items (fix round review 2026-05-08)

### TaskClosure consolidation (Medium Priority)
Add `setup_boxed()` / `collect_boxed()` builder methods accepting `TaskClosure` directly; make `setup`/`collect` fields `pub(crate)` on `Task`. Resolves dual-interface inconsistency (direct field mutation vs builder method).

### dead_code cleanup (Low Priority)
Move `default_chain_config` inside `#[cfg(test)]` in `examples/scf_dos_chain/src/config.rs:94`.

### External crate isolation (Low Priority)
`examples/castep-cell-io` is a full git checkout inside the workspace. Consider git submodule or `[patch]` section for cleaner dependency management.

### Regression tests for collect closures (Low Priority)
If example binaries graduate from examples, add targeted tests asserting collect closure path construction behavior.

### castep-cell-io test fixtures gitignored (Info)
Two tests in external `castep-cell-fmt` fail due to missing fixture files. The fixtures are in `.gitignore`, so they don't exist in a fresh checkout. Out of scope for this phase — needs upstream resolution in the castep-cell-io repo.
