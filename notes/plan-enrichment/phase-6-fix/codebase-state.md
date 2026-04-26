# Codebase State — Phase 6 Fix Plan Enrichment

Generated: 2026-04-26
Plan: `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`
Branch: `phase-6-fix`

---

## File: Cargo.toml (workspace root)

### Public API
N/A (workspace configuration)

### Module wiring
- Workspace members: `workflow_core`, `workflow_utils`, `examples/hubbard_u_sweep`, `examples/hubbard_u_sweep_slurm`, `workflow-cli`
- Workspace dependencies: `anyhow`, `serde`, `petgraph`, `serde_json`, `tracing`, `tracing-subscriber`, `clap`, `signal-hook`, `thiserror`, `time`, `itertools`
- `workflow_core` defined as workspace member at `workflow_core`

### Plan relationship
- Plan says: Remove `"examples/hubbard_u_sweep_slurm"` from members, add `"examples/multi_param_sweep"` and `"examples/scf_dos_chain"`
- Current state: `"examples/hubbard_u_sweep_slurm"` is still in members list; new binaries not listed
- Gap: Needs two additions and one removal in `members` array (line 3-8)

---

## File: examples/hubbard_u_sweep_slurm/ (entire directory)

### Public API
N/A (to be deleted)

### Module wiring
- `Cargo.toml`: binary crate `hubbard_u_sweep_slurm`, deps: `anyhow`, `clap`, `castep-cell-fmt 0.1.0`, `castep-cell-io 0.4.0`, `itertools`, `workflow_core`, `workflow_utils`
- `src/main.rs`: mod `config`, mod `job_script`; functions: `build_one_task`, `build_chain`, `parse_second_values`, `build_sweep_tasks`, `main`
- `src/config.rs`: `pub struct SweepConfig` (Parser derive); `pub fn parse_u_values(s: &str) -> Result<Vec<f64>, String>`; test module with 7 tests
- `src/job_script.rs`: `pub fn generate_job_script(config, task_id, seed_name) -> String`; test module with 6 tests including `no_literal_tabs`
- `seeds/ZnO.cell`, `seeds/ZnO.param`
- `.validation-complete`: marker file from Phase 5A

### Plan relationship
- Plan says: Delete entire directory. Code will be ported/rewritten into two new binaries.
- Current state: Fully intact — three source files with complete implementations including tests.
- Gap: Directory exists and must be deleted as part of the fix.

---

## File: examples/hubbard_u_sweep/ (keep, unchanged)

### Public API
- Binary: `hubbard_u_sweep`
- No modules, single `main.rs`

### Module wiring
- `Cargo.toml`: binary crate, deps: `anyhow`, `castep-cell-fmt 0.1.0`, `castep-cell-io 0.4.0`, `workflow_core`, `workflow_utils`
- `src/main.rs`: no modules; single `main()` that builds 6 SCF tasks (U=0.0..5.0) in a direct-execution workflow
- `seeds/ZnO.cell` (253 bytes), `seeds/ZnO.param` (19 bytes)

### Plan relationship
- Plan says: Keep unchanged.
- Current state: Intact, matches plan.
- Gap: None.

---

## File: examples/multi_param_sweep/Cargo.toml

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Create with deps: `anyhow`, `clap`, `castep-cell-fmt`, `castep-cell-io`, `itertools`, `workflow_core`, `workflow_utils`; use `workspace = true` for workspace-managed deps (`anyhow`, `clap`, `itertools`)
- Current state: The directory `examples/multi_param_sweep/` exists with empty `src/` and `seeds/` subdirectories, but no files.
- Gap: `Cargo.toml` needs to be created. Reference: `examples/hubbard_u_sweep_slurm/Cargo.toml` for structure.

---

## File: examples/multi_param_sweep/src/main.rs

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Entry point with task builders, sweep logic, workflow runner. Functions:
  - `fn parse_kpoints(s: &str) -> anyhow::Result<Vec<[u32; 3]>>`
  - `fn parse_cutoffs(s: &str) -> anyhow::Result<Vec<f64>>`
  - Sweep logic: `product` mode via `itertools::iproduct!`, `pairwise` mode via zip (requires `--kpoints` and `--cutoffs` to be Some)
  - Task ID format: `scf_U{u}_k{k}_c{c}`
  - Setup closure: inject HubbardU, KPOINTS_MP_GRID, cutoff energy
  - KpointsMpGrid workaround: serialize via `.to_cell()` and merge into output `Vec<Cell>` via `to_string_many_spaced()`
  - API: `CellDocument` parse/field mutation, `ParamDocument` parse/`.basis_set.cutoff_energy`, `HubbardU::builder()`, `AtomHubbardU::builder()`, `OrbitalU::D(f64)` / `OrbitalU::F(f64)`, `CutOffEnergy { value, unit: None }`
