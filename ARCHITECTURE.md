# Workflow Framework Architecture

**Version:** 2.2 (Utilities-Based)  
**Last Updated:** 2026-04-10  
**Status:** Phase 2.2 Complete (1.1 + 1.2 + 1.3 + 2.1 + 2.2)

## Executive Summary

Utilities-based arch: Layer 2 = generic exec utils, NOT software adapters. Software logic → parser libs (castep-cell-io) or project crates (Layer 3).

**Key Decision:** After first-principles analysis, killed adapter pattern. Layer 2 = pure generic utils. No traits, no adapters, no software code.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Project Crates (Domain-Specific)              │
│  - User's research-specific workflow logic              │
│  - Uses parser libraries directly (castep-cell-io)      │
│  - Uses Layer 2 utilities for I/O and execution         │
│  - Full control over workflow construction              │
│  - Examples: hubbard_u_sweep, convergence_test          │
└────────────────────────┬────────────────────────────────┘
                         │ uses
┌────────────────────────▼────────────────────────────────┐
│  Layer 2: workflow_utils (Generic Utilities)            │
│  - TaskExecutor: Generic process execution              │
│  - files module: Generic file I/O                       │
│  - MonitoringHook: External monitoring integration      │
│  - NO software-specific code                            │
│  - NO traits, NO adapters                               │
└────────────────────────┬────────────────────────────────┘
                         │ uses
┌────────────────────────▼────────────────────────────────┐
│  Layer 1: workflow_core (Foundation)                    │
│  - Workflow: DAG container and orchestration            │
│  - Task: Execution unit with closure                    │
│  - Execution engine: Dependency resolution, parallel    │
│  - State management: Serialization, resume              │
└────────────────────────┬────────────────────────────────┘
                         │ uses
┌────────────────────────▼────────────────────────────────┐
│  Parser Libraries (Software-Specific)                   │
│  - castep-cell-io: CASTEP file format                   │
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

Generic workflow orchestration: DAG mgmt, dependency resolution, parallel exec, state persistence.

### Core Types

```rust
/// Workflow container with DAG execution
pub struct Workflow {
    name: String,
    tasks: HashMap<String, Task>,
    state_path: PathBuf,
    max_parallel: usize,
}

impl Workflow {
    /// Create new workflow via builder (bon)
    /// Required: name. Optional: state_dir (default "."), max_parallel (default num_cpus)
    pub fn builder() -> WorkflowBuilder;

    /// Add task to workflow
    pub fn add_task(&mut self, task: Task) -> Result<()>;

    /// Execute workflow (resolves dependencies, runs in parallel where possible)
    pub fn run(&mut self) -> Result<()>;

    /// Resume interrupted workflow; re-register tasks via add_task after calling this
    pub fn resume(name: impl Into<String>, state_dir: impl Into<PathBuf>) -> Result<Self>;

    /// Dry-run: returns task execution order without executing
    pub fn dry_run(&self) -> Result<Vec<String>>;
}

// Builder usage:
// Workflow::builder().name("name".to_string()).build()?
// Workflow::builder().name("name".to_string()).state_dir("./runs".into()).max_parallel(8).build()?

/// Task: execution unit with closure
pub struct Task {
    id: String,
    dependencies: Vec<String>,
    status: TaskStatus,

    // Execution closure - contains all task logic
    execute_fn: Arc<dyn Fn() -> Result<()> + Send + Sync>,
}

impl Task {
    /// Create task with execution closure
    pub fn new<F>(id: impl Into<String>, execute_fn: F) -> Self
    where
        F: Fn() -> Result<()> + Send + Sync + 'static;

    /// Add dependency on another task
    pub fn depends_on(mut self, task_id: impl Into<String>) -> Self;

    /// Execute the task
    pub(crate) fn execute(&self) -> Result<()>;
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

/// Workflow state for serialization/resume
#[derive(Serialize, Deserialize)]
pub struct WorkflowState {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub tasks: HashMap<String, TaskStatus>,
}
```

### Key Features

1. **DAG Execution**: Topo sort, parallel where possible
2. **Dependency Resolution**: Auto ordering via `depends_on`
3. **State Persistence**: Save/resume workflow state (task status, not closures)
4. **Error Handling**: Fail-fast or continue-on-error
5. **Progress Tracking**: Real-time status updates

### What Layer 1 Does NOT Do

