# Deferred Improvements and Architectural Patterns -- Phase 6 Fix

> Synthesised from:
> - `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`
> - `notes/pr-reviews/phase-6/deferred.md`
> - `notes/plan-enrichment/phase-6-fix/deferred-and-patterns.md` (previous enrichment)
>
> Generated 2026-05-05 during phase-6-fix plan elaboration.

---

## Deferred Improvements

Items carried forward from prior review rounds. Each has a precondition that
must be met before it is actionable.

### D.1: Restore plan-specified portable config fields

- **Source:** Phase 5A review
- **Problem:** The old `hubbard_u_sweep_slurm` example uses NixOS-specific config
  fields (`nix_flake`, `mpi_if`, `--nodelist=nixos`) instead of the
  plan-specified portable fields (`account`, `walltime`, `modules`,
  `castep_command`). Reduces the example's value as a reference for non-NixOS
  clusters.
- **Precondition:** A second user adopts the examples, or Tony moves to a
  non-NixOS cluster.
- **Action:** When triggered, migrate both `multi_param_sweep` and
  `scf_dos_chain` to use portable SLURM fields.

### D.2: `generate_job_script` formatting inconsistencies

- **Source:** Phase 5A review
- **Problem:** `job_script.rs` uses a literal `\t` character among spaces for
  the `--map-by` flag. SBATCH directives have inconsistent quoting.
- **Precondition:** Next functional edit to `job_script.rs`.
- **Action:** Use a clean heredoc template (or `indoc!` macro) with consistent
  quoting -- no literal tab characters mixed with spaces. The existing
  `no_literal_tabs` test from the old binary should be ported and pass.

### D.3 (partial): Unit tests for `generate_job_script`

- **Source:** Phase 5A review
- **Problem:** `parse_u_values` tests are comprehensive (done in Phase 5B).
  `generate_job_script` tests are tightly coupled to NixOS-specific output,
  making assertions brittle without a second template variant.
- **Precondition:** D.1 must be addressed first -- a portable template variant
  makes test assertions meaningful.
- **Action:** Add unit tests for the portable `generate_job_script` after D.1.

### `read_task_ids` empty-string edge case

