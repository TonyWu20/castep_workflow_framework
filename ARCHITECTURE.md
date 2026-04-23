# Workflow Framework Architecture

**Version:** 5.0 (Utilities-Based)
**Last Updated:** 2026-04-23
**Status:** Phases 1–5 Complete (1.1 + 1.2 + 1.3 + 2.1 + 2.2 + 3 + 4 + 5A + 5B complete)

## Executive Summary

Utilities-based arch: Layer 2 = generic exec utils, NOT software adapters. Software logic → parser libs (castep-cell-io, castep-cell-fmt) or project crates (Layer 3).

**Key Decision:** After first-principles analysis, killed adapter pattern. Layer 2 = pure generic utils. No traits, no adapters, no software code.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Project Crates (Domain-Specific)              │
│  - User's research-specific workflow logic              │
│  - Uses parser libraries directly (castep-cell-io)      │
│  - Uses Layer 2 utilities for I/O and execution         │
│  - Full control over workflow construction              │
│  - Examples: hubbard_u_sweep, hubbard_u_sweep_slurm     │
└────────────────────────┬────────────────────────────────┘
                         │ uses
┌────────────────────────▼────────────────────────────────┐
│  Layer 2: workflow_utils (Generic Utilities)            │
│  - TaskExecutor: Generic process execution              │
│  - files module: Generic file I/O                       │
│  - MonitoringHook: External monitoring integration      │
│  - SystemProcessRunner / ShellHookExecutor              │
│  - QueuedRunner: SLURM/PBS batch submission             │
│  - run_default(): convenience runner (Phase 5B)         │
│  - prelude: re-exports all common types (Phase 5B)      │
│  - NO software-specific code                            │
│  - NO traits, NO adapters                               │
└────────────────────────┬────────────────────────────────┘
                         │ uses