- File I/O (Layer 2)
- Process exec (Layer 2)
- Software logic (parser libs or Layer 3)
- Input prep (Layer 3)

## Layer 2: workflow_utils (Generic Utilities)

### Purpose

Generic utils for file I/O, process exec, monitoring. No software code.

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
    /// Create executor for a workdir
    pub fn new(workdir: impl Into<PathBuf>) -> Self;

    /// Set command to execute
    pub fn command(mut self, cmd: impl Into<String>) -> Self;

    /// Add single argument
    pub fn arg(mut self, arg: impl Into<String>) -> Self;

    /// Add multiple arguments
    pub fn args(mut self, args: Vec<String>) -> Self;

    /// Set environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self;

    /// Execute command (blocking)
    pub fn execute(&self) -> Result<ExecutionResult>;

    /// Execute in background
    pub fn spawn(&self) -> Result<ExecutionHandle>;
}

/// Execution result
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

/// Handle for background execution
pub struct ExecutionHandle {
    pid: u32,
    workdir: PathBuf,
}

impl ExecutionHandle {
    /// Wait for completion
    pub fn wait(self) -> Result<ExecutionResult>;

    /// Check if still running
    pub fn is_running(&self) -> bool;

    /// Kill the process
    pub fn terminate(&self) -> Result<()>;
}
```

### files: Generic File I/O

Re-exported flat at crate root (`files` module private):

```rust
use workflow_utils::{read_file, write_file, copy_file, create_dir, remove_dir, exists};

pub fn read_file(path: impl AsRef<Path>) -> Result<String>;
pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()>;
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()>;
pub fn create_dir(path: impl AsRef<Path>) -> Result<()>;
pub fn remove_dir(path: impl AsRef<Path>) -> Result<()>;
pub fn exists(path: impl AsRef<Path>) -> bool;
```

### MonitoringHook: External Monitoring

```rust
/// Hook for external monitoring/parsing commands
#[derive(Debug, Clone)]
pub struct MonitoringHook {
    pub name: String,
    pub command: String,
    pub trigger: HookTrigger,
}

#[derive(Debug, Clone)]
pub enum HookTrigger {
    /// Run before task starts
    OnStart,

    /// Run after task completes
    OnComplete,

    /// Run when task fails
    OnFailure,

    /// Run periodically during execution
    Periodic { interval_secs: u64 },
}

impl MonitoringHook {
    /// Create new hook
    pub fn new(name: &str, command: &str, trigger: HookTrigger) -> Self;

    /// Execute the hook
    pub fn execute(&self, context: &HookContext) -> Result<HookResult>;
}

/// Context passed to monitoring hooks
pub struct HookContext {
    pub task_id: String,
    pub workdir: PathBuf,
    pub status: TaskStatus,
}

/// Result from hook execution
pub struct HookResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
```

### What Layer 2 Does NOT Do

- Parse CASTEP files (castep-cell-io)
- Know HubbardU or CASTEP concepts (Layer 3)
- Implement traits/adapters (just utils)
- Decide workflow structure (Layer 1 or Layer 3)

## Layer 3: Project Crates (Domain-Specific)

### Purpose

User's research workflow logic. Uses parser libs directly, uses Layer 2 utils for I/O/exec.

### Example 1: Direct Low-Level Usage

```rust
use workflow_core::{Workflow, Task};
use workflow_utils::{TaskExecutor, create_dir, write_file, MonitoringHook, HookTrigger};
use castep_cell_io::prelude::*;
use anyhow::Result;

fn main() -> Result<()> {
    let mut workflow = Workflow::builder()
        .name("hubbard_u_sweep".to_string())
        .build()?;

    let u_values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    for u in u_values {
        let task_id = format!("scf_U{}", u);
        let workdir = format!("runs/U{}", u);

        // Create task with closure containing all logic
        let task = Task::new(&task_id, move || {
            // 1. Create workdir
            create_dir(&workdir)?;

            // 2. Read seed files
            let mut cell_doc = CellDocument::from_file("seeds/ZnO.cell")?;
            let param_doc = ParamDocument::from_file("seeds/ZnO.param")?;

            // 3. Modify using castep-cell-io v0.4.0 builders
            let atom_u = AtomHubbardU::builder()
                .species(Species::Symbol("Zn".to_string()))
                .orbitals(vec![OrbitalU::D(u)])
                .build();

            cell_doc.hubbard_u = Some(HubbardU::builder()
                .unit(HubbardUUnit::ElectronVolt)
                .atom_u_values(vec![atom_u])
                .build());

            // 4. Write modified files
            write_file(
                format!("{}/ZnO.cell", workdir),
                &cell_doc.to_string()?
            )?;
            write_file(
                format!("{}/ZnO.param", workdir),
                &param_doc.to_string()?
            )?;

            // 5. Execute CASTEP
            TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .execute()?;

            Ok(())
        });

        workflow.add_task(task)?;
    }

    // Add dependent DOS tasks
    for u in u_values {
        let scf_task_id = format!("scf_U{}", u);
        let dos_task_id = format!("dos_U{}", u);
        let workdir = format!("runs/U{}", u);

        let task = Task::new(&dos_task_id, move || {
            // DOS calculation uses output from SCF
            TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .env("CASTEP_TASK", "dos")
                .execute()?;
            Ok(())
        }).depends_on(scf_task_id);

        workflow.add_task(task)?;
    }

    workflow.run()?;
    Ok(())
}
```

### Example 2: Helper Functions (User-Defined in Layer 3)

```rust
use workflow_core::{Workflow, Task};
use workflow_utils::{TaskExecutor, create_dir, write_file};
use castep_cell_io::prelude::*;
use anyhow::Result;

/// User-defined helper for CASTEP tasks
struct CastepTaskBuilder {
    seed_dir: PathBuf,
    seed_name: String,
}

impl CastepTaskBuilder {
    fn new(seed_dir: &str, seed_name: &str) -> Self {
        Self {
            seed_dir: PathBuf::from(seed_dir),
            seed_name: seed_name.to_string(),
        }
    }

    /// Create task with cell modification
    fn create_task<F>(
        &self,
        task_id: &str,
        workdir: &str,
        modify_cell: F,
    ) -> Result<Task>
    where
        F: Fn(&mut CellDocument) -> Result<()> + Send + Sync + 'static,
    {
        let seed_dir = self.seed_dir.clone();
        let seed_name = self.seed_name.clone();
        let workdir = workdir.to_string();
        let task_id = task_id.to_string();

        Ok(Task::new(&task_id, move || {
            // Setup
            create_dir(&workdir)?;

            // Read seeds
            let mut cell_doc = CellDocument::from_file(
                format!("{}/{}.cell", seed_dir.display(), seed_name)
            )?;
            let param_doc = ParamDocument::from_file(
                format!("{}/{}.param", seed_dir.display(), seed_name)
            )?;

            // Apply modification
            modify_cell(&mut cell_doc)?;

            // Write files
            write_file(
                format!("{}/{}.cell", workdir, task_id),
                &cell_doc.to_string()?
            )?;
            write_file(
                format!("{}/{}.param", workdir, task_id),
                &param_doc.to_string()?
            )?;

            // Execute
            TaskExecutor::new(&workdir)
                .command("castep")
                .arg(&task_id)
                .execute()?;

            Ok(())
        }))
    }
}

fn main() -> Result<()> {
    let mut workflow = Workflow::builder().name("hubbard_u_sweep".to_string()).build()?;
    let castep = CastepTaskBuilder::new("./seeds", "ZnO");

    for u in vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task = castep.create_task(
            &format!("scf_U{}", u),
            &format!("runs/U{}", u),
            move |cell_doc| {
                let atom_u = AtomHubbardU::builder()
                    .species(Species::Symbol("Zn".to_string()))
                    .orbitals(vec![OrbitalU::D(u)])
                    .build();

                cell_doc.hubbard_u = Some(HubbardU::builder()
                    .unit(HubbardUUnit::ElectronVolt)
                    .atom_u_values(vec![atom_u])
                    .build());
                Ok(())
            },
        )?;

        workflow.add_task(task)?;
    }

    workflow.run()?;
    Ok(())
}
```

### Example 3: Domain-Specific Builder (Optional Library)

Users can create reusable domain builders as separate libs:

```rust
// In a separate crate: castep_workflow_helpers

use workflow_core::{Workflow, Task};
use workflow_utils::{TaskExecutor, create_dir, write_file};
use castep_cell_io::prelude::*;
use anyhow::Result;

