## Deferred Improvements

- **D.1: Restore plan-specified portable config fields** — The `hubbard_u_sweep_slurm` example uses NixOS-specific config fields (`nix_flake`, `mpi_if`, `--nodelist=nixos`) instead of the plan-specified portable fields (`account`, `walltime`, `modules`, `castep_command`). Precondition: second user or non-NixOS cluster required.

- **D.2: `generate_job_script` formatting inconsistencies** — `job_script.rs` uses a literal `\t` character among spaces for `--map-by`; SBATCH directives have inconsistent quoting. Cleanup via `indoc!` macro or heredoc-style template deferred until next functional edit to `job_script.rs`.

- **D.3 (partial): Unit tests for `generate_job_script`** — `parse_u_values` tests are complete (done in Phase 5B), but `generate_job_script` tests are tightly coupled to NixOS-specific output. Precondition: D.1 (portable template) must be addressed first so a second template variant makes test assertions meaningful.

- **`read_task_ids` empty-string edge case** — Returns `Ok([""])` when called with `vec![""]`, producing a downstream error rather than a clear diagnostic. Not reachable through normal CLI usage but a future maintainer could be confused by the gap between the documented stdin-fallback description and the actual single-condition check.

## Known Failure Modes

From Phase 4, Phase 5, and Phase 5b fix plans:

- **Missing `pub use` re-exports** — Phase 4 TASK-2: `TaskSuccessors` was implemented in `state.rs` but omitted from the `pub use` line in `workflow_core/src/lib.rs`, making it inaccessible at the crate root despite being available from the submodule.

- **Incomplete consumer updates** — Phase 5 TASK-1 and TASK-2: After introducing the `JOB_SCRIPT_NAME` constant, two locations (the `hubbard_u_sweep_slurm` consumer and `queued_integration.rs` tests) still used the hardcoded string `"job.sh"`. A rename or abstraction change in the library is not fully propagated until every consumer is updated.

- **Stale imports after refactoring** — Phase 4 TASK-3: Moving `downstream_tasks` BFS from the CLI into `workflow_core` as a `TaskSuccessors` method left behind the original function definition and six duplicate unit tests in the CLI. The refactoring was correct in the library but the old code remained in the consumer.

- **Dead / unenforced API surface** — Phase 4 TASK-1: `TaskSuccessors::inner()` exposed the `HashMap` backing type, defeating the newtype abstraction, and was dead code (no callers). Without a dead-code lint or removal discipline, such methods accumulate.

- **Stale documentation** — Phase 5b TASK-1: `ARCHITECTURE.md` contained outdated struct field names (`execution_mode` vs `mode`, `dependencies` vs `depends_on`), outdated trait signatures (pre-refactor `StateStore`), and outdated error types. Documentation drifts silently when not treated as compilable code.
