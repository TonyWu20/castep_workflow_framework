# Phase 1 Implementation Plan: workflow_core Revision

## Context

The current `workflow_core` implementation was created before the architecture rethink documented in:
- `LAYER2_RETHINK.md` - First-principles analysis eliminating the adapter pattern
- `FINAL_ARCHITECTURE_DESIGN.md` - Target utilities-based architecture
- `RUST_API_DESIGN_PLAN.md` - Three-layer design with Rust-first approach

**Current State:**
- Existing code uses TOML-driven, trait-based architecture (ExecutorFactory + Executor traits)
- Has solid DAG execution (petgraph), state persistence (SQLite), and scheduler
- Implements factory pattern for executor dispatch
- Static parameters via HashMap<String, toml::Value>

**Target State:**
- Utilities-based architecture (no traits in Layer 2)
- Rust-first: users write Rust code with closures
- Task contains execution closure with full control
- Generic utilities for file I/O and process execution
- Integration with castep-cell-io v0.4.0 builders

**Key Decision:** We need to **revise** the current implementation to align with the target architecture, not evolve it. The trait-based adapter pattern should be replaced with utilities.

## Architecture Decision

### What to Keep (Salvageable)
1. **DAG algorithms** - petgraph-based dependency resolution is solid
2. **State persistence concepts** - SQLite-backed resume capability works well
3. **Parallelism control** - Per-executor caps and dependency blocking
4. **Timeout handling** - Wall-time limits and cancellation

### What to Replace
1. **Executor/ExecutorFactory traits** → Generic TaskExecutor utility
2. **TOML workflow definitions** → Rust-first Task::new(closure)
3. **Static parameters** → Closures with castep-cell-io builders
4. **Registry dispatch** → Direct utility usage
5. **Scheduler as separate entity** → Integrate into Workflow

### Target Architecture

```
Layer 3: User Project Crate
  ↓ writes Rust code using
Layer 2: workflow_utils (NEW)
  - TaskExecutor: generic process execution
  - files module: generic file I/O
  - MonitoringHook: external command hooks
  ↓ used by
Layer 1: workflow_core (REVISED)
  - Workflow: DAG container + orchestration
  - Task: execution unit with closure
  - Execution engine: dependency resolution, parallel execution
  - State management: serialization/resume
```

## Implementation Plan

### Phase 1.1: Create workflow_utils (Layer 2) - Days 1-2

**New crate:** `workflow_utils/`

**Files to create:**
1. `workflow_utils/Cargo.toml`
2. `workflow_utils/src/lib.rs` - Public API exports
3. `workflow_utils/src/executor.rs` - TaskExecutor implementation
4. `workflow_utils/src/files.rs` - File I/O utilities
5. `workflow_utils/src/monitoring.rs` - MonitoringHook system

**TaskExecutor API:**
```rust
pub struct TaskExecutor {
    workdir: PathBuf,
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl TaskExecutor {
    pub fn new(workdir: impl Into<PathBuf>) -> Self;
    pub fn command(mut self, cmd: impl Into<String>) -> Self;
    pub fn arg(mut self, arg: impl Into<String>) -> Self;
    pub fn args(mut self, args: Vec<String>) -> Self;
    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self;
    
    // Blocking execution
    pub fn execute(&self) -> Result<ExecutionResult>;
    
    // Background execution (for async/parallel)
    pub fn spawn(&self) -> Result<ExecutionHandle>;
}

pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}
```

**files module API:**
```rust
pub fn read_file(path: impl AsRef<Path>) -> Result<String>;
pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()>;
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()>;
pub fn create_dir(path: impl AsRef<Path>) -> Result<()>;
pub fn remove_dir(path: impl AsRef<Path>) -> Result<()>;
```

**MonitoringHook API:**
```rust
pub struct MonitoringHook {
    pub name: String,
    pub command: String,
    pub trigger: HookTrigger,
}

pub enum HookTrigger {
    OnStart,
    OnComplete,
    OnFailure,
    Periodic { interval_secs: u64 },
}

impl MonitoringHook {
    pub fn execute(&self, context: &HookContext) -> Result<HookResult>;
}
```