┌────────────────────────▼────────────────────────────────┐
│  Layer 1: workflow_core (Foundation)                    │
│  - Workflow: DAG container and orchestration            │
│  - Task: Execution unit with setup/collect closures     │
│  - ExecutionMode: Direct | Queued                       │
│  - Execution engine: Dependency resolution, parallel    │
│  - State management: StateStore trait, JsonStateStore   │
│  - WorkflowError (#[non_exhaustive]), WorkflowSummary   │
│  - Signal handling: SIGTERM/SIGINT graceful shutdown    │
│  - workflow-cli: status/inspect/retry binary            │
│  - prelude: re-exports all common types (Phase 5B)      │
└────────────────────────┬────────────────────────────────┘
                         │ uses
┌────────────────────────▼────────────────────────────────┐
│  Parser Libraries (Software-Specific)                   │
│  - castep-cell-io: CASTEP file format (structs)         │
│  - castep-cell-fmt: CASTEP file format/parse utilities  │
│  - vasp-io: VASP file format (future)                   │
│  - qe-io: Quantum ESPRESSO format (future)              │
│  - Builder pattern for all keyword structs              │
└─────────────────────────────────────────────────────────┘
```

## First Principles: Why No Adapters?

### What Does a Workflow Actually Need?

1. **Input Preparation**: Read seed, modify, write to workdir
2. **Execution**: Run binary in workdir w/ args
3. **Monitoring**: Check output, run external scripts, detect done

### What's Truly Software-Specific?

| Concern                       | Where It Belongs                          |
| ----------------------------- | ----------------------------------------- |
| File format (.cell syntax)    | castep-cell-io (parser library)           |
| Document structure (HubbardU) | castep-cell-io (parser library)           |
| Modifications (set U value)   | castep-cell-io builders                   |
| Binary name ("castep")        | Layer 3 (project knows what it's running) |
| Command arguments             | Layer 3 (project-specific)                |
| Output parsing                | External scripts (via monitoring hooks)   |

**Answer: Nothing!** All software logic already in parser libs or Layer 3.

### What's Truly Generic?

- File I/O (read/write any file)
- Process exec (run any cmd)
- Workdir mgmt (create, clean)
- Monitoring hooks (run external cmds)
- Status tracking (running, done, failed)

**Conclusion:** Layer 2 = generic utils, not software adapters.

## Layer 1: workflow_core (Foundation)

### Purpose

Generic workflow orchestration: DAG mgmt, dependency resolution, parallel exec, state persistence, signal handling.

### Core Types

```rust
/// Workflow container with DAG execution
pub struct Workflow {
    name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    log_dir: Option<PathBuf>,
    queued_submitter: Option<Arc<dyn QueuedSubmitter>>,
}

impl Workflow {
    /// Create new workflow
    pub fn new(name: impl Into<String>) -> Self;

    /// Set max concurrent tasks (returns Err if zero)
    pub fn with_max_parallel(self, n: usize) -> Result<Self, WorkflowError>;

    /// Set log directory for task stdout/stderr persistence
    pub fn with_log_dir(self, dir: impl Into<PathBuf>) -> Self;

    /// Set queued job submitter (for ExecutionMode::Queued tasks)
    pub fn with_queued_submitter(self, submitter: Arc<dyn QueuedSubmitter>) -> Self;

    /// Add task to workflow
    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError>;

    /// Execute workflow (resolves dependencies, runs in parallel where possible)
    pub fn run(
        &mut self,
        state: &mut dyn StateStore,
        runner: Arc<dyn ProcessRunner>,
        executor: Arc<dyn HookExecutor>,
    ) -> Result<WorkflowSummary, WorkflowError>;

    /// Dry-run: returns task execution order without executing
    pub fn dry_run(&self) -> Result<Vec<String>, WorkflowError>;
}

/// Task: execution unit with setup/collect closures
pub struct Task {
    id: String,
    dependencies: Vec<String>,
    execution_mode: ExecutionMode,
    workdir: Option<PathBuf>,
    setup: Option<TaskClosure>,
    collect: Option<TaskClosure>,
    monitors: Vec<MonitoringHook>,
}

/// Closure type alias to avoid type_complexity lint
pub type TaskClosure = Box<dyn Fn(&Path) -> Result<(), WorkflowError> + Send + Sync>;

impl Task {
    /// Create task with execution mode
    pub fn new(id: impl Into<String>, mode: ExecutionMode) -> Self;

    /// Set task working directory
    pub fn workdir(self, dir: PathBuf) -> Self;

    /// Add dependency on another task
    pub fn depends_on(self, task_id: impl Into<String>) -> Self;

    /// Set setup closure (runs before execution)
    pub fn setup<F>(self, f: F) -> Self
    where F: Fn(&Path) -> Result<(), WorkflowError> + Send + Sync + 'static;

    /// Set collect closure (runs after successful execution to validate output)
    pub fn collect<F>(self, f: F) -> Self
    where F: Fn(&Path) -> Result<(), WorkflowError> + Send + Sync + 'static;

    /// Attach monitoring hooks
    pub fn monitors(self, hooks: Vec<MonitoringHook>) -> Self;
}

/// Execution mode for tasks
pub enum ExecutionMode {
    /// Run command directly in subprocess
    Direct {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        timeout: Option<Duration>,
    },
    /// Submit to scheduler queue (SLURM/PBS via QueuedRunner in workflow_utils)
    Queued,
}

impl ExecutionMode {
    /// Convenience constructor for Direct mode (Phase 5B)
    pub fn direct(command: impl Into<String>, args: &[&str]) -> Self;
}

/// Task status for state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String },
    Skipped,
    SkippedDueToDependencyFailure,
}

/// Workflow result summary
pub struct WorkflowSummary {
    pub succeeded: Vec<String>,
    pub failed: Vec<FailedTask>,
    pub skipped: Vec<String>,
    pub duration: Duration,
}

/// State storage trait (I/O boundary abstraction)
pub trait StateStore {
    fn load(&mut self) -> Result<(), WorkflowError>;       // crash-recovery (resets Failed/Running → Pending)
    fn load_raw(&self) -> Result<WorkflowState, WorkflowError>; // read-only, no resets
    fn save(&mut self) -> Result<(), WorkflowError>;
    fn get_status(&self, id: &str) -> Option<TaskStatus>;
    fn set_status(&mut self, id: &str, status: TaskStatus);
}

pub trait StateStoreExt: StateStore {
    /// BFS over task_successors graph from given start nodes (Phase 5B: generic S)
    fn downstream_of<S: AsRef<str>>(&self, start: &[S]) -> Vec<String>;
}

/// JSON-backed state store with atomic writes
pub struct JsonStateStore {
    name: String,
    path: PathBuf,
    state: Option<WorkflowState>,
}

impl JsonStateStore {
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self;
}

/// Error type
#[non_exhaustive]
pub enum WorkflowError {
    DuplicateTaskId(String),
    CycleDetected,
    UnknownDependency { task: String, dep: String },
    StateCorrupted(String),
    TaskTimeout { task_id: String, timeout: Duration },
    InvalidConfig(String),
    Io(std::io::Error),
    Interrupted,
}

