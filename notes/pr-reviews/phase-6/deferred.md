## Deferred Improvements: `phase-6` — 2026-04-25

Items carried forward from prior phases after plan-review decisions. All other prior deferred items were closed (already fixed, already codified, or subsumed by Phase 6 goals).

---

### D.1: Restore plan-specified portable config fields

**Source:** Phase 5A review
**Rationale:** The `hubbard_u_sweep_slurm` example uses NixOS-specific config fields (`nix_flake`, `mpi_if`, `--nodelist=nixos`) instead of the plan-specified portable fields (`account`, `walltime`, `modules`, `castep_command`). The example's value as a reference for non-NixOS clusters is reduced.
**Candidate for:** When a second user attempts to adopt the example, or Tony moves to a non-NixOS cluster.
**Precondition:** Second user or non-NixOS cluster required — no earlier.

---

### D.2: `generate_job_script` formatting inconsistencies

**Source:** Phase 5A review
**Rationale:** `job_script.rs` line 20 uses a literal `\t` character among spaces for the `--map-by` flag. SBATCH directives have inconsistent quoting. A heredoc-style template or `indoc!` macro would be cleaner.
**Candidate for:** Next functional edit to `job_script.rs`.
**Precondition:** Next edit to `job_script.rs` for functional reasons — fix formatting in the same pass.

---

### D.3 (partial): Unit tests for `generate_job_script`

**Source:** Phase 5A review
**Rationale:** `parse_u_values` tests are comprehensive (done in Phase 5B). `generate_job_script` tests are tightly coupled to NixOS-specific output, making assertions brittle without a second template variant. Only worthwhile once D.1 (portable template) is addressed.
**Candidate for:** When D.1 is resolved and a portable job script template exists.
**Precondition:** D.1 must be addressed first — a second template variant makes test assertions meaningful.
