# Phase 5A Deferred Improvements

Items identified during PR review v1, classified as `[Improvement]` -- better design but outside Phase 5A plan scope.

## D.1: Restore plan-specified portable config fields

**Rationale:** The plan specifies `account`, `walltime`, `modules`, and `castep_command` as portable SLURM config fields. The implementation replaced them with NixOS-specific fields (`nix_flake`, `mpi_if`, `--nodelist=nixos`). While the NixOS adaptation was necessary for Tony's cluster, the example's value as a reference for other users (or other clusters) is reduced. Consider either parameterizing the job script template (NixOS vs module-based) or keeping both field sets.

**Candidate for:** Phase 5B

**Precondition:** When the example is intended to be used on a non-NixOS cluster, or when a second user tries to adopt it.

## D.2: `generate_job_script` formatting inconsistencies

**Rationale:** Line 20 of `job_script.rs` uses a literal `\t` character among spaces for the `--map-by` flag. The SBATCH directives also have inconsistent quoting (job-name is quoted, partition is not). While this does not affect sbatch parsing, it makes the generated script harder to read and debug. A heredoc-style template or `indoc!` macro would be cleaner.

**Candidate for:** Phase 5B

**Precondition:** When `job_script.rs` is next modified for any reason.

## D.3: Unit tests for `parse_u_values` and `generate_job_script`

**Rationale:** Both are pure functions with clear inputs and outputs -- ideal unit test targets. `parse_u_values` has edge cases (empty string, trailing comma, negative values, whitespace). `generate_job_script` should verify that all SBATCH directives appear in the output.

**Candidate for:** Phase 5B (or as part of fixing Issue 2 from fix document)

**Precondition:** When Issue 2 is fixed, add tests alongside.

## D.4: `submit()` log-path absolutization should use `std::path::absolute`

**Rationale:** `cwd.join(log_dir)` does not resolve `..` or symlinks. `std::path::absolute` (stabilized in Rust 1.79) would produce cleaner results and handle edge cases like `"../logs"`.

**Candidate for:** Phase 5B

**Precondition:** When `submit()` is next touched, or if a bug report involves symlinked log directories.

## D.5: Pedantic clippy findings (`uninlined_format_args`, `doc_markdown`)

**Rationale:** 8 `uninlined_format_args` warnings and 1 `doc_markdown` warning from `clippy::pedantic`. Style-only, does not affect correctness.

**Candidate for:** Phase 5B or any touch to the affected files

**Precondition:** Next edit to `config.rs` or `main.rs`.

## D.6: `--workdir` / `--output-dir` flag for invocation-location independence (FRICTION-2)

**Rationale:** The binary must currently be invoked from the directory where `runs/`, `logs/`, and the state file should be created — all those paths are hardcoded as relative strings in `main.rs`. HPC submission scripts frequently run binaries from a different directory. A `--workdir` flag would remove this constraint.

**Candidate for:** Phase 5B

**Precondition:** Already identified in Phase 5A FRICTION-2; no additional trigger needed. This is the most user-visible ergonomic gap from the production run.

## D.7: `squeue` empty-output treated as job success — false-positive risk

**Rationale:** When `squeue -j <id> -h` returns empty output (job no longer in queue), `is_running()` sets `finished_exit_code = Some(0)` (assumed success). Tony observed this during Phase 5A: a job that failed due to filesystem inaccessibility was marked Completed. The collect closure guards against this by checking for "Total time" in the output file, but only if the workflow engine actually runs collect after a 0 exit code. This wiring should be audited to confirm collect is never skipped on exit 0 from a queued job.

**Candidate for:** Phase 5B or Phase 6

**Precondition:** A second false-success case is observed, or the collect-vs-exit-code wiring in `workflow.rs` is confirmed to be bypassable.

---

## Round 2 items (2026-04-23)

## D.8: Double `s.trim()` call in `parse_u_values`

**Source:** Round 2 review
**Rationale:** In `config.rs`, the `parse_u_values` closure calls `s.trim()` once for `parse::<f64>()` and again in the `map_err` format string. Extracting to `let trimmed = s.trim();` would eliminate the redundant call and improve readability.
**Candidate for:** Phase 5B or any touch to `config.rs`
**Precondition:** Next edit to `config.rs`

## D.9: `anyhow::anyhow!(e)` vs `anyhow::Error::msg(e)` at `parse_u_values` call site

**Source:** Round 2 review
**Rationale:** `main.rs` uses `.map_err(|e| anyhow::anyhow!(e))` to convert a `String` error. The `anyhow!` macro is intended for format strings; the idiomatic form for wrapping an existing `Display` value is `.map_err(anyhow::Error::msg)`. Style-only, no correctness impact.
**Candidate for:** Phase 5B or any touch to `main.rs`
**Precondition:** Next edit to `main.rs`