/// I/O boundary traits (implemented in workflow_utils)
pub trait ProcessRunner: Send + Sync {
    fn spawn(&self, cmd: &str, args: &[String], workdir: &Path, env: &HashMap<String, String>)
        -> Result<Box<dyn ProcessHandle>, WorkflowError>;
}

pub trait ProcessHandle: Send {
    fn wait(self: Box<Self>) -> Result<i32, WorkflowError>;
    fn pid(&self) -> u32;
}

pub trait HookExecutor: Send + Sync {
    fn execute(&self, hook: &MonitoringHook, ctx: &HookContext) -> Result<HookResult, WorkflowError>;
}
```

### Key Features

1. **DAG Execution**: Topo sort, parallel where possible, configurable `max_parallel`
2. **Dependency Resolution**: Auto ordering via `depends_on`
3. **State Persistence**: Crash-recovery (`load`) and read-only inspection (`load_raw`); atomic JSON writes
4. **Error Handling**: `WorkflowError` `#[non_exhaustive]` with `thiserror`; returns `WorkflowSummary` from `run()`
5. **Signal Handling**: SIGTERM/SIGINT via `signal-hook`; graceful shutdown, re-registers on each `run()`
6. **Structured Logging**: `tracing` events for task lifecycle and timing
7. **ExecutionMode::Queued**: delegates job submission to `QueuedRunner` (workflow_utils); polls via `squeue`/`qstat`
8. **Graph-Aware Retry**: successor graph persisted in state; CLI `retry` skips already-successful descendants

### What Layer 1 Does NOT Do

- File I/O (Layer 2)
- Process exec implementations (Layer 2 via `ProcessRunner` impl)
- Software logic (parser libs or Layer 3)
- Input prep (Layer 3)

## Layer 2: workflow_utils (Generic Utilities)

### Purpose

Generic utils for file I/O, process exec, monitoring, scheduler submission. No software code.

### TaskExecutor: Generic Process Execution

```rust
/// Generic task executor for running commands
pub struct TaskExecutor {
    workdir: PathBuf,
    command: String,
    args: Vec<String>,
    env_vars: HashMap<String, String>,
}

impl TaskExecutor {
    pub fn new(workdir: impl Into<PathBuf>) -> Self;
    pub fn command(self, cmd: impl Into<String>) -> Self;
    pub fn arg(self, arg: impl Into<String>) -> Self;
    pub fn args(self, args: Vec<String>) -> Self;
    pub fn env(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    pub fn execute(&self) -> Result<ExecutionResult, WorkflowError>;
    pub fn spawn(&self) -> Result<ExecutionHandle, WorkflowError>;
}
```

### ProcessRunner / HookExecutor Implementations

```rust
/// Implements ProcessRunner trait from workflow_core
pub struct SystemProcessRunner;

impl Default for SystemProcessRunner { ... }
impl SystemProcessRunner {
    pub fn new() -> Self;
}

/// Implements HookExecutor trait from workflow_core
/// Passes TASK_ID, TASK_STATE, WORKDIR, EXIT_CODE as env vars
pub struct ShellHookExecutor;
```

### QueuedRunner: SLURM/PBS Submission

```rust
pub enum SchedulerKind { Slurm, Pbs }

/// Submits job.sh from workdir via sbatch/qsub; polls squeue/qstat for completion
pub struct QueuedRunner {
    kind: SchedulerKind,
}

impl QueuedRunner {
    pub fn new(kind: SchedulerKind) -> Self;
}
```

### files: Generic File I/O

Re-exported flat at crate root:

```rust
use workflow_utils::{read_file, write_file, copy_file, create_dir, remove_dir, exists};

pub fn read_file(path: impl AsRef<Path>) -> Result<String, WorkflowError>;
pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<(), WorkflowError>;
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), WorkflowError>;
pub fn create_dir(path: impl AsRef<Path>) -> Result<(), WorkflowError>;
pub fn remove_dir(path: impl AsRef<Path>) -> Result<(), WorkflowError>;
pub fn exists(path: impl AsRef<Path>) -> bool;
```

### run_default: Convenience Runner (Phase 5B)

```rust
/// Runs a workflow with SystemProcessRunner and ShellHookExecutor.
/// Eliminates repeated Arc wiring in every binary.
pub fn run_default(
    workflow: &mut Workflow,
    state: &mut dyn StateStore,
) -> Result<WorkflowSummary, WorkflowError>;
```

