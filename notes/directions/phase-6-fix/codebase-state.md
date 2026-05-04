# Codebase State Analysis for Phase 6 Fix Plan

**Generated**: 2026-05-05
**Plan file**: `plans/phase-6-fix/PHASE_6_FIX_PLAN.md`

---

## 1. Current File Tree for Affected Areas

### Workspace root
```
/Users/tony/programming/castep_workflow_framework/
  Cargo.toml              (workspace root)
  Cargo.lock
  workflow_core/           (lib crate)
    Cargo.toml
    src/
      lib.rs, dag.rs, error.rs, monitoring.rs, prelude.rs,
      process.rs, state.rs, task.rs, workflow.rs
    tests/
      bin/mock_castep       (pre-compiled test binary)
      collect_failure_policy.rs, dependencies.rs, hook_recording.rs,
      hubbard_u_sweep.rs, integration.rs, log_persistence.rs,
      queued_workflow.rs, resume.rs, timeout_integration.rs
  workflow_utils/          (lib crate)
    Cargo.toml
    src/
      lib.rs, executor.rs, files.rs, monitoring.rs, prelude.rs, queued.rs
    tests/
      executor_tests.rs, files_tests.rs, monitoring_tests.rs,
      process_tests.rs, queued_integration.rs
  workflow-cli/            (bin crate)
    Cargo.toml
    src/main.rs
  examples/
    hubbard_u_sweep/       (bin crate -- TO BE UPDATED)
      Cargo.toml
      src/main.rs
      seeds/ZnO.cell, ZnO.param
    hubbard_u_sweep_slurm/ (bin crate -- TO BE DELETED)
      Cargo.toml
      src/main.rs, config.rs, job_script.rs
      seeds/ZnO.cell, ZnO.param
      .validation-complete
  notes/
    directions/phase-6-fix/
      workspace-map.json
      deferred-and-patterns.md
    plan-enrichment/phase-6-fix/
      codebase-state.md, deferred-and-patterns.md, draft-elaboration.md,
      gather-summary.md, task-checklist.md
    plan-reviews/phase-6-fix/
      decisions.md
```

### Files to be created
```
examples/multi_param_sweep/
  Cargo.toml, src/main.rs, src/config.rs, src/job_script.rs,
  seeds/ZnO.cell, ZnO.param
examples/scf_dos_chain/
  Cargo.toml, src/main.rs, src/config.rs, src/job_script.rs,
  seeds/ZnO.cell, ZnO.param
```

### Files to be deleted
```
examples/hubbard_u_sweep_slurm/   (entire directory)
```

### Files to be updated
```
/Users/tony/programming/castep_workflow_framework/Cargo.toml   (workspace members)
/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep/Cargo.toml
/Users/tony/programming/castep_workflow_framework/examples/hubbard_u_sweep/src/main.rs
```

---

## 2. Key Type and Function Signatures

### Core types (workflow_core::task)

```
pub struct Task {
    pub id: String,
    pub dependencies: Vec<String>,
    pub workdir: PathBuf,
    pub mode: ExecutionMode,
    pub setup: Option<TaskClosure>,
    pub collect: Option<TaskClosure>,
    pub monitors: Vec<MonitoringHook>,
    pub(crate) collect_failure_policy: CollectFailurePolicy,
}

impl Task {
    pub fn new(id: impl Into<String>, mode: ExecutionMode) -> Self;
    pub fn depends_on(mut self, id: impl Into<String>) -> Self;
    pub fn workdir(mut self, path: impl Into<PathBuf>) -> Self;
    pub fn setup<F, E>(mut self, f: F) -> Self
        where F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
              E: std::error::Error + Send + Sync + 'static;
    pub fn collect<F, E>(mut self, f: F) -> Self
        where F: Fn(&Path) -> Result<(), E> + Send + Sync + 'static,
              E: std::error::Error + Send + Sync + 'static;
}
```

### ExecutionMode
```
pub enum ExecutionMode {
    Direct { command: String, args: Vec<String>, env: HashMap<String, String>, timeout: Option<Duration> },
    Queued,
}
impl ExecutionMode {
    pub fn direct(command: impl Into<String>, args: &[&str]) -> Self;
}
```

### Workflow
```
pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    ...
}

impl Workflow {
    pub fn new(name: impl Into<String>) -> Self;
    pub fn with_max_parallel(mut self, n: usize) -> Result<Self, WorkflowError>;
    pub fn with_log_dir(mut self, path: impl Into<PathBuf>) -> Self;
    pub fn with_queued_submitter(mut self, qs: Arc<dyn QueuedSubmitter>) -> Self;
    pub fn with_root_dir(mut self, path: impl Into<PathBuf>) -> Self;
    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError>;
    pub fn dry_run(&self) -> Result<Vec<String>, WorkflowError>;
    pub fn run(&mut self, state: &mut dyn StateStore, runner: Arc<dyn ProcessRunner>,
               hook_executor: Arc<dyn HookExecutor>) -> Result<WorkflowSummary, WorkflowError>;
}
```

