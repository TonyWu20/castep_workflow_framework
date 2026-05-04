## Plan Review Decisions — PHASE_6_FIX_PLAN — 2026-05-05

### Design Assessment

The plan is structurally sound. Splitting the buggy `hubbard_u_sweep_slurm` into two purpose-built binaries (`multi_param_sweep` for independent SCF sweeps, `scf_dos_chain` for the chained SCF-to-DOS workflow) correctly resolves all three bugs. Task ID encoding (`scf_U{u}_k{k}_c{c}`) eliminates the duplicate ID collision, and the `.check` file copy pattern for DOS chains follows standard CASTEP convention. Crate boundaries are respected: `anyhow` only in binaries, domain logic stays in library code. Two issues were identified and amended: (1) the plan was written against v0.4.0 APIs but `castep-cell-io` is now v0.5.0 — parse API changed, `KpointsMpGrid` is now a direct field, and dependencies must use local path refs; (2) the `default-logging` feature flag was missing from both new `Cargo.toml` files.

### Deferred Item Decisions

#### D.1: Restore plan-specified portable config fields
**Decision:** Defer again
**Rationale:** The current plan is a bug-fix rewrite, not a portability initiative. The NixOS-specific SLURM template remains appropriate for Tony's sole-user context.
**Action:** No plan update needed.

#### D.2: `generate_job_script` formatting inconsistencies
**Decision:** Absorb
**Rationale:** The plan explicitly calls for cleaning up the heredoc template — no literal tab characters, consistent SBATCH quoting — and porting the `no_literal_tabs` test. This is the "next functional edit" the precondition was waiting for.
**Action:** Already addressed in plan Section "Setup closure" step 5.

#### D.3 (partial): Unit tests for `generate_job_script`
**Decision:** Defer again
**Rationale:** Without a second (portable) job script template variant, test assertions remain tightly coupled to NixOS-specific output. D.1 must be addressed first. The plan ports existing tests (including `no_literal_tabs`), which is sufficient.
**Action:** Updated precondition: D.1 must be addressed first.

#### `read_task_ids` empty-string edge case
**Decision:** Defer again
**Rationale:** Lives in `workflow-cli/src/main.rs` — a different crate entirely, unrelated to the example binary rewrite.
**Action:** Updated precondition: next functional edit to `workflow-cli/src/main.rs`.

### Plan Amendments Applied

**Amendment 1** — Updated all `castep-cell-io` references to v0.5.0 API:
- Parse calls: `castep_cell_fmt::parse::<CellDocument>(&input)` instead of `CellDocument::parse()`
- `KpointsMpGrid` is a direct field on `CellDocument` (no serialization workaround)
- Dependencies use local path dep: `castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }`
- Removed the "issue to be filed" note about missing KpointsMpGrid field
- Bumped API version reference from v0.4.0 to v0.5.0 throughout
- Added note that kept `hubbard_u_sweep` also needs Cargo.toml and main.rs updates

**Amendment 2** — Added `features = ["default-logging"]` to both new `Cargo.toml` files

**Amendment 3** — Added empty-string handling to `parse_kpoints` spec

**Amendment 4** — Added lockfile note to build verification step
