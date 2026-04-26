## Plan Review Decisions — phase-6-fix — 2026-04-26

### Design Assessment

The plan is sound. All three bugs (string-parsed flags with no format hints, default mode generating only SCF, duplicate DOS task IDs) are correctly identified and eliminated by the two-binary rewrite. The plan respects the project's three-layer architecture and existing example patterns. Five gaps were identified: unspecified kpoint/cutoff parser error handling, vague KpointsMpGrid-to-CellDocument merging steps, missing DOS collect closure, unhandled edge case in pairwise mode with optional parameters, and raw OS errors from `.check` file copy. All five are addressed via plan amendments.

### Deferred Item Decisions

#### D.1: Restore plan-specified portable config fields
**Decision:** Defer again
**Rationale:** Precondition (second user or non-NixOS cluster) is not met. Adding portable config fields would be a functional redesign of the SLURM interface, out of scope for this bug-fix plan.
**Updated precondition:** When the `scf_dos_chain` or `multi_param_sweep` example is shared with a user on a non-NixOS cluster, or Tony migrates away from NixOS.

#### D.2: `generate_job_script` formatting inconsistencies
**Decision:** Absorb
**Rationale:** `job_script.rs` is being freshly written for both new binaries. Formatting should be fixed at creation time rather than carried forward.
**Action:** Plan amended — step 5 of setup closure now reads: "**Formatting fix (from D.2)**: use a clean heredoc template — no literal tab characters mixed with spaces, consistent quoting around SBATCH directives."

#### D.3 (partial): Unit tests for `generate_job_script`
**Decision:** Defer again
**Rationale:** Meaningful tests require a second template variant (D.1). With the job script remaining NixOS-specific, tests would assert brittle NixOS-specific strings.
**Updated precondition:** When D.1 (portable config fields) is implemented — a second template variant makes test assertions meaningful.

#### Improve: `read_task_ids` empty-string edge case
**Decision:** Defer again
**Rationale:** This plan touches only the workspace `Cargo.toml` and the `examples/` directory — no edits to `workflow-cli/src/main.rs`.
**Updated precondition:** Next edit to `workflow-cli/src/main.rs` for any functional purpose.

### Plan Amendments Applied

1. Specify kpoint parser: `parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>` with error handling for malformed input
2. Specify cutoff parser: `parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>`
3. Clarify KpointsMpGrid merging: concrete API sequence (`.to_cell_file()` → push `kpoints.to_cell()` → `to_string_many_spaced()`)
4. Add DOS collect closure: verify `ZnO_DOS.castep` exists with "Total time" completion marker
5. Specify pairwise-mode constraint: error if `--kpoints` or `--cutoffs` is `None` in pairwise mode
6. Absorb D.2: clean heredoc template in new `job_script.rs` — no literal tabs, consistent quoting
7. Use `workspace = true` for workspace-managed deps (anyhow, clap, itertools) in both new binaries
8. Verified: `examples/hubbard_u_sweep/` remains unchanged (no action needed)