- Current state: File does not exist (empty `src/` dir).
- Gap: Entire file needs to be created.

---

## File: examples/multi_param_sweep/src/config.rs

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Clap CLI config with flags: `--u-values` (default `"0.0,1.0,2.0,3.0,4.0,5.0"`), `--kpoints` (Option, default None), `--cutoffs` (Option, default None), `--sweep-mode` (default `"product"`), `--element` (default `"Zn"`), `--orbital` (default `'d'`), `--seed-name` (default `"ZnO"`), `--max-parallel` (default 4), `--local`, `--dry-run`, `--castep-command` (default `"castep"`), `--workdir` (default `"."`), plus slurm flags (`--partition`, `--ntasks`, `--nix-flake`, `--mpi-if`).
  - Also: `parse_u_values` should be imported or duplicated from old `config.rs`.
- Current state: File does not exist.
- Gap: Entire file needs to be created. The `parse_u_values` function can be copied from `examples/hubbard_u_sweep_slurm/src/config.rs` (lines 75-84).

---

## File: examples/multi_param_sweep/src/job_script.rs

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: SLURM script generation, ported from old binary (`examples/hubbard_u_sweep_slurm/src/job_script.rs`). Formatting fix: clean heredoc template, no literal tab characters mixed with spaces, consistent quoting around SBATCH directives. Port the `no_literal_tabs` test.
- Current state: File does not exist. Source to port exists at `examples/hubbard_u_sweep_slurm/src/job_script.rs` (87 lines).
- Gap: Entire file needs to be created. The existing `generate_job_script` uses `format!()` with literal `\t`-free template (already clean). The formatting fix from D.2 is about ensuring consistent quoting around SBATCH directives.

---

## File: examples/multi_param_sweep/seeds/ZnO.cell

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Seed cell file for ZnO.
- Current state: Empty `seeds/` directory. Source exists at `examples/hubbard_u_sweep_slurm/seeds/ZnO.cell` (252 bytes).
- Gap: File needs to be copied from old binary.

---

## File: examples/multi_param_sweep/seeds/ZnO.param

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Seed param file for ZnO.
- Current state: Empty `seeds/` directory. Source exists at `examples/hubbard_u_sweep_slurm/seeds/ZnO.param` (19 bytes).
- Gap: File needs to be copied from old binary.

---

## File: examples/scf_dos_chain/Cargo.toml

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Create with same deps as `multi_param_sweep`.
- Current state: Directory exists with empty `src/` and `seeds/`, no files.
- Gap: `Cargo.toml` needs to be created.

---

## File: examples/scf_dos_chain/src/main.rs

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Entry point with SCF task, DOS task, chain wiring.
  - Task 1 `scf`: inject HubbardU, write `.cell` + `.param`, execute `castep ZnO`, collect: verify `ZnO.castep` exists with "Total time"
  - Task 2 `dos` (depends on `scf`): same workdir. Setup: write `ZnO_DOS.cell` (from seed), write `ZnO_DOS.param` (with `general.task = Some(Task::BandStructure)`), copy `ZnO.check` -> `ZnO_DOS.check`. Execute: `castep ZnO_DOS`. Collect: verify `ZnO_DOS.castep` has "Total time"
  - API: `ParamDocument.general.task = Some(Task::BandStructure)`, `Task::BandStructure` from `castep_cell_io::param::general::task::Task`, `copy_file` from `workflow_utils::files`
- Current state: File does not exist.
- Gap: Entire file needs to be created.

---

## File: examples/scf_dos_chain/src/config.rs

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Simpler CLI. Single `--u-value` (default `3.0`), `--element` (default `"Zn"`), `--orbital` (default `'d'`), `--seed-name` (default `"ZnO"`), `--max-parallel` (default `1`), `--local`, `--dry-run`, `--castep-command` (default `"castep"`), `--workdir` (default `"."`), plus slurm flags. No `--kpoints`, `--cutoffs`, `--sweep-mode`, `--u-values` (plural).
- Current state: File does not exist.
- Gap: Entire file needs to be created.

---

## File: examples/scf_dos_chain/src/job_script.rs

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Same as Binary 1 (`multi_param_sweep/src/job_script.rs`), ported from old binary.
- Current state: File does not exist.
- Gap: Entire file needs to be created. Same content as `multi_param_sweep/src/job_script.rs`.

---

## File: examples/scf_dos_chain/seeds/ZnO.cell

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Seed cell file.
- Current state: Empty `seeds/` directory.
- Gap: File needs to be copied from old binary.

---