/// Domain-specific builder for HubbardU sweeps
pub struct HubbardUSweep {
    system_name: String,
    seed_dir: PathBuf,
    species: String,
    orbital: char,
    u_values: Vec<f64>,
}

impl HubbardUSweep {
    pub fn new(system_name: &str) -> Self {
        Self {
            system_name: system_name.to_string(),
            seed_dir: PathBuf::from("./seeds"),
            species: String::new(),
            orbital: 'd',
            u_values: vec![],
        }
    }

    pub fn seed_dir(mut self, path: &str) -> Self {
        self.seed_dir = PathBuf::from(path);
        self
    }

    pub fn element(mut self, species: &str) -> Self {
        self.species = species.to_string();
        self
    }

    pub fn orbital(mut self, orbital: char) -> Self {
        self.orbital = orbital;
        self
    }

    pub fn values(mut self, values: Vec<f64>) -> Self {
        self.u_values = values;
        self
    }

    pub fn build_workflow(self) -> Result<Workflow> {
        let mut workflow = Workflow::builder().name(self.system_name.clone()).build()?;

        for u in self.u_values {
            let task_id = format!("scf_U{}", u);
            let workdir = format!("runs/U{}", u);
            let seed_dir = self.seed_dir.clone();
            let system_name = self.system_name.clone();
            let species = self.species.clone();
            let orbital = self.orbital;

            let task = Task::new(&task_id, move || {
                create_dir(&workdir)?;

                let mut cell_doc = CellDocument::from_file(
                    format!("{}/{}.cell", seed_dir.display(), system_name)
                )?;
                let param_doc = ParamDocument::from_file(
                    format!("{}/{}.param", seed_dir.display(), system_name)
                )?;

                let atom_u = AtomHubbardU::builder()
                    .species(Species::Symbol(species.clone()))
                    .orbitals(vec![match orbital {
                        'd' => OrbitalU::D(u),
                        'f' => OrbitalU::F(u),
                        _ => return Err(anyhow!("Invalid orbital")),
                    }])
                    .build();

                cell_doc.hubbard_u = Some(HubbardU::builder()
                    .unit(HubbardUUnit::ElectronVolt)
                    .atom_u_values(vec![atom_u])
                    .build());

                write_file(
                    format!("{}/{}.cell", workdir, task_id),
                    &cell_doc.to_string()?
                )?;
                write_file(
                    format!("{}/{}.param", workdir, task_id),
                    &param_doc.to_string()?
                )?;

                TaskExecutor::new(&workdir)
                    .command("castep")
                    .arg(&task_id)
                    .execute()?;

                Ok(())
            });

            workflow.add_task(task)?;
        }

        Ok(workflow)
    }
}

