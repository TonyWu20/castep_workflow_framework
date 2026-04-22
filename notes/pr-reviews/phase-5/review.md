# Phase 5A PR Review

## PR Review: `phase-5` -> `main`

**Rating:** Request Changes

**Summary:** Phase 5A delivers a functional SLURM sweep example and fixes the critical `sbatch` script-path bug in `queued.rs`. The implementation diverges from the plan's `config.rs` and `job_script.rs` specifications in ways that are understandable adaptations to Tony's NixOS cluster but leave the code non-portable (plan fields like `account`, `walltime`, `modules`, `castep_command` are missing; NixOS-specific fields like `nix_flake` and `mpi_if` replace them). One correctness issue in `queued.rs` (hardcoded script filename) creates a hidden API contract, and `parse_u_values` silently swallows bad input. Both need fixes before merge.

**Cross-Round Patterns:** None

**Deferred Improvements:** 5 items -> notes/pr-reviews/phase-5/deferred.md

**Axis Scores:**

- Plan & Spec: Partial -- `config.rs` and `job_script.rs` diverge substantially from the plan (fields removed/replaced); the FRICTION-1 fix changes *log paths* instead of *script path* canonicalization
- Architecture: Pass -- layer boundaries respected; `anyhow` only in binary; `WorkflowError` used in closures with appropriate comment about `InvalidConfig` reuse
- Rust Style: Pass -- clean `cargo check` and `cargo clippy -D warnings`; pedantic clippy finds only `uninlined_format_args` and `doc_markdown`
- Test Coverage: Partial -- no unit tests for new public functions (`parse_u_values`, `generate_job_script`); no test for `submit()` log-path absolutization logic

---

## Fix Document for Author

### Issue 1: Hardcoded `"job.sh"` filename in `QueuedRunner::submit()` creates invisible contract

**Classification:** Correctness
**File:** `workflow_utils/src/queued.rs`
**Severity:** Major
**Problem:** The original code computed `script_path = workdir.join("job.sh")` and passed it to `sbatch`. The fix replaced this with a hardcoded string literal `"job.sh"`. While this works because `.current_dir(workdir)` is set, it creates an invisible contract: every consumer of `QueuedSubmitter` must name their script `job.sh`. The previous code at least made the path a variable. More importantly, this is a library crate (`workflow_utils`) -- hardcoding a filename here couples all downstream consumers to a convention that is not enforced by the type system or documented in the trait's contract. If someone names their script `run.sh`, they get a confusing `sbatch` error with no indication that the framework requires `job.sh`.

**Fix:** Accept the script filename as a parameter, or make it a const with documentation. The simplest backward-compatible fix:

```rust
// In QueuedRunner or QueuedSubmitter trait:
/// The expected job script filename within the task workdir.
pub const JOB_SCRIPT_NAME: &str = "job.sh";
```

Then use `JOB_SCRIPT_NAME` in `submit()` and export it so consumers can reference the same constant. Alternatively, add a `script_name: &str` parameter to `submit()` (this would require a trait signature change, which may be better deferred to 5B if it touches the trait).

For now, the minimal fix is: define the constant and document it.

### Issue 2: `parse_u_values()` silently drops unparseable values

**Classification:** Correctness
**File:** `examples/hubbard_u_sweep_slurm/src/config.rs`
**Severity:** Minor
**Problem:** `filter_map(|s| s.trim().parse::<f64>().ok())` silently swallows any non-numeric token. If a user passes `--u-values "0.0,1.0,oops,3.0"`, the `oops` entry is silently ignored and only three U values are swept. For a computational chemistry workflow where each job may cost real compute hours, silently dropping a parameter is worse than crashing. The user would not notice the missing data point until they inspect results.

**Fix:** Use `map` + `collect::<Result<Vec<_>, _>>()` and surface the parse error:

```rust
pub fn parse_u_values(&self) -> Result<Vec<f64>, String> {
    self.u_values
        .split(',')
        .map(|s| {
            let trimmed = s.trim();
            trimmed.parse::<f64>().map_err(|e| {
                format!("invalid U value '{}': {}", trimmed, e)
            })
        })
        .collect()
}
```

Then propagate the error in `main.rs` (it already uses `anyhow::Result`, so `.context("parsing --u-values")?` suffices).

---

## Deferred Improvements

### D.1: Restore plan-specified portable config fields

**Rationale:** The plan specifies `account`, `walltime`, `modules`, and `castep_command` as portable SLURM config fields. The implementation replaced them with NixOS-specific fields (`nix_flake`, `mpi_if`, `--nodelist=nixos`). While the NixOS adaptation was necessary for Tony's cluster, the example's value as a reference for other users (or other clusters) is reduced. Consider either parameterizing the job script template (NixOS vs module-based) or keeping both field sets with feature flags / subcommands.
**Candidate for:** Phase 5B
**Precondition:** When the example is intended to be used on a non-NixOS cluster, or when a second user tries to adopt it.

### D.2: `generate_job_script` uses tab character and inconsistent indentation

**Rationale:** Line 20 of `job_script.rs` uses a literal `\t` character among spaces for the `--map-by` flag. The SBATCH directives also have inconsistent quoting (job-name is quoted, partition is not). While this does not affect sbatch parsing, it makes the generated script harder to read and debug when a user inspects `job.sh` in the workdir. A heredoc-style template or `indoc!` macro would be cleaner.
**Candidate for:** Phase 5B
**Precondition:** When `job_script.rs` is next modified for any reason.

### D.3: Unit tests for `parse_u_values` and `generate_job_script`

**Rationale:** Both are pure functions with clear inputs and outputs -- ideal unit test targets. `parse_u_values` has edge cases (empty string, trailing comma, negative values, whitespace). `generate_job_script` should verify that all SBATCH directives appear in the output. These tests would have caught the silent-drop behavior in Issue 2.
**Candidate for:** Phase 5B (or as part of fixing Issue 2)
**Precondition:** When Issue 2 is fixed, add tests alongside.

### D.4: `submit()` log-path absolutization should use `dunce::canonicalize` or `std::path::absolute`

**Rationale:** `cwd.join(log_dir)` does not resolve `..` or symlinks. If `log_dir` is `"../logs"`, the resulting path is `"/abs/path/to/cwd/../logs"` rather than the clean `"/abs/path/to/logs"`. On most systems this works, but it produces ugly paths in SLURM output directives and state files. `std::path::absolute` (stabilized in Rust 1.79) or `dunce::canonicalize` would produce cleaner results.
**Candidate for:** Phase 5B
**Precondition:** When `submit()` is next touched, or if a bug report involves symlinked log directories.

### D.5: Pedantic clippy findings (`uninlined_format_args`, `doc_markdown`)

**Rationale:** Running `clippy::pedantic` on the `hubbard_u_sweep_slurm` package produces 8 `uninlined_format_args` warnings and 1 `doc_markdown` warning. These are style-only and do not affect correctness, but cleaning them up would bring the example in line with modern Rust format string idioms (e.g., `format!("{task_id}")` instead of `format!("{}", task_id)`).
**Candidate for:** Phase 5B or any touch to the affected files
**Precondition:** Next edit to `config.rs` or `main.rs`.