## File: examples/scf_dos_chain/seeds/ZnO.param

### Public API
N/A (does not exist yet)

### Module wiring
N/A (does not exist yet)

### Plan relationship
- Plan says: Seed param file.
- Current state: Empty `seeds/` directory.
- Gap: File needs to be copied from old binary.

---

## Workspace Crate: workflow_core

### Public API (types referenced by plan)

- `pub struct Task` (workflow_core/src/task.rs, line 57)
  - Fields: `id: String`, `dependencies: Vec<String>`, `workdir: PathBuf`, `mode: ExecutionMode`, `setup: Option<TaskClosure>`, `collect: Option<TaskClosure>`, `monitors: Vec<MonitoringHook>`, `collect_failure_policy: CollectFailurePolicy` (crate-private)
  - Builder: `Task::new(id, mode) -> Self`, `.depends_on(id) -> Self`, `.workdir(path) -> Self`, `.setup(f) -> Self`, `.collect(f) -> Self`, `.collect_failure_policy(policy) -> Self`, `.monitors(hooks) -> Self`, `.add_monitor(hook) -> Self`

- `pub enum ExecutionMode` (workflow_core/src/task.rs, line 26)
  - Variants: `Direct { command: String, args: Vec<String>, env: HashMap<String, String>, timeout: Option<Duration> }`, `Queued`
  - Constructor: `ExecutionMode::direct(command, args: &[&str]) -> Self` (convenience, no env/timeout)

- `pub struct Workflow` (workflow_core/src/workflow.rs, line 28)
  - Public methods: `new(name) -> Self`, `with_max_parallel(n) -> Result<Self, WorkflowError>`, `with_log_dir(path) -> Self`, `with_queued_submitter(qs) -> Self`, `with_root_dir(path) -> Self`, `successor_map() -> Option<&TaskSuccessors>`, `add_task(task) -> Result<(), WorkflowError>`, `dry_run() -> Result<Vec<String>, WorkflowError>`, `run(state, runner, hook_executor) -> Result<WorkflowSummary, WorkflowError>`

- `pub struct WorkflowSummary` (workflow_core/src/workflow.rs, line 570)
  - Fields: `succeeded: Vec<String>`, `failed: Vec<FailedTask>`, `skipped: Vec<String>`, `duration: Duration`

- `pub enum WorkflowError` (workflow_core/src/error.rs, line 5)
  - Variants: `DuplicateTaskId(String)`, `CycleDetected`, `UnknownDependency { task, dependency }`, `StateCorrupted(String)`, `TaskTimeout(String)`, `InvalidConfig(String)`, `IoWithPath { path, source }`, `Io(std::io::Error)`, `Interrupted`, `QueueSubmitFailed(String)`

- `pub trait StateStore` (workflow_core/src/state.rs, line 33)
  - Methods: `get_status`, `set_status`, `all_tasks`, `save`
- `pub trait StateStoreExt` (workflow_core/src/state.rs, line 48) — extension methods: `mark_running`, `mark_completed`, `mark_failed`, `mark_pending`, `mark_skipped`, `mark_skipped_due_to_dep_failure`, `summary`, `is_completed`
- `pub struct JsonStateStore` (workflow_core/src/state.rs, line 190)
  - Methods: `new(name, path)`, `load(path)`, `load_raw(path)`, `workflow_name()`, `path()`, `task_successors()`, `set_task_graph(successors)`

- `pub trait ProcessRunner` (workflow_core/src/process.rs, line 12) — trait with `spawn`
- `pub trait ProcessHandle` (workflow_core/src/process.rs, line 25) — trait with `is_running`, `terminate`, `wait`
- `pub trait QueuedSubmitter` (workflow_core/src/process.rs, line 51) — trait with `submit`
- `pub trait HookExecutor` (workflow_core/src/monitoring.rs) — trait with `execute_hook`

### Plan relationship
- Plan says: New binaries will use `Task`, `ExecutionMode`, `Workflow`, `WorkflowError`, `WorkflowSummary`, `JsonStateStore`, `QueuedRunner`, `SchedulerKind`, etc.
- Current state: All types exist and match the API described in the plan.
- Gap: None — crate API is sufficient.

---

## Workspace Crate: workflow_utils

### Public API (types referenced by plan)

- `pub mod prelude` (workflow_utils/src/prelude.rs)
  - Re-exports: `workflow_core::prelude::*`, `crate::{copy_file, create_dir, exists, read_file, remove_dir, run_default, write_file, QueuedRunner, SchedulerKind, ShellHookExecutor, SystemProcessRunner, JOB_SCRIPT_NAME}`