### prelude: Re-exports (Phase 5B)

```rust
// workflow_utils/src/prelude.rs — imports all common types from both crates
use workflow_utils::prelude::*;
```

### What Layer 2 Does NOT Do

- Parse CASTEP files (castep-cell-io / castep-cell-fmt)
- Know HubbardU or CASTEP concepts (Layer 3)
- Implement traits/adapters (just utils)
- Decide workflow structure (Layer 1 or Layer 3)

## Layer 3: Project Crates (Domain-Specific)

### Purpose

User's research workflow logic. Uses parser libs directly, uses Layer 2 utils for I/O/exec.

### Example: Direct Mode (hubbard_u_sweep)

```rust
use workflow_utils::prelude::*;
use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use anyhow::Result;

fn main() -> Result<()> {
    workflow_core::init_default_logging().ok();

    let mut workflow = Workflow::new("hubbard_u_sweep")
        .with_max_parallel(4)?;

    let u_values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");

    for u in &u_values {
        let u = *u;
        let task_id = format!("scf_U{u:.1}");
        let workdir = PathBuf::from(format!("runs/U{u:.1}"));
        let seed_cell = seed_cell.to_owned();
        let seed_param = seed_param.to_owned();

        let task = Task::new(&task_id, ExecutionMode::direct("castep", &["ZnO"]))
            .workdir(workdir.clone())
            .setup(move |workdir| {
                create_dir(workdir)?;
                let mut cell_doc: CellDocument = parse(&seed_cell)
                    .map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;
                // ... inject HubbardU ...
                write_file(workdir.join("ZnO.cell"), &to_string_many_spaced(&cell_doc.to_cell_file()))?;
                write_file(workdir.join("ZnO.param"), &seed_param)?;
                Ok(())
            });

        workflow.add_task(task)?;
    }

    let state_path = PathBuf::from(".workflow.json");
    let mut state = JsonStateStore::new("hubbard_u_sweep", state_path);
    let summary = run_default(&mut workflow, &mut state)?;
    println!("{} succeeded, {} failed", summary.succeeded.len(), summary.failed.len());
    Ok(())
}
```

### Example: SLURM Queued Mode (hubbard_u_sweep_slurm — Phase 5A)

See `examples/hubbard_u_sweep_slurm/` for the full production implementation. Key differences:

```rust
// Workflow configured for queued submission
let mut workflow = Workflow::new("hubbard_u_sweep_slurm")
    .with_max_parallel(config.max_parallel)?
    .with_log_dir("logs")
    .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));

// Task uses Queued mode; setup writes job.sh for sbatch
let task = Task::new(&task_id, ExecutionMode::Queued)
    .workdir(workdir.clone())
    .setup(move |workdir| {
        create_dir(workdir)?;
        // ... write ZnO.cell, ZnO.param, job.sh ...
        Ok(())
    })
    .collect(move |workdir| {
        // Verify CASTEP output exists and is complete
        let castep_out = workdir.join(format!("{}.castep", seed_name));
        if !castep_out.exists() { return Err(...); }
        let content = read_file(&castep_out)?;
        if !content.contains("Total time") { return Err(...); }
        Ok(())
    });
```

Configuration via `clap` with env-var support (`CASTEP_SLURM_ACCOUNT`, `CASTEP_SLURM_PARTITION`, `CASTEP_MODULES`, `CASTEP_COMMAND`). Run with `--dry-run` to print topological order without submitting.

## Comparison: Adapter-Based vs Utilities-Based

### Old Design (Adapter-Based) ❌

```
Layer 3: HubbardUSweepProject
  ↓ uses
Layer 2: CastepAdapter (implements TaskAdapter trait)
  - prepare_input_files() - reads seeds, applies modifications
  - execute() - runs castep binary
  - monitoring_hooks() - returns hooks
  ↓ uses
Layer 1: workflow_core
  - Workflow/Task with TaskAdapter trait
```

**Problems:**

1. CastepAdapter dupes logic Layer 3 already has
2. TaskAdapter trait forces specific abstraction
3. New software needs new adapter impl
4. Layer 3 loses control over file prep details

### New Design (Utilities-Based) ✅

