# Task Checklist — Phase 6 Fix Plan

Generated: 2026-04-26
Plan source: `notes/plan-enrichment/phase-6-fix/draft-plan.toml`
Branch: `phase-6-fix`

---

## TASK-1: Create multi_param_sweep binary source files (Cargo.toml, config.rs, job_script.rs, main.rs)

**Goal**: Clear. Create four source files for a new binary crate `multi_param_sweep`.

**Target files**: Explicitly specified with full file paths.

**Implementation detail**: Complete. Full file contents provided as `after` blocks.

**New types/interfaces**: Fully defined inline.

**Module wiring check** (binary crate — no parent `pub mod` needed):
- `pub mod` in parent: Not applicable (binary crate has no parent mod)
- `pub use` re-export: Not applicable
- Consumer updates co-located: Yes — `main.rs` declares `mod config; mod job_script;`, `config.rs` defines `pub struct SweepConfig`, `job_script.rs` imports via `use crate::config::SweepConfig;`, `main.rs` imports via `use config::SweepConfig;` and `use job_script::generate_job_script;`

**Known failure mode check**:
- Missing `pub mod` risk: Low — both `mod config;` and `mod job_script;` are declared in `main.rs` as written in the draft
- Missing `pub use` risk: Low — no `pub use` required for a binary crate
- Stale import risk: Medium — the `main.rs` references `seed_cell` and `seed_param` via `include_str!("../seeds/ZnO.cell")` and `include_str!("../seeds/ZnO.param")`, but those seed files are created by TASK-2. This is acceptable because the dependency chain (TASK-5 depends on TASK-1 through TASK-4) ensures seeds exist before workspace compilation.

**Before-block check**:
- Grep confirmed: No — the new files do not exist yet (empty directories confirmed via `ls`)
- Acceptance commands present: Yes — 4 `test -f` checks

**Depends on**: None (creates new files that reference seed files from TASK-2, but files can be created independently)

**Notes**:
- The `KpointsMpGrid` import path (`castep_cell_io::cell::bz_sampling_kpoints::KpointsMpGrid`) is from an external crate and cannot be verified locally. The codebase-state.md asserts this path exists.
- The `Workspace::with_root_dir` takes `impl Into<PathBuf>` and returns `Self`, matching the chaining pattern used.
- The `main.rs` uses unused variable `seed_name_collect` (line 383) — this may produce a compiler warning but not an error.

---

## TASK-2: Copy seed files for multi_param_sweep binary

**Goal**: Clear. Create ZnO.cell and ZnO.param seed files.

**Target files**: Explicitly specified with full file paths (`examples/multi_param_sweep/seeds/ZnO.cell`, `examples/multi_param_sweep/seeds/ZnO.param`).

**Implementation detail**: Complete. Full file contents provided as `after` blocks.

**New types/interfaces**: Not applicable (data files).

**Module wiring check**: Not applicable (not Rust source files).

**Known failure mode check**:
- Missing `pub mod` risk: Low — seed files, not modules
- Missing `pub use` risk: Not applicable
- Stale import risk: Low — these files are `include_str!`'d by TASK-1's `main.rs`. No import paths to go stale.

**Before-block check**:
- Grep confirmed: No — the seed directory exists but is empty
- Acceptance commands present: Yes — 2 `test -f` checks

**Depends on**: None

**Notes**: The seed files content matches the existing seed files in `examples/hubbard_u_sweep_slurm/seeds/`.

---

## TASK-3: Create scf_dos_chain binary source files (Cargo.toml, config.rs, job_script.rs, main.rs)

**Goal**: Clear. Create four source files for a new binary crate `scf_dos_chain`.

**Target files**: Explicitly specified with full file paths.

**Implementation detail**: Complete. Full file contents provided as `after` blocks.

**New types/interfaces**: Fully defined inline.

**Module wiring check** (binary crate):
- `pub mod` in parent: Not applicable
- `pub use` re-export: Not applicable
- Consumer updates co-located: Yes — `main.rs` declares `mod config; mod job_script;`, `config.rs` defines `pub struct ChainConfig`, `job_script.rs` imports via `use crate::config::ChainConfig;`, `main.rs` imports via `use config::ChainConfig;` and `use job_script::generate_job_script;`

**Known failure mode check**:
- Missing `pub mod` risk: Low — both `mod config;` and `mod job_script;` are declared in `main.rs`
- Missing `pub use` risk: Low — binary crate
- Stale import risk: Medium — same as TASK-1, seed files are in TASK-4. The same `include_str!("../seeds/...")` dependency exists.

**Before-block check**:
- Grep confirmed: No — the new files do not exist yet (empty directories confirmed via `ls`)
- Acceptance commands present: Yes — 4 `test -f` checks

**Depends on**: None, but TASK-4 seed files are needed before compilation would succeed.

**Notes**:
- **WIRING ISSUE FLAGGED**: The import `use castep_cell_io::param::general::Task;` on line 787 of the main.rs code will shadow the `workflow_core::Task` that is brought in via `use workflow_utils::prelude::*` (which re-exports `workflow_core::prelude::*`). In Rust, an explicit `use` shadows glob imports. This means all references to `Task` in the file (such as `Task::new()` on line 831 and the return type `Result<Task, WorkflowError>` on line 807) would resolve to `castep_cell_io::param::general::Task` rather than `workflow_core::Task`.
  - If `castep_cell_io::param::general::Task` is an enum (with variants like `BandStructure`, `SinglePoint`), it likely has no `new()` method, causing a compilation error at line 831.
  - The only place where the castep `Task` type is needed is on line 946: `param_doc.general.task = Some(Task::BandStructure);`
  - **Recommended fix**: Use `use castep_cell_io::param::general::Task as CastepTask;` and change line 946 to `CastepTask::BandStructure`. Alternatively, test whether the actual module path requires the `task` submodule (i.e., `castep_cell_io::param::general::task::Task::BandStructure`), in which case the import path may be simply wrong.
- The import path `castep_cell_io::param::general::Task` cannot be verified against the external crate from this workspace. If the path is wrong, it would produce an "unresolved import" error instead of a naming conflict.

---

## TASK-4: Copy seed files for scf_dos_chain binary

**Goal**: Clear. Create ZnO.cell and ZnO.param seed files.

**Target files**: Explicitly specified (`examples/scf_dos_chain/seeds/ZnO.cell`, `examples/scf_dos_chain/seeds/ZnO.param`).

**Implementation detail**: Complete. Full file contents provided.

**New types/interfaces**: Not applicable.

**Module wiring check**: Not applicable.

**Known failure mode check**:
- Missing `pub mod` risk: Low
- Missing `pub use` risk: Not applicable
- Stale import risk: Low

**Before-block check**:
- Grep confirmed: No — empty `seeds/` directory
- Acceptance commands present: Yes — 2 `test -f` checks

**Depends on**: None

**Notes**: Identical content to TASK-2 seed files.

---

## TASK-5: Update root Cargo.toml workspace members and delete old hubbard_u_sweep_slurm directory

**Goal**: Clear. Two actions: (1) modify workspace members in root `Cargo.toml`, (2) delete old binary directory.

**Target files**: Root `Cargo.toml` specified with exact `before`/`after` blocks.

**Implementation detail**: Complete. The `before` and `after` sections clearly show the exact diff.

**New types/interfaces**: Not applicable.

**Module wiring check**:
- `pub mod` in parent: Not applicable (workspace config, not Rust module)
- `pub use` re-export: Not applicable
- Consumer updates co-located: Not applicable — the old member is removed and two new members are added. This is the workspace configuration change.

**Known failure mode check**:
- Missing `pub mod` risk: Low — workspace members are listed in `Cargo.toml`, not `pub mod` declarations
- Missing `pub use` risk: Not applicable
- Stale import risk: Medium — after deleting `examples/hubbard_u_sweep_slurm`, any remaining file that references it would break. A `grep` for `hubbard_u_sweep_slurm` found only in: root `Cargo.toml` (being modified by this task), the plan files, and codebase-state.md. No source files outside the binary itself reference it.

**Before-block check**:
- Grep confirmed: Yes — root `Cargo.toml` contains `"examples/hubbard_u_sweep_slurm"` on line 6
- Acceptance commands present: Yes — `cargo check --workspace`, `cargo test --workspace`, `rm -rf`, `test ! -d`

**Depends on**: TASK-1, TASK-2, TASK-3, TASK-4 (as declared in `[dependencies]`). The new binary directories must exist and the new seed files must be present before the workspace is updated and verified.

**Notes**:
- The `cargo check --workspace` acceptance criterion will verify that all new code compiles. If TASK-3 has the Task naming conflict flagged above, this check will fail.
- The `cargo test --workspace` criterion will run all tests including the `no_literal_tabs`, `contains_sbatch_directives`, etc. tests in the new binaries.
- The `rm -rf` and `test ! -d` acceptance commands are destructive but irreversible. No backup step is mentioned, but the deleted code is being replaced by the two new binaries.
- The plan's dependency declaration `TASK-5 = ["TASK-1", "TASK-2", "TASK-3", "TASK-4"]` is correct and necessary.

---

## Summary

| Task | Module Wiring | Known Failure Risks | Before-block Verified | Depends On |
|------|--------------|---------------------|-----------------------|------------|
| TASK-1 | Not applicable (binary) | Stale import: Medium (seed refs) | No (files don't exist) | None |
| TASK-2 | Not applicable (data files) | Low | No (files don't exist) | None |
| TASK-3 | **FLAGGED**: Task naming conflict at `use castep_cell_io::param::general::Task;` shadows `workflow_core::Task` from prelude | Stale import: Medium (seed refs); Naming conflict: **High** (compilation error) | No (files don't exist) | None |
| TASK-4 | Not applicable (data files) | Low | No (files don't exist) | None |
| TASK-5 | Not applicable (workspace config) | Stale import: Medium (deleted dir references) | Yes (confirmed in Cargo.toml) | TASK-1, TASK-2, TASK-3, TASK-4 |

**Tasks checked**: 5
**Wiring issues flagged**: 1
- TASK-3: `use castep_cell_io::param::general::Task;` will shadow `workflow_core::Task` from the glob prelude import, causing `Task::new()` at line 831 to fail compilation.

**Before-block unverified**: 4 (TASK-1, TASK-2, TASK-3, TASK-4 — files don't exist yet, as expected)

**Additional items flagged**:
1. TASK-3 import path `castep_cell_io::param::general::Task` may itself be incorrect — the plan documentation references `castep_cell_io::param::general::task::Task` (note the lowercase `task` module). If the draft plan's shortened path is wrong, this produces an "unresolved import" error rather than a naming conflict.
2. TASK-1's `main.rs` uses unused variable `seed_name_collect` (line 383), which will generate a compiler warning but not an error.
3. Both TASK-1 and TASK-3 `main.rs` files import `use std::sync::Arc;` — confirmed correct for `Arc<dyn ProcessRunner>` and `Arc::new(QueuedRunner(...))` usage.