**Unit Tests:**
- `tests/executor_tests.rs` - Test command execution, args, env vars
- `tests/files_tests.rs` - Test file I/O operations
- `tests/monitoring_tests.rs` - Test hook execution and triggers

### Phase 1.2: Revise workflow_core (Layer 1) - Days 3-5

**Files to modify/create:**
1. `workflow_core/src/workflow.rs` - NEW: Workflow struct with DAG execution
2. `workflow_core/src/task.rs` - NEW: Task struct with closure
3. `workflow_core/src/state.rs` - MODIFY: Adapt for new architecture
4. `workflow_core/src/dag.rs` - NEW: Extract DAG logic from pipeline.rs
5. `workflow_core/src/lib.rs` - MODIFY: Export new API

**Files to deprecate/remove:**
- `workflow_core/src/executor.rs` - Remove trait definitions
- `workflow_core/src/executors/` - Remove trait implementations
- `workflow_core/src/schema.rs` - Remove TOML parsing
- `workflow_core/src/pipeline.rs` - Extract reusable parts to dag.rs
- `workflow_core/src/scheduler.rs` - Integrate logic into Workflow

**Workflow API:**
```rust
pub struct Workflow {
    name: String,
    tasks: HashMap<String, Task>,
    dag: DAG,
    state: WorkflowState,
}

impl Workflow {
    pub fn new(name: impl Into<String>) -> Result<Self>;
    pub fn add_task(&mut self, task: Task) -> Result<()>;
    pub fn run(&mut self) -> Result<()>;
    pub fn resume(name: impl Into<String>) -> Result<Self>;
    pub fn dry_run(&self) -> Result<WorkflowSummary>;
}
```

**Task API:**
```rust
pub struct Task {
    id: String,
    dependencies: Vec<String>,
    execute_fn: Box<dyn Fn() -> Result<ExecutionResult> + Send + Sync>,
}

impl Task {
    pub fn new<F>(id: impl Into<String>, execute_fn: F) -> Self
    where F: Fn() -> Result<ExecutionResult> + Send + Sync + 'static;
    
    pub fn depends_on(mut self, task_id: impl Into<String>) -> Self;
}
```

**DAG module (extracted from pipeline.rs):**
```rust
pub struct DAG {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl DAG {
    pub fn new() -> Self;
    pub fn add_node(&mut self, id: String) -> Result<()>;
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<()>;
    pub fn topological_order(&self) -> Result<Vec<String>>;
    pub fn ready_tasks(&self, completed: &HashSet<String>) -> Vec<String>;
}
```

**State management (adapted from state.rs):**
```rust
pub struct WorkflowState {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub task_states: HashMap<String, TaskStatus>,
}

pub enum TaskStatus {
    Pending,
    Running,
    Completed { exit_code: i32, duration: Duration },
    Failed { error: String },
}

impl WorkflowState {
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()>;
    pub fn load(path: impl AsRef<Path>) -> Result<Self>;
}
```

**Unit Tests:**
- `tests/workflow_tests.rs` - Test workflow creation, task addition
- `tests/dag_tests.rs` - Test DAG construction, topological sort, cycle detection
- `tests/state_tests.rs` - Test state persistence and resume
- `tests/execution_tests.rs` - Test task execution with mocks

### Phase 1.3: Integration & Examples - Days 6-7

**Example project:** `examples/hubbard_u_sweep/`

```rust
use workflow_core::{Workflow, Task};
use workflow_utils::{TaskExecutor, files};
use castep_cell_io::{CellDocument, cell::species::hubbard_u::*, cell::species::Species};
use anyhow::Result;

fn main() -> Result<()> {
    let mut workflow = Workflow::new("hubbard_u_sweep")?;
    
    for u in vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{}", u);
        let workdir = format!("runs/U{}", u);
        
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
            
            // 4. Write files
            files::write_file(
                format!("{}/ZnO.cell", workdir),
                &cell_doc.to_string()?
            )?;
            files::write_file(
                format!("{}/ZnO.param", workdir),
                &param_doc.to_string()?
            )?;
            
            // 5. Execute
            TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .execute()
        });
        
        workflow.add_task(task)?;
    }
    
    workflow.run()?;
    Ok(())
}
```

**Integration tests:**
- `tests/integration/hubbard_u_sweep.rs` - End-to-end test with mock castep
- `tests/integration/resume.rs` - Test workflow resume after interruption
- `tests/integration/dependencies.rs` - Test complex dependency chains

## Handling Key Concerns

### 1. Closure Serialization for Resume

**Problem:** Closures can't be serialized.

**Solution:** 
- State only tracks task completion status (Pending/Running/Completed/Failed)
- On resume, user re-runs `main()`, which reconstructs all closures
- Workflow checks state and skips completed tasks
- This is the same pattern as Make/Ninja

```rust
// On resume
let mut workflow = Workflow::resume("hubbard_u_sweep")?;
// State loaded from disk, completed tasks marked
workflow.run()?;  // Only runs incomplete tasks
```

### 2. MonitoringHook Integration

**Implementation:**
- Hooks stored in Task (optional)
- Workflow executor calls hooks at appropriate triggers
- Hooks run as separate processes via TaskExecutor

```rust
let task = Task::new("scf", || { /* ... */ })
    .add_monitor(MonitoringHook {
        name: "convergence".into(),
        command: "./scripts/check_convergence.sh".into(),
        trigger: HookTrigger::OnComplete,
    });
```

### 3. Preserving Scheduler Strengths

**What to keep:**
- Parallelism control (max concurrent tasks)
- Dependency blocking (wait for deps)
- Timeout handling (wall-time limits)
- Cancellation support (graceful shutdown)

**How to integrate:**
- Move scheduler logic into `Workflow::run()`
- Use tokio for async execution
- Maintain state persistence at each cycle

### 4. Local vs. SLURM Execution

**Without traits, how do we support different backends?**

**Solution:** TaskExecutor handles both internally:
```rust
pub enum ExecutorBackend {
    Local { max_parallel: usize },
    Slurm { partition: String, ntasks: u32 },
}

impl TaskExecutor {
    pub fn with_backend(mut self, backend: ExecutorBackend) -> Self;
}
```

Or simpler: separate utilities
```rust
// workflow_utils/src/local.rs
pub fn execute_local(cmd: &str, args: &[String], workdir: &Path) -> Result<ExecutionResult>;

// workflow_utils/src/slurm.rs
pub fn submit_slurm(cmd: &str, args: &[String], workdir: &Path, config: SlurmConfig) -> Result<JobId>;
pub fn poll_slurm(job_id: JobId) -> Result<JobStatus>;
```

User chooses in their closure:
```rust
Task::new("task", || {
    // For local
    TaskExecutor::new("workdir").command("castep").execute()?;
    
    // For SLURM
    let job_id = slurm::submit_slurm("castep", &["ZnO"], "workdir", slurm_config)?;
    slurm::wait_for_completion(job_id)?;
})
```

## Unit Test Plan

### workflow_utils Tests

**executor_tests.rs:**
- `test_executor_basic()` - Simple command execution
- `test_executor_with_args()` - Command with arguments
- `test_executor_with_env()` - Environment variables
- `test_executor_exit_code()` - Non-zero exit codes
- `test_executor_stdout_stderr()` - Capture output
- `test_executor_spawn()` - Background execution
- `test_executor_timeout()` - Timeout handling

**files_tests.rs:**
- `test_read_write_file()` - Basic file I/O
- `test_copy_file()` - File copying
- `test_create_dir()` - Directory creation
- `test_remove_dir()` - Directory removal
- `test_file_not_found()` - Error handling

**monitoring_tests.rs:**
- `test_hook_on_complete()` - OnComplete trigger
- `test_hook_on_failure()` - OnFailure trigger
- `test_hook_periodic()` - Periodic trigger
- `test_hook_context()` - Context passing

### workflow_core Tests

**workflow_tests.rs:**
- `test_workflow_creation()` - Create empty workflow
- `test_add_task()` - Add tasks to workflow
- `test_duplicate_task_id()` - Error on duplicate IDs
- `test_workflow_run_empty()` - Run empty workflow

**dag_tests.rs:**
- `test_dag_add_node()` - Add nodes to DAG
- `test_dag_add_edge()` - Add dependency edges
- `test_dag_topological_sort()` - Correct ordering
- `test_dag_cycle_detection()` - Detect cycles
- `test_dag_missing_dependency()` - Error on missing deps
- `test_dag_ready_tasks()` - Identify ready tasks

**state_tests.rs:**
- `test_state_save_load()` - Persistence round-trip
- `test_state_task_status()` - Track task states
- `test_state_resume()` - Resume from saved state

**execution_tests.rs:**
- `test_task_execution()` - Execute single task
- `test_task_with_dependencies()` - Respect dependencies
- `test_parallel_execution()` - Run independent tasks in parallel
- `test_task_failure()` - Handle task failures
- `test_task_timeout()` - Timeout enforcement

### Integration Tests

**integration/hubbard_u_sweep.rs:**
- End-to-end HubbardU sweep with mock CASTEP
- Verify all tasks execute in correct order
- Check output files created correctly

**integration/resume.rs:**
- Start workflow, interrupt mid-execution
- Resume and verify only incomplete tasks run
- Check state persistence across restarts

**integration/dependencies.rs:**
- Complex dependency graph (diamond, chain, parallel)
- Verify correct execution order
- Test failure propagation

## Timeline

- **Days 1-2:** workflow_utils implementation + tests
- **Days 3-5:** workflow_core revision + tests
- **Days 6-7:** Integration tests + examples
- **Days 8-9:** Documentation + polish
- **Day 10:** Review + final testing

**Total: 10 days**

## Critical Files

**To create:**
- `/Users/tony/programming/castep_workflow_framework/workflow_utils/src/executor.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_utils/src/files.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_utils/src/monitoring.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/workflow.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/task.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/dag.rs`

**To modify:**
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/state.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/lib.rs`
- `/Users/tony/programming/castep_workflow_framework/Cargo.toml` (add workflow_utils)

**To remove:**
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/executor.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/executors/`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/schema.rs`
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/pipeline.rs` (extract to dag.rs)
- `/Users/tony/programming/castep_workflow_framework/workflow_core/src/scheduler.rs` (integrate into workflow.rs)

## Verification

**After implementation:**

1. **Run unit tests:** `cargo test --all`
2. **Run integration tests:** `cargo test --test '*'`
3. **Run example:** `cargo run --example hubbard_u_sweep`
4. **Test resume:** Interrupt example mid-run, restart, verify completion
5. **Check state files:** Verify state persistence format
6. **Review API:** Ensure matches FINAL_ARCHITECTURE_DESIGN.md

## Notes

- This plan aligns with FINAL_ARCHITECTURE_DESIGN.md (utilities-based, no traits)
- Preserves proven algorithms (DAG, state persistence) while changing architecture
- Integrates with castep-cell-io v0.4.0 builders
- Maintains resume capability through state tracking
- Supports both local and distributed execution via utilities

---

## Additional Considerations from Rust Architect Review

### Reusable Components from Existing Code

**From pipeline.rs (petgraph DAG):**
- `Pipeline::from_tasks()` - DAG construction logic
- Topological sorting algorithm
- Cycle detection
- Can extract to new `dag.rs` module

**From scheduler.rs:**
- 7-step execution loop pattern
- Parallelism control with active task counting
- Timeout enforcement via timestamps
- Cancellation token integration
- Can integrate into `Workflow::run()`

**From state.rs (SQLite persistence):**
- `StateDb` with async SQLite
- `TaskRecord` and `TaskState` enums
- Can simplify to JSON/TOML for Phase 1, keep SQLite for Phase 2

**From executors/local.rs:**
- Process spawning with tokio
- PID tracking and sentinel files
- Exit code capture
- Can adapt for `TaskExecutor::spawn()`

### Migration Strategy

**Step 1: Create workflow_utils alongside existing code**
- New crate, no conflicts with existing code
- Can be tested independently

**Step 2: Create new API in workflow_core**
- Add `workflow.rs`, `task.rs`, `dag.rs` as new modules
- Keep existing modules temporarily
- Update `lib.rs` to export both old and new APIs

**Step 3: Deprecate old API**
- Mark `executor.rs`, `schema.rs`, `scheduler.rs` as deprecated
- Update examples to use new API
- Remove old code in Phase 2

**Step 4: Update adapters**
- `castep_adapter` and `lammps_adapter` become example projects
- Show how to use new API with castep-cell-io

### Async vs Sync Considerations

**Decision: Start with sync, add async in Phase 2**

Rationale:
- Closures in `Task::new()` are simpler as sync
- Can use `std::thread` for parallelism initially
- Async adds complexity (Send + Sync + 'static bounds)
- Existing scheduler is async, but can be adapted

**Phase 1 approach:**
```rust
// Sync closure
Task::new("task", || {
    files::create_dir("workdir")?;
    TaskExecutor::new("workdir").command("castep").execute()
})

// Workflow::run() uses threads for parallelism
impl Workflow {
    pub fn run(&mut self) -> Result<()> {
        // Use thread pool for parallel execution
        let pool = ThreadPool::new(max_parallel);
        // ... execute tasks
    }
}
```

**Phase 2 can add:**
```rust
// Async closure
Task::new_async("task", || async {
    files::create_dir("workdir").await?;
    TaskExecutor::new("workdir").command("castep").execute().await
})
```

### Error Handling Strategy

**Use anyhow::Result throughout:**
- Simple error propagation with `?`
- Context can be added with `.context()`
- Errors bubble up to `Workflow::run()`

**Task failure handling:**
```rust
pub enum FailureMode {
    FailFast,           // Stop on first failure
    ContinueOnError,    // Run all independent tasks
    SkipDependents,     // Skip tasks depending on failed task
}

impl Workflow {
    pub fn with_failure_mode(mut self, mode: FailureMode) -> Self;
}
```

### State Persistence Format

**Phase 1: JSON files**
```json
{
  "version": "1.0",
  "workflow_name": "hubbard_u_sweep",
  "created_at": "2026-04-06T12:00:00Z",
  "last_updated": "2026-04-06T12:05:00Z",
  "tasks": {
    "scf_U0": {
      "status": "Completed",
      "exit_code": 0,
      "duration_secs": 120
    },
    "scf_U1": {
      "status": "Running",
      "started_at": "2026-04-06T12:04:00Z"
    },
    "scf_U2": {
      "status": "Pending"
    }
  }
}
```

**Phase 2: Can migrate to SQLite for better concurrency**

---

## Final Implementation Order

1. **Day 1:** workflow_utils/executor.rs + tests
2. **Day 2:** workflow_utils/files.rs + monitoring.rs + tests
3. **Day 3:** workflow_core/dag.rs (extract from pipeline.rs) + tests
4. **Day 4:** workflow_core/task.rs + tests
5. **Day 5:** workflow_core/workflow.rs (basic structure) + tests
6. **Day 6:** workflow_core/workflow.rs (execution engine) + tests
7. **Day 7:** workflow_core/state.rs (JSON persistence) + tests
8. **Day 8:** Integration tests + example
9. **Day 9:** Documentation + API polish
10. **Day 10:** Final review + testing

---

**Plan Status:** READY FOR REVIEW