```
Layer 3: Project Crate
  - Uses castep-cell-io/castep-cell-fmt directly
  - Uses Layer 2 utilities for I/O and execution
  - Full control over workflow logic
  ↓ uses
Layer 2: Execution Utilities (workflow_utils)
  - SystemProcessRunner, ShellHookExecutor, QueuedRunner
  - files module (generic I/O)
  - run_default() convenience helper
  ↓ uses
Layer 1: workflow_core
  - Workflow/Task with ExecutionMode
  - StateStore trait, WorkflowError, WorkflowSummary
  - No TaskAdapter trait
```

**Benefits:**

1. Simpler: No trait, no adapters, just utils
2. More flexible: Layer 3 full control
3. Less code: No adapter boilerplate
4. Easier extend: Just use different parser lib
5. Clearer separation: Layer 2 truly generic

## Design Principles

1. **Separation of Concerns**
   - Layer 1: Orchestration only
   - Layer 2: Generic utils only
   - Layer 3: Domain logic only
   - Parser libs: Format-specific only

2. **No Premature Abstraction**
   - No traits unless multiple impls exist
   - No adapters unless they add value
   - Keep simple until complexity justified

3. **User Control**
   - Layer 3 full control over workflow
   - No hidden magic, no implicit behavior
   - Explicit > implicit

4. **Composability Over Inheritance**
   - Use functions, not class hierarchies
   - Compose utils, don't extend adapters
   - Build helpers as needed, don't force patterns

## Implementation Guidelines

These rules codify lessons learned from prior phases. Apply them from the start on new types.

**Newtype Encapsulation:** Design newtypes with full encapsulation on introduction. Expose methods that delegate to the inner collection, never expose the raw inner type via a public accessor. Introducing `inner()` and then removing it one phase later causes churn across fix plans.

**Domain Logic Placement:** Place domain logic operating on `workflow_core` types in `workflow_core` from the initial implementation. Logic written in the CLI binary and later migrated to `workflow_core` causes churn (BFS `downstream_tasks` pattern, v2/v4/v5).

## Project Structure

```
castep_workflow_framework/
├── workflow_core/           # Layer 1: Foundation
│   ├── src/
│   │   ├── workflow.rs
│   │   ├── task.rs
│   │   ├── state.rs
│   │   ├── dag.rs
│   │   ├── error.rs
│   │   ├── process.rs
│   │   ├── prelude.rs       # (Phase 5B)
│   │   └── lib.rs
│   ├── tests/
│   └── Cargo.toml
│
├── workflow_utils/          # Layer 2: Generic Utilities
│   ├── src/
│   │   ├── executor.rs
│   │   ├── files.rs
│   │   ├── monitoring.rs
│   │   ├── runner.rs        # SystemProcessRunner, QueuedRunner
│   │   ├── prelude.rs       # (Phase 5B)
│   │   └── lib.rs
│   ├── tests/
│   └── Cargo.toml
│
├── workflow-cli/            # CLI Binary
│   ├── src/main.rs
│   └── Cargo.toml
│
├── examples/                # Layer 3: Example Projects
│   ├── hubbard_u_sweep/     # Direct mode reference impl
│   │   ├── src/main.rs
│   │   ├── seeds/
│   │   └── Cargo.toml
│   └── hubbard_u_sweep_slurm/  # Phase 5A: SLURM production sweep
│       ├── src/
│       │   ├── main.rs
│       │   ├── config.rs
│       │   └── job_script.rs
│       ├── seeds/
│       └── Cargo.toml
│
└── plans/                   # Phase plans
    └── phase-5/
```

## Implementation Status

### Phases 1–2: Complete ✅ (2026-04-08 to 2026-04-10)

**Phase 1.1: workflow_utils** — TaskExecutor, files module, MonitoringHook
**Phase 1.2: workflow_core** — Workflow (DAG), Task (closure), petgraph sort, JSON state
**Phase 1.3: Integration** — hubbard_u_sweep example, integration tests, resume bug fixed
**Phase 2.1** — castep-cell-io wired into hubbard_u_sweep
**Phase 2.2** — tracing logging, PeriodicHookManager, task timing, tokio removed

### Phase 3: Complete ✅ (2026-04-15)

- `StateStore` trait + `JsonStateStore` with atomic writes (write-to-temp + rename)
- `load_raw()` for read-only CLI inspection (no crash-recovery resets)
- `WorkflowError` `#[non_exhaustive]` enum with `thiserror`
- `run()` returns `Result<WorkflowSummary>`
- `ExecutionMode::Direct` with per-task timeout; `Queued` stub
- OS signal handling: SIGTERM/SIGINT via `signal-hook`, re-registers each `run()`
- `workflow-cli` binary: `status`, `inspect`, `retry` subcommands
- `Task` gains `setup`/`collect` closure fields; `TaskClosure` type alias
- `anyhow` removed from `workflow_core`; `TaskStatus` re-exported from crate root
- End-to-end resume + timeout integration tests

