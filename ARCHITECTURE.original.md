# Workflow Framework Architecture

**Version:** 2.1 (Utilities-Based)  
**Last Updated:** 2026-04-08  
**Status:** Phase 1 Complete (1.1 + 1.2 + 1.3)

## Executive Summary

This framework uses a **utilities-based architecture** where Layer 2 provides generic execution utilities, not software-specific adapters. All software-specific logic belongs in parser libraries (castep-cell-io) or project crates (Layer 3).

**Key Decision:** After first-principles analysis, we eliminated the adapter pattern. Layer 2 is purely generic utilities with no traits, no adapters, no software-specific code.

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

1. **Input Preparation**: Read seed files, apply modifications, write to workdir
2. **Execution**: Run binary in workdir with arguments
3. **Monitoring**: Check output, run external scripts, detect completion

### What's Truly Software-Specific?

| Concern                       | Where It Belongs                          |
| ----------------------------- | ----------------------------------------- |
| File format (.cell syntax)    | castep-cell-io (parser library)           |
| Document structure (HubbardU) | castep-cell-io (parser library)           |
| Modifications (set U value)   | castep-cell-io builders                   |
| Binary name ("castep")        | Layer 3 (project knows what it's running) |
| Command arguments             | Layer 3 (project-specific)                |
| Output parsing                | External scripts (via monitoring hooks)   |

**Answer: Nothing!** All software-specific logic is already in parser libraries or Layer 3.

### What's Truly Generic?

- File I/O (read/write any file)
- Process execution (run any command)
- Workdir management (create, clean up)
- Monitoring hooks (run external commands)
- Status tracking (running, completed, failed)

**Conclusion:** Layer 2 should provide generic utilities, not software-specific adapters.

## Layer 1: workflow_core (Foundation)

### Purpose

Provide generic workflow orchestration: DAG management, dependency resolution, parallel execution, state persistence.

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

1. **DAG Execution**: Topological sort, parallel execution where possible
2. **Dependency Resolution**: Automatic ordering based on `depends_on`
3. **State Persistence**: Save/resume workflow state (task status, not closures)
4. **Error Handling**: Fail-fast or continue-on-error modes
5. **Progress Tracking**: Real-time status updates

### What Layer 1 Does NOT Do

- File I/O (that's Layer 2)
- Process execution (that's Layer 2)
- Software-specific logic (that's parser libraries or Layer 3)
- Input file preparation (that's Layer 3)

## Layer 2: workflow_utils (Generic Utilities)

### Purpose

Provide generic utilities for file I/O, process execution, and monitoring. No software-specific code.

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

Functions are re-exported flat at the crate root (the `files` module is private):

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

- Parse CASTEP files (that's castep-cell-io)
- Know about HubbardU or any CASTEP concepts (that's Layer 3)
- Implement traits or adapters (just simple utilities)
- Make decisions about workflow structure (that's Layer 1 or Layer 3)

## Layer 3: Project Crates (Domain-Specific)

### Purpose

User's research-specific workflow logic. Uses parser libraries directly, uses Layer 2 utilities for I/O and execution.

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

Users can create reusable domain-specific builders as separate libraries:

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

1. CastepAdapter duplicates logic that Layer 3 already has
2. TaskAdapter trait forces a specific abstraction
3. Adding new software requires new adapter implementation
4. Layer 3 loses control over file preparation details

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

1. Simpler: No trait, no adapters, just utilities
2. More flexible: Layer 3 has full control
3. Less code: No adapter boilerplate
4. Easier to extend: Just use different parser library
5. Clearer separation: Layer 2 is truly generic

## Design Principles

1. **Separation of Concerns**
   - Layer 1: Orchestration only
   - Layer 2: Generic utilities only
   - Layer 3: Domain logic only
   - Parser libraries: Format-specific only

2. **No Premature Abstraction**
   - Don't create traits unless multiple implementations exist
   - Don't create adapters unless they add value
   - Keep it simple until complexity is justified

3. **User Control**
   - Layer 3 has full control over workflow
   - No hidden magic, no implicit behavior
   - Explicit is better than implicit

4. **Composability Over Inheritance**
   - Use functions, not class hierarchies
   - Compose utilities, don't extend adapters
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

- TaskExecutor with blocking and background execution
- files module with read/write/copy/create_dir/remove_dir (flat re-exports at crate root)
- MonitoringHook with trigger system

**Phase 1.2: workflow_core** ✅

- Workflow with DAG execution and `bon` builder (`Workflow::builder().name(...).build()`)
- `max_parallel` configurable via builder (defaults to `available_parallelism`)
- Task with closure-based execution
- DAG with petgraph topological sort
- WorkflowState with JSON persistence
- Dependency failure propagation

**Phase 1.3: Integration & Examples** ✅

- Resume bug fixed: `Running` tasks reset to `Pending` on state load
- `examples/hubbard_u_sweep`: Layer 3 reference implementation
- Integration tests: sweep pattern, resume semantics, DAG ordering/failure propagation

### Phase 2: Planned 📋

**Examples and Documentation**

- Full HubbardU sweep with castep-cell-io builders (castep-cell-io integration pending)
- Convergence test example
- Comprehensive documentation

## Dependencies

### workflow_core

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
petgraph = "0.8"
workflow_utils = { path = "../workflow_utils" }
```

### workflow_utils

```toml
[dependencies]
anyhow = "1.0"
nix = { version = "0.29", features = ["process", "signal"] }
```

### Example projects

```toml
[dependencies]
workflow_core = { path = "../../workflow_core" }
workflow_utils = { path = "../../workflow_utils" }
castep-cell-io = { path = "../../../castep-cell-io/castep_cell_io" }
anyhow = "1.0"
```

## Advantages of This Design

### 1. Simplicity

- No traits, no adapters, no boilerplate
- Just three simple concepts: Workflow, Task, utilities
- Easy to understand and maintain

### 2. Flexibility

- Layer 3 has full control over workflow logic
- Can use any parser library (castep-cell-io, vasp-io, etc.)
- Can mix different software in same workflow

### 3. Composability

- Utilities are independent, use what you need
- Easy to create helper functions
- Can build domain-specific libraries on top

### 4. Testability

- Each layer independently testable
- No mocking needed (utilities are simple functions)
- Easy to write integration tests

### 5. Extensibility

- Adding new software: just use its parser library
- Adding new utilities: just add functions to Layer 2
- Adding domain-specific helpers: create new library

### 6. Performance

- No trait dispatch overhead
- Closures can be inlined
- Parallel execution where possible

### 7. Type Safety

- Full compile-time checking via parser library builders
- No runtime type dispatch
- Clear error messages

## Migration from Old Design

If you have existing code using the adapter pattern:

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

The utilities-based architecture is simpler, more flexible, and easier to maintain than the adapter-based design. By eliminating unnecessary abstractions and giving Layer 3 full control, we create a framework that's both powerful and easy to use.

**Key Insight:** With Rust's type system and parser libraries with builders, we don't need adapters. Simple utilities are sufficient.