### workflow_utils convenience API
```
pub fn run_default(workflow: &mut Workflow, state: &mut dyn StateStore)
    -> Result<WorkflowSummary, WorkflowError>;

// prelude re-exports:
// WorkflowError, JsonStateStore, StateStore, StateStoreExt, TaskStatus,
// CollectFailurePolicy, ExecutionMode, Task, Workflow, WorkflowSummary,
// HookExecutor, ProcessRunner
// create_dir, copy_file, read_file, write_file, exists, remove_dir
// run_default, QueuedRunner, SchedulerKind, ShellHookExecutor,
// SystemProcessRunner, JOB_SCRIPT_NAME
```

---

## 3. Module Dependency Graph

```
                     workflow_core (lib)
                     /     |       \
                    /      |        \
                   /       |         \
     workflow_utils     workflow-cli   hubbard_u_sweep (bin)
         |                                 |
         |                          hubbard_u_sweep_slurm (bin) [TO DELETE]
         |
         +-- depends on workflow_core
         +-- re-exports workflow_core::prelude fully
```

### External dependencies
- `castep-cell-fmt` v0.1.0 — used by hubbard_u_sweep, hubbard_u_sweep_slurm
- `castep-cell-io` v0.4.0 — used by hubbard_u_sweep, hubbard_u_sweep_slurm

The plan requires changing to a local path dep for `castep-cell-io` at v0.5.0 API:
`castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }`

---

## 4. Existing Test Structure

### workflow_core tests
- `task.rs` — task_builder, direct_constructor_fields, execution_mode_debug, depends_on_chaining
- `workflow.rs` — single_task_completes, chain_respects_order, failed_task_skips_dependent,
  dry_run_returns_topo_order, duplicate_task_id_errors, valid_dependency_add, etc.
- Integration tests: collect_failure_policy, dependencies, hook_recording, hubbard_u_sweep,
  integration (resume), log_persistence, queued_workflow, resume, timeout_integration

### workflow_utils tests
- `queued.rs` — 6 tests for parse_job_id (Slurm/PBS variants, edge cases)
- `files_tests.rs` — 5 tests for read/write/copy/remove operations
- executor, monitoring, process, queued_integration test files

### hubbard_u_sweep_slurm tests (must be ported before deletion)
- `config.rs` — 7 tests for parse_u_values (basic, whitespace, single, invalid, empty, etc.)
- `job_script.rs` — 6 tests for generate_job_script (sbatch directives, seed name, tabs, shebang, nix, mpi)

---

## 5. Observations and Patterns

### A. castep-cell-io/fmt are NOT local — must create local path dep
The plan requires `castep-cell-io = { path = "../castep-cell-io/castep_cell_io" }`.
The existing dep is v0.4.0 from crates.io. The plan amendment confirms path-dep approach.

### B. TASK-3 naming conflict: Task name collision
`use castep_cell_io::param::general::Task;` will shadow `workflow_core::task::Task` from glob import.
Resolution: alias import (`as CastepTask`) or use fully-qualified path.

### C. Dead re-export warnings in workspace map
~15 false positives from cross-crate resolution limitations. Not real issues.

### D. hubbard_u_sweep (non-slurm) also needs updates
- Cargo.toml: path dep for castep-cell-io
- main.rs: v0.5.0 parse API

### E. Seed files identical across existing binaries
Simple ZnO wurtzite cell for new binaries.

### F. job_script.rs code reuse
Both new binaries need identical job_script module. Config structs differ but SLURM fields overlap.

### G. Generate_job_script function signature
Takes config (SLURM fields), task_id, seed_name.

### H. API change: v0.4.0 → v0.5.0
- Free function `castep_cell_fmt::parse::<CellDocument>(&input)` replaces builder
- `KpointsMpGrid` is direct field on `CellDocument`
- `CutOffEnergy { value, unit: None }` pattern
- `to_cell_file()` and `to_string_many_spaced()` serialization unchanged

### I. Test coverage gaps for new binaries
- parse_kpoints and parse_cutoffs tests needed (not in plan)
- Sweep mode combinatoric logic tests
- Setup closure correctness tests
- DOS chain setup tests

### J. Existing integration test `test_hubbard_u_sweep_with_mock_castep`
Continues working — targets workflow_core directly.

### K. Compilation status
Current state compiles with v0.4.0 castep-cell-io from crates.io. New binaries need v0.5.0 path dep.

### L. Known failure modes from previous phases
1. Missing `pub use` re-exports
2. Incomplete consumer updates (constants/hardcoded strings)
3. Stale imports after refactoring
4. Dead API surface (newtypes with exposed inner types)
5. Stale documentation