// Usage in user's project:
fn main() -> Result<()> {
    let workflow = HubbardUSweep::new("ZnO")
        .seed_dir("./seeds")
        .element("Zn")
        .orbital('d')
        .values(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
        .build_workflow()?;

    workflow.run()?;
    Ok(())
}
```

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
  - Uses castep-cell-io directly
  - Uses Layer 2 utilities for I/O and execution
  - Full control over workflow logic
  ↓ uses
Layer 2: Execution Utilities (workflow_utils)
  - TaskExecutor (generic process execution)
  - files module (generic I/O)
  - MonitoringHook (generic hooks)
  ↓ uses
Layer 1: workflow_core
  - Workflow/Task (just DAG + orchestration)
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

## Project Structure

```
castep_workflow_framework/
├── workflow_core/           # Layer 1: Foundation
│   ├── src/
│   │   ├── workflow.rs
│   │   ├── task.rs
│   │   ├── state.rs
│   │   ├── dag.rs
│   │   └── lib.rs
│   ├── tests/
│   └── Cargo.toml
│
├── workflow_utils/          # Layer 2: Generic Utilities
│   ├── src/
│   │   ├── executor.rs
│   │   ├── files.rs
│   │   ├── monitoring.rs
│   │   └── lib.rs
│   ├── tests/
│   └── Cargo.toml
│
├── examples/                # Layer 3: Example Projects
│   ├── hubbard_u_sweep/
│   │   ├── src/main.rs
│   │   ├── seeds/
│   │   └── Cargo.toml
│   ├── convergence_test/
│   └── custom_workflow/
│
└── castep_workflow_helpers/ # Optional: Domain-Specific Helpers
    ├── src/
    │   ├── hubbard_u.rs
    │   ├── convergence.rs
    │   └── lib.rs
    └── Cargo.toml
```

## Implementation Status

### Phase 1: Complete ✅ (2026-04-08)

**Phase 1.1: workflow_utils** ✅

- TaskExecutor w/ blocking + background exec
- files module w/ read/write/copy/create_dir/remove_dir (flat re-exports at crate root)
- MonitoringHook w/ trigger system

**Phase 1.2: workflow_core** ✅

- Workflow w/ DAG exec + `bon` builder (`Workflow::builder().name(...).build()`)
- `max_parallel` configurable via builder (defaults to `available_parallelism`)
- Task w/ closure-based exec
- DAG w/ petgraph topo sort
- WorkflowState w/ JSON persistence
- Dependency failure propagation

**Phase 1.3: Integration & Examples** ✅

- Resume bug fixed: `Running` tasks reset to `Pending` on state load
- `examples/hubbard_u_sweep`: Layer 3 reference impl
- Integration tests: sweep pattern, resume semantics, DAG ordering/failure propagation

### Phase 2.1: Complete ✅

- castep-cell-io wired into `hubbard_u_sweep` example
- Execution reports tracked in `execution_reports/`

### Phase 2.2: Complete ✅ (2026-04-10)

**Production Readiness — Logging & Periodic Monitoring**

- `tracing` integrated into `workflow_core` (structured log events at `debug`/`info`/`error` levels)
- `init_default_logging()` helper (behind `default-logging` feature flag, uses `tracing-subscriber`)
- `PeriodicHookManager`: spawns background threads for `HookTrigger::Periodic` hooks, stops them cleanly on task completion/failure
- Task-level `monitors()` builder method added to `Task`
- Workflow-level timing: per-task duration logged on complete/fail; total workflow summary on finish
- `capture_task_error_context` kept generic (no CASTEP-specific filenames in Layer 1)
- `tokio` dep removed from `workflow_utils` (now pure std-thread)
- 36/36 tests pass; Clippy: 0 warnings

### Phase 3: Planned 📋

- Convergence test example
- Comprehensive docs

## Dependencies

### workflow_core

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1"
petgraph = "0.8"
tracing = "0.1"
workflow_utils = { path = "../workflow_utils" }

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
workflow_core = { path = "../../workflow_core" }
workflow_utils = { path = "../../workflow_utils" }
castep-cell-io = { path = "../../../castep-cell-io/castep_cell_io" }
anyhow = "1.0"  # anyhow is fine in binary/example crates (Layer 3)
```

## Advantages of This Design

### 1. Simplicity

No traits, no adapters, no boilerplate. Just 3 concepts: Workflow, Task, utils. Easy understand/maintain.

### 2. Flexibility

Layer 3 full control. Use any parser lib (castep-cell-io, vasp-io, etc). Mix different software in same workflow.

### 3. Composability

Utils independent, use what you need. Easy create helpers. Can build domain libs on top.

### 4. Testability

Each layer independently testable. No mocking needed (utils = simple functions). Easy integration tests.

### 5. Extensibility

New software: just use parser lib. New utils: add functions to Layer 2. Domain helpers: create new lib.

### 6. Performance

No trait dispatch overhead. Closures can inline. Parallel exec where possible.

### 7. Type Safety

Full compile-time checking via parser lib builders. No runtime type dispatch. Clear error msgs.

## Migration from Old Design

Existing code using adapter pattern:

**Before (adapter-based):**

```rust
let adapter = CastepAdapter::new("seeds", "ZnO");
let task = Task::builder()
    .adapter(adapter)
    .build()?;
```

**After (utilities-based):**

```rust
let task = Task::new("task_id", || {
    // Direct control over everything
    let cell_doc = CellDocument::from_file("seeds/ZnO.cell")?;
    // ... modify ...
    write_file("runs/task/ZnO.cell", &cell_doc.to_string()?)?;
    TaskExecutor::new("runs/task").command("castep").arg("ZnO").execute()?;
    Ok(())
});
```

## Conclusion

Utilities-based arch simpler, more flexible, easier maintain than adapter design. By killing unnecessary abstractions + giving Layer 3 full control, we create framework that's both powerful + easy use.

**Key Insight:** W/ Rust's type system + parser libs w/ builders, no need adapters. Simple utils sufficient.