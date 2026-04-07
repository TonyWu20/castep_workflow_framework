# Final Architecture Design: Three-Layer Workflow Framework

## Executive Summary

After first-principles analysis, we've eliminated the adapter pattern. The framework uses a **utilities-based architecture** where Layer 2 provides generic execution utilities, not software-specific adapters.

**Key Decision:** All software-specific logic belongs in parser libraries (castep-cell-io) or project crates (Layer 3). Layer 2 is purely generic utilities.

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

## Layer 1: workflow_core (Foundation)

### Purpose
Provide generic workflow orchestration: DAG management, dependency resolution, parallel execution, state persistence.

### Core Types

```rust
/// Workflow container with DAG execution
pub struct Workflow {
    name: String,
    tasks: HashMap<String, Task>,
    state: WorkflowState,
}

impl Workflow {
    /// Create new workflow
    pub fn new(name: impl Into<String>) -> Result<Self>;
    
    /// Add task to workflow
    pub fn add_task(&mut self, task: Task) -> Result<()>;
    
    /// Execute workflow (resolves dependencies, runs in parallel where possible)
    pub fn run(&mut self) -> Result<()>;
    
    /// Resume interrupted workflow
    pub fn resume(name: impl Into<String>) -> Result<Self>;
    
    /// Dry-run: show task graph without executing
    pub fn dry_run(&self) -> Result<WorkflowSummary>;
}

/// Task: execution unit with closure
pub struct Task {
    id: String,
    dependencies: Vec<String>,
    status: TaskStatus,
    
    // Execution closure - contains all task logic
    execute_fn: Box<dyn Fn() -> Result<ExecutionResult> + Send + Sync>,
}

impl Task {
    /// Create task with execution closure
    pub fn new<F>(id: impl Into<String>, execute_fn: F) -> Self
    where
        F: Fn() -> Result<ExecutionResult> + Send + Sync + 'static;
    
    /// Add dependency on another task
    pub fn depends_on(mut self, task_id: impl Into<String>) -> Self;
    
    /// Execute the task
    pub(crate) fn execute(&self) -> Result<ExecutionResult>;
}

/// Task execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

/// Task status for state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed { error: String },
}

/// Workflow state for serialization/resume
#[derive(Serialize, Deserialize)]
pub struct WorkflowState {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub task_states: HashMap<String, TaskStatus>,
    pub execution_log: Vec<ExecutionEvent>,
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
    monitoring_hooks: Vec<MonitoringHook>,
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
    
    /// Add monitoring hook
    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self;
    
    /// Execute command (blocking)
    pub fn execute(&self) -> Result<ExecutionResult>;
    
    /// Execute in background
    pub fn spawn(&self) -> Result<ExecutionHandle>;
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
    pub fn kill(&self) -> Result<()>;
}
```

### files: Generic File I/O

```rust
pub mod files {
    use std::path::Path;
    use anyhow::Result;
    
    /// Read file to string
    pub fn read_file(path: impl AsRef<Path>) -> Result<String>;
    
    /// Write string to file
    pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()>;
    
    /// Copy file
    pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()>;
    
    /// Create directory (including parents)
    pub fn create_dir(path: impl AsRef<Path>) -> Result<()>;
    
    /// Remove directory recursively
    pub fn remove_dir(path: impl AsRef<Path>) -> Result<()>;
    
    /// Check if file exists
    pub fn exists(path: impl AsRef<Path>) -> bool;
    
    /// List files in directory
    pub fn list_files(dir: impl AsRef<Path>) -> Result<Vec<PathBuf>>;
}
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
    
    /// Run periodically during execution
    Periodic { interval_secs: u64 },
    
    /// Run when specific pattern appears in output
    OnPattern { pattern: String },
}

impl MonitoringHook {
    /// Create new hook
    pub fn new(name: &str, command: &str, trigger: HookTrigger) -> Self;
    
    /// Execute the hook
    pub fn execute(&self, workdir: &Path) -> Result<HookResult>;
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
use workflow_core::{Workflow, Task, ExecutionResult};
use workflow_utils::{TaskExecutor, files, MonitoringHook, HookTrigger};
use castep_cell_io::prelude::*;
use anyhow::Result;

fn main() -> Result<()> {
    let mut workflow = Workflow::new("hubbard_u_sweep")?;
    
    let u_values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    
    for u in u_values {
        let task_id = format!("scf_U{}", u);
        let workdir = format!("runs/U{}", u);
        
        // Create task with closure containing all logic
        let task = Task::new(&task_id, move || {
            // 1. Create workdir
            files::create_dir(&workdir)?;
            
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
            files::write_file(
                format!("{}/ZnO.cell", workdir),
                &cell_doc.to_string()?
            )?;
            files::write_file(
                format!("{}/ZnO.param", workdir),
                &param_doc.to_string()?
            )?;
            
            // 5. Execute CASTEP
            let result = TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .add_monitor(MonitoringHook::new(
                    "convergence",
                    "./scripts/check_convergence.sh",
                    HookTrigger::OnComplete,
                ))
                .execute()?;
            
            Ok(result)
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
                .execute()
        }).depends_on(scf_task_id);
        
        workflow.add_task(task)?;
    }
    
    workflow.run()?;
    Ok(())
}
```

### Example 2: Helper Functions (User-Defined in Layer 3)

```rust
use workflow_core::{Workflow, Task, ExecutionResult};
use workflow_utils::{TaskExecutor, files};
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
            files::create_dir(&workdir)?;
            
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
            files::write_file(
                format!("{}/{}.cell", workdir, task_id),
                &cell_doc.to_string()?
            )?;
            files::write_file(
                format!("{}/{}.param", workdir, task_id),
                &param_doc.to_string()?
            )?;
            
            // Execute
            TaskExecutor::new(&workdir)
                .command("castep")
                .arg(&task_id)
                .execute()
        }))
    }
}

fn main() -> Result<()> {
    let mut workflow = Workflow::new("hubbard_u_sweep")?;
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
use workflow_utils::{TaskExecutor, files};
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
        let mut workflow = Workflow::new(&self.system_name)?;
        
        for u in self.u_values {
            let task_id = format!("scf_U{}", u);
            let workdir = format!("runs/U{}", u);
            let seed_dir = self.seed_dir.clone();
            let system_name = self.system_name.clone();
            let species = self.species.clone();
            let orbital = self.orbital;
            
            let task = Task::new(&task_id, move || {
                files::create_dir(&workdir)?;
                
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
                
                files::write_file(
                    format!("{}/{}.cell", workdir, task_id),
                    &cell_doc.to_string()?
                )?;
                files::write_file(
                    format!("{}/{}.param", workdir, task_id),
                    &param_doc.to_string()?
                )?;
                
                TaskExecutor::new(&workdir)
                    .command("castep")
                    .arg(&task_id)
                    .execute()
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

## Implementation Plan

### Phase 0: Prerequisite - Builders in castep-cell-io ✅ COMPLETED

**Status:** Completed in castep-cell-io v0.4.0 (2026-04-06)

**See:** `/Users/tony/programming/castep_workflow_framework/CASTEP_CELL_IO_V0.4_INTEGRATION.md`

**Priority blocks:**
1. `HubbardU` and `AtomHubbardU`
2. `PositionsFrac` and `AtomPosition`
3. `SpeciesPot`
4. `KpointsList`

**Timeline:** 2-3 days

### Phase 1: Layer 1 - workflow_core

**Files to create:**
1. `workflow_core/src/workflow.rs` - Workflow struct with DAG execution
2. `workflow_core/src/task.rs` - Task struct with closure
3. `workflow_core/src/state.rs` - WorkflowState and serialization
4. `workflow_core/src/executor.rs` - DAG execution engine
5. `workflow_core/src/lib.rs` - Public API

**Key features:**
- Workflow container with task DAG
- Task with execution closure
- Dependency resolution (topological sort)
- Parallel execution where possible
- State persistence (save/resume)
- Error handling (fail-fast, continue-on-error)

**Tests:**
- DAG resolution with complex dependencies
- Parallel execution
- State serialization/deserialization
- Error propagation

**Timeline:** 4-5 days

### Phase 2: Layer 2 - workflow_utils

**Files to create:**
1. `workflow_utils/src/executor.rs` - TaskExecutor
2. `workflow_utils/src/files.rs` - File I/O utilities
3. `workflow_utils/src/monitoring.rs` - MonitoringHook
4. `workflow_utils/src/lib.rs` - Public API

**Key features:**
- TaskExecutor for process execution
- files module for I/O operations
- MonitoringHook for external commands
- ExecutionHandle for background processes

**Tests:**
- Process execution (blocking and background)
- File operations (read, write, copy)
- Monitoring hook execution
- Error handling

**Timeline:** 2-3 days

### Phase 3: Examples and Documentation

**Files to create:**
1. `examples/hubbard_u_sweep/` - Complete HubbardU sweep
2. `examples/convergence_test/` - K-point convergence
3. `examples/custom_workflow/` - Complex multi-stage workflow
4. `examples/helpers/` - Reusable helper functions
5. `workflow_core/README.md` - Architecture and API docs
6. `workflow_utils/README.md` - Utilities documentation

**Timeline:** 1 week

### Phase 4: Optional - Domain-Specific Helpers Library

**Files to create:**
1. `castep_workflow_helpers/src/hubbard_u.rs` - HubbardUSweep
2. `castep_workflow_helpers/src/convergence.rs` - ConvergenceTest
3. `castep_workflow_helpers/src/lib.rs` - Public API

**Timeline:** 1-2 weeks (optional, can be done by users)

## Project Structure

```
castep_workflow_framework/
├── workflow_core/           # Layer 1: Foundation
│   ├── src/
│   │   ├── workflow.rs
│   │   ├── task.rs
│   │   ├── state.rs
│   │   ├── executor.rs
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

## Dependencies

### workflow_core
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
```

### workflow_utils
```toml
[dependencies]
anyhow = "1.0"
thiserror = "1.0"
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

## Comparison with Adapter-Based Design

| Aspect | Adapter-Based | Utilities-Based |
|--------|---------------|-----------------|
| Layer 2 abstraction | TaskAdapter trait | Simple utilities |
| Software-specific code | In adapters | In parser libraries |
| Boilerplate | High (trait impl) | Low (just functions) |
| Flexibility | Limited by trait | Full control |
| Complexity | Medium-High | Low |
| Extensibility | New trait impl | Use different library |
| Learning curve | Steeper | Gentler |

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
    files::write_file("runs/task/ZnO.cell", &cell_doc.to_string()?)?;
    TaskExecutor::new("runs/task").command("castep").arg("ZnO").execute()
});
```

## Conclusion

The utilities-based architecture is simpler, more flexible, and easier to maintain than the adapter-based design. By eliminating unnecessary abstractions and giving Layer 3 full control, we create a framework that's both powerful and easy to use.

**Key Insight:** With Rust's type system and parser libraries with builders, we don't need adapters. Simple utilities are sufficient.