- **Source:** Round 2 review
- **Problem:** `read_task_ids` returns `Ok([""])` when called with `vec![""]`,
  producing a downstream error rather than a clear diagnostic. Not reachable
  through normal CLI usage (clap's `default_value = "-"` ensures a dash), but a
  future maintainer could be confused by the gap between the documented
  stdin-fallback description and the actual single-condition check.
- **Precondition:** Actual need to handle a different edge case in
  `read_task_ids` -- not worth a standalone change.
- **Action:** Tighten the check to reject empty strings explicitly with a clear
  error message.

---

## Key Architectural Patterns

Patterns established or reinforced by this fix plan. Implementation should
follow these consistently.

### 1. Purpose-specific binaries over mode-flags

The old `hubbard_u_sweep_slurm` binary had three operation modes (single,
product, pairwise) controlled by a flag, plus implicit chain-building in
product/pairwise modes. This caused three bugs:

- String-parsed "second values" with no CLI format hints
- Default sweep mode "single" silently producing only SCF tasks
- Product/pairwise modes building SCF->DOS chains with colliding task IDs

**Fix pattern:** Decompose into two purpose-built binaries, each owning exactly
one workflow:

| Binary | Workflow | Tasks |
|--------|----------|-------|
| `multi_param_sweep` | Independent SCF parameter sweep | All SCF, no children |
| `scf_dos_chain` | SCF followed by DOS (chain) | 2 tasks, 1 dependency |

**Rule:** If a binary needs a flag to enable or disable a fundamentally
different kind of workflow, that workflow probably belongs in its own binary.

### 2. Parameter encoding in task IDs

The old binary generated `dos_kpt8x8x8` as a task ID without encoding the
Hubbard U value. Sweeping U in chain mode gave duplicate IDs.

**Pattern:** Encode every distinguishing parameter into the task ID:

```
scf_U3.0_k8x8x8_c500  # multi_param_sweep: all three params encoded
scf                      # scf_dos_chain: single parameter set, no collision risk
dos                      # scf_dos_chain: single parameter set, no collision risk
```

**Rule:** A task ID must differentiate every parameter that varies in scope.

### 3. Parse-time validation gates with clear diagnostics

The plan defines two parsing utility functions with strict validation:

- `parse_kpoints`: comma-separated, each segment must have exactly 3 axes
  separated by `x`. `"8xx8"`, `"abc"`, `"8x8"` all produce clear `anyhow!`
  errors. Empty input returns an explicit "kpoints list is empty" error.
- `parse_cutoffs`: comma-separated f64 list, same pattern as `parse_u_values`.

Pairwise mode requires both `--kpoints` and `--cutoffs` to be explicitly
provided, with a clear error if not.

**Pattern:** Validate at the boundary, not at use-site. Provide the exact
expected format in the error message.

**Rule:** Every parsing function should produce `anyhow::Result<T>` with a
message that tells the user what format was expected and what went wrong.

### 4. Explicit defaults over silent modes

The old binary defaulted `--sweep-mode` to `"single"`, which silently produced
only SCF tasks. The new plan defaults to `"product"` -- the most useful
full-sweep mode. If a user wants a subset, they choose `"pairwise"` explicitly.

- `--u-values` gets a generous default (`"0.0,1.0,2.0,3.0,4.0,5.0"`) so a user
  gets useful output from a bare invocation.
- `--kpoints` and `--cutoffs` are `Option<String>`, defaulting to `None`. When
  absent, seed file defaults are used -- this keeps the binary usable for
  simple U-only sweeps.

**Rule:** Defaults should do something useful and visible, not silently
restrict behaviour.

### 5. In-memory document mutation (not string manipulation)

All CASTEP file modifications follow parse-mutate-serialize:

1. Parse seed file via `castep_cell_fmt::parse::<CellDocument>(&input)` or
   `castep_cell_fmt::parse::<ParamDocument>(&input)`
2. Mutate typed fields directly (e.g., `doc.kpoints_mp_grid =
   Some(KpointsMpGrid(...))`, `doc.basis_set.cutoff_energy =
   Some(CutOffEnergy { ... })`)
3. Serialize via `doc.to_cell_file()` and `to_string_many_spaced()`

**Rule:** Never construct or modify CASTEP input files through string
concatenation or regex replacement. Always go through the typed document API.

### 6. Shared workdir for chained tasks

The `scf_dos_chain` binary places both the SCF and DOS task in the same
workdir (`runs/scf_dos/`). This makes checkpoint file access straightforward:

- SCF produces `ZnO.check` in the workdir
- DOS setup copies `<workdir>/ZnO.check` to `<workdir>/ZnO_DOS.check` by
  simple filename, no path traversal needed

**Rule:** Dependent tasks that share files (checkpoints, wavefunctions) should
share a workdir. Only split workdirs when tasks are entirely independent.

### 7. Clean heredoc template for job scripts

The plan mandates proper heredoc templates in `generate_job_script`:

- No literal tab characters mixed with spaces
- Consistent quoting around SBATCH directives
- Ported `no_literal_tabs` test must pass

**Rule:** Any file generation template must use consistent indentation
(either all spaces or all tabs, never mixed) and consistent quoting.

---

## Known Failure Modes

Failure modes identified from prior fix rounds (Phase 4, Phase 5, Phase 5b)
that should be consciously avoided during phase-6-fix implementation.

### F.1: Missing `pub use` re-exports

- **Source:** Phase 4 TASK-2
- **Pattern:** A type is implemented in a submodule but omitted from the `pub
  use` line in the crate root, making it inaccessible at the crate root despite
  being available from the submodule.
- **Mitigation:** After adding any new public type or function to a submodule,
  verify it appears in the crate's `lib.rs` re-export section.

### F.2: Incomplete consumer updates

- **Source:** Phase 5 TASK-1, TASK-2
- **Pattern:** After introducing a named constant (e.g., `JOB_SCRIPT_NAME`),
  some consumers still use the hardcoded string (`"job.sh"`). A rename or
  abstraction change in the library is not fully propagated until every
  consumer is updated.
- **Mitigation:** Search all crate consumers (including example binaries and
  test files) for both the old and new forms after any rename. Use `rg` to
  verify zero occurrences of the old form remain in non-library code.

### F.3: Stale imports after refactoring

- **Source:** Phase 4 TASK-3
- **Pattern:** Moving a function from CLI to library leaves behind the original
  definition plus duplicate tests in the CLI. The refactoring is correct in the
  library but the dead code persists in the consumer.
- **Mitigation:** After moving or extracting a function, delete the original
  definition immediately. Run `cargo clippy --all-targets` with
  `-W dead-code` to detect survivors.

### F.4: Dead / unenforced API surface

- **Source:** Phase 4 TASK-1
- **Pattern:** A method like `TaskSuccessors::inner()` exposes the backing
  type (`HashMap`), defeating a newtype abstraction, and has zero callers.
  Without a dead-code lint, such methods accumulate silently.
- **Mitigation:** Do not add getter methods that expose internal representation
  unless a concrete consumer exists. Run dead-code detection as part of the
  implementation loop.

### F.5: Stale documentation

- **Source:** Phase 5b TASK-1
- **Pattern:** `ARCHITECTURE.md` contains outdated struct field names
  (`execution_mode` vs `mode`, `dependencies` vs `depends_on`), outdated trait
  signatures (pre-refactor `StateStore`), and outdated error types.
  Documentation drifts silently when not treated as compilable code.
- **Mitigation:** Update any developer-facing documentation in the same
  commit that changes the code it describes. Review the plan's verification
  steps for documentation checks.

### F.6: Overloaded binary syndrome (new, from phase-6-fix)

- **Source:** The old `hubbard_u_sweep_slurm` binary
- **Pattern:** A single binary uses a mode flag to enable fundamentally
  different workflows (independent tasks vs. chained tasks). This causes
  hidden-mode confusion, duplicate task IDs, and untested flag combinations.
- **Mitigation:** If a binary needs a flag to toggle between "has dependent
  tasks" and "does not have dependent tasks", split into separate binaries.
  See Pattern 1.

### F.7: Default-driven silent behaviour (new, from phase-6-fix)

- **Source:** Old binary defaulted `--sweep-mode` to `"single"` without
  indicating this to the user.
- **Pattern:** A default value that silently restricts functionality, causing
  first-time users to get incomplete results without warning.
- **Mitigation:** Choose defaults that produce the most complete and useful
  behaviour. If a restricted mode exists, require explicit opt-in. See Pattern
  4.

### F.8: String-format coupling without documentation (new, from phase-6-fix)

- **Source:** Old binary accepted comma-separated values without printing
  expected format in help text or error messages.
- **Pattern:** A CLI argument expects structured string input but does not
  document the expected format or validate it at the boundary, leaving users
  to discover format requirements through trial and error.
- **Mitigation:** Every CLI string argument that carries structured data must
  document its expected format in the help text (if `clap` does not already
  handle it) and produce `anyhow::Result<T>` with the expected format in the
  error message. See Pattern 3.