- `pub fn read_file(path) -> Result<String, WorkflowError>` (workflow_utils/src/files.rs, line 8)
- `pub fn write_file(path, content) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 13)
- `pub fn copy_file(from, to) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 21)
- `pub fn create_dir(path) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 31)
- `pub fn remove_dir(path) -> Result<(), WorkflowError>` (workflow_utils/src/files.rs, line 36)
- `pub fn exists(path) -> bool` (workflow_utils/src/files.rs, line 41)
- `pub fn run_default(workflow, state) -> Result<WorkflowSummary, WorkflowError>` (workflow_utils/src/lib.rs, line 31)

- `pub struct QueuedRunner` (workflow_utils/src/queued.rs, line 24)
  - Methods: `new(scheduler: SchedulerKind) -> Self`, `scheduler() -> SchedulerKind`
  - Implements: `QueuedSubmitter`

- `pub enum SchedulerKind` (workflow_utils/src/queued.rs, line 13)
  - Variants: `Slurm`, `Pbs`

- `pub const JOB_SCRIPT_NAME: &str = "job.sh"` (workflow_utils/src/queued.rs, line 9)

- `pub struct SystemProcessRunner` (workflow_utils/src/executor.rs, line 110)
  - Methods: `new() -> Self`, `with_log_dir(dir) -> Self`
  - Implements: `ProcessRunner`

- `pub struct ShellHookExecutor` (workflow_utils/src/monitoring.rs, line 6)
  - Implements: `HookExecutor`

### Plan relationship
- Plan says: New binaries use `read_file`, `copy_file`, `run_default`, `QueuedRunner`, `SchedulerKind`, `SystemProcessRunner`, `ShellHookExecutor`, `JOB_SCRIPT_NAME`.
- Current state: All types and functions exist and are publicly exported via `prelude` and direct crate re-exports.
- Gap: None — crate API is sufficient.

---

## External Crate: castep-cell-io (v0.4.0)

Types referenced in the plan (not inspectable in this workspace since it's an external dependency):

- `CellDocument` — `parse()`, field mutation (`.hubbard_u`), `.to_cell_file()`
- `ParamDocument` — `parse()`, `.basis_set.cutoff_energy`, `.general.task`, `.to_cell_file()`
- `KpointsMpGrid([u32; 3])` — `.to_cell()` returns `Cell` block; NOT a field on `CellDocument`
- `HubbardU::builder()`, `AtomHubbardU::builder()`, `OrbitalU::D(f64)`, `OrbitalU::F(f64)`, `Species::Symbol(String)`, `HubbardUUnit::ElectronVolt`
- `CutOffEnergy { value: f64, unit: None }` — None defaults to eV in CASTEP
- `Task::BandStructure` — path: `castep_cell_io::param::general::task::Task`
- `Cell` type — output of `.to_cell()`, input to `to_string_many_spaced()`

### Plan relationship
- Plan says: New binaries will use all of these types. Notable: `KpointsMpGrid` is NOT a field on `CellDocument` in v0.4.0. Workaround: serialize via `.to_cell()` to a `Cell` block, merge into `Vec<Cell>`, serialize via `to_string_many_spaced()`.
- Current state: External dependency, confirmed available via `Cargo.lock` / existing code that uses these types.
- Gap: `KpointsMpGrid` workaround is a known limitation documented in the plan.

---

## External Crate: castep-cell-fmt (v0.1.0)

Functions referenced in the plan:

- `parse(&str) -> Result<CellDocument, ...>` — parses cell file text
- `to_string_many_spaced(&[Cell]) -> String` — serializes `Vec<Cell>` blocks to spaced CASTEP format
- `ToCellFile` trait — provides `.to_cell_file()` -> `Vec<Cell>` on `CellDocument` and `ParamDocument`

### Plan relationship
- Plan says: Used for parsing seeds and serializing modified documents.
- Current state: Already used by both existing binaries. API is stable.
- Gap: None.

---

## Summary

| Status | Count | Details |
|--------|-------|---------|
| Files to create | 12 | 2 Cargo.toml, 2 main.rs, 2 config.rs, 2 job_script.rs, 4 seed files |
| Files to modify | 1 | Root `Cargo.toml` (workspace members) |
| Files to delete | 7+ | `examples/hubbard_u_sweep_slurm/` entire directory (Cargo.toml, 3 src files, 2 seeds, 1 validation marker) |
| Files to keep | 4 | `examples/hubbard_u_sweep/` (Cargo.toml, main.rs, 2 seeds) |
| Workspace crates OK | 2 | `workflow_core`, `workflow_utils` — all referenced APIs exist |
| Empty placeholder dirs | 2 | `examples/multi_param_sweep/`, `examples/scf_dos_chain/` — dirs exist, contents empty |