### Phase 4: Complete ✅ (2026-04-20)

- Log persistence for task stdout/stderr (`with_log_dir`)
- `HookTrigger::Periodic` background thread manager
- `ExecutionMode::Queued` fully implemented via `QueuedRunner` (SLURM/PBS)
- `TaskSuccessors`: successor graph persisted in `JsonStateStore` (`task_successors` field, `#[serde(default)]`)
- `set_task_graph()` on `StateStore` trait (default no-op)
- `downstream_of()` BFS on state for graph-aware retry
- `SystemProcessRunner::default()` via derive
- CLI `retry` skips already-successful downstream tasks

### Phase 5A: Production SLURM Sweep ✅ (2026-04-22)

- New workspace member: `examples/hubbard_u_sweep_slurm/`
- `clap` CLI with env-var config (`CASTEP_SLURM_ACCOUNT`, `CASTEP_SLURM_PARTITION`, etc.)
- First end-to-end SLURM production sweep using `ExecutionMode::Queued`
- Job script generation (`job.sh`) per task; collect closure for output validation
- `--dry-run` flag support
- Implementation plan: `plans/phase-5/phase5a_implementation.toml`

### Phase 5B: API Ergonomics ✅ (2026-04-23)

- `ExecutionMode::direct(cmd, &[args])` convenience constructor
- `workflow_core::prelude` module: re-exports all commonly used types
- `workflow_utils::prelude` module: re-exports all commonly used types from both crates; used in Layer 3 binaries with `use workflow_utils::prelude::*`
- `run_default(&mut workflow, &mut state)` in `workflow_utils`: eliminates repeated `Arc` wiring in binaries
- `downstream_of<S: AsRef<str>>` generic signature (callers pass `&[&str]` without allocating)
- Whitespace cleanup (workflow-cli)
- `init_default_logging()` exposed in `workflow_core::lib`; inlined format args throughout
- Full ARCHITECTURE.md + ARCHITECTURE_STATUS.md update (this round)

## Dependencies

### workflow_core

```toml
[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
thiserror = { workspace = true }
petgraph = { workspace = true }
tracing = { workspace = true }
signal-hook = { workspace = true }

[features]
default-logging = ["dep:tracing-subscriber"]
```

Note: `anyhow` is **not** a dependency of `workflow_core` or `workflow_utils` — both are library crates and use `WorkflowError` directly.

### workflow_utils

```toml
[dependencies]
workflow_core = { path = "../workflow_core" }
nix = { version = "0.29", features = ["process", "signal"] }
```

### Example projects

```toml
[dependencies]
workflow_core = { path = "../../workflow_core", features = ["default-logging"] }
workflow_utils = { path = "../../workflow_utils" }
castep-cell-fmt = "0.1.0"
castep-cell-io = "0.4.0"
anyhow = { workspace = true }  # anyhow fine in binary/example crates (Layer 3)
clap = { workspace = true }    # hubbard_u_sweep_slurm only
```

## Advantages of This Design

### 1. Simplicity

No traits beyond I/O boundaries, no adapters, no boilerplate. Three concepts: Workflow, Task, utils.

### 2. Flexibility

Layer 3 full control. Use any parser lib (castep-cell-io, castep-cell-fmt, vasp-io, etc). Mix different software in same workflow.

### 3. Composability

Utils independent, use what you need. Easy create helpers. Can build domain libs on top.

### 4. Testability

Each layer independently testable. No mocking needed (utils = simple functions). Easy integration tests.

### 5. Extensibility

New software: just use parser lib. New utils: add functions to Layer 2. Domain helpers: create new lib. New scheduler: implement `QueuedSubmitter`.

### 6. Performance

No trait dispatch overhead in hot path. Closures can inline. Parallel exec where possible.

### 7. Type Safety

Full compile-time checking via parser lib builders. `WorkflowError` `#[non_exhaustive]`. Clear error msgs.

## Conclusion

Utilities-based arch simpler, more flexible, easier maintain than adapter design. By killing unnecessary abstractions + giving Layer 3 full control, we create framework that's both powerful + easy use.

**Key Insight:** With Rust's type system + parser libs with builders + `StateStore` as the one justified I/O trait, simple utils are sufficient for production HPC workflow orchestration.