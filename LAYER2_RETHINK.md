# Layer 2 Rethink: From Adapters to Execution Utilities

## First Principles Analysis

### What Does a Workflow Actually Need?

1. **Input Preparation**: Read seed files, apply modifications, write to workdir
2. **Execution**: Run binary in workdir with arguments
3. **Monitoring**: Check output, run external scripts, detect completion

### Current Design Problem

The "adapter" abstraction assumes we need software-specific wrappers:
- `CastepAdapter` implements `TaskAdapter`
- `VaspAdapter` implements `TaskAdapter`
- Each adapter knows how to prepare inputs for its software

**But this is wrong!** With the Rust approach:
- Layer 3 already uses castep-cell-io builders directly
- Layer 3 constructs `CellDocument` and `ParamDocument`
- Layer 3 knows the file format (it's using the parser library)

So what does `CastepAdapter` actually do that's not already handled?

### What's Truly Software-Specific?

| Concern | Where It Belongs |
|---------|------------------|
| File format (.cell syntax) | castep-cell-io (parser library) |
| Document structure (HubbardU) | castep-cell-io (parser library) |
| Modifications (set U value) | castep-cell-io builders |
| Binary name ("castep") | Layer 3 (project knows what it's running) |
| Command arguments | Layer 3 (project-specific) |
| Output parsing | External scripts (via monitoring hooks) |

**Answer: Nothing!** All software-specific logic is already in parser libraries or Layer 3.

### What's Truly Generic?

- File I/O (read/write any file)
- Process execution (run any command)
- Workdir management (create, clean up)
- Monitoring hooks (run external commands)
- Status tracking (running, completed, failed)

## Proposed New Architecture

### Layer 2: Execution Utilities (NOT Adapters)

```rust
// No trait, no adapters - just utilities

/// Generic task execution
pub struct TaskExecutor {
    workdir: PathBuf,
    command: String,
    args: Vec<String>,
    monitoring_hooks: Vec<MonitoringHook>,
}

impl TaskExecutor {
    pub fn new(workdir: impl Into<PathBuf>) -> Self;
    
    pub fn command(mut self, cmd: impl Into<String>) -> Self;
    pub fn arg(mut self, arg: impl Into<String>) -> Self;
    pub fn args(mut self, args: Vec<String>) -> Self;
    
    pub fn add_monitor(mut self, hook: MonitoringHook) -> Self;
    
    /// Execute the command (blocking)
    pub fn execute(&self) -> Result<ExecutionResult>;
    
    /// Execute in background
    pub fn spawn(&self) -> Result<ExecutionHandle>;
}

/// Generic file utilities
pub mod files {
    /// Read any file to string
    pub fn read_file(path: impl AsRef<Path>) -> Result<String>;
    
    /// Write string to file
    pub fn write_file(path: impl AsRef<Path>, content: &str) -> Result<()>;
    
    /// Copy file
    pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()>;
    
    /// Create workdir
    pub fn create_workdir(path: impl AsRef<Path>) -> Result<()>;
}

/// Monitoring hook (unchanged)
pub struct MonitoringHook {
    pub name: String,
    pub command: String,
    pub trigger: HookTrigger,
}
```

### Layer 3: Project Crate (Full Control)

```rust
use workflow_core::{Workflow, Task};
use workflow_utils::{TaskExecutor, files};  // Layer 2 utilities
use castep_cell_io::prelude::*;
use anyhow::Result;

fn main() -> Result<()> {
    let mut workflow = Workflow::new("hubbard_u_sweep")?;
    
    for u in vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{}", u);
        let workdir = format!("runs/U{}", u);
        
        // Create task with closure that does EVERYTHING
        let task = Task::new(&task_id, move || {
            // 1. Create workdir
            files::create_workdir(&workdir)?;
            
            // 2. Read seed files
            let mut cell_doc = CellDocument::from_file("seeds/ZnO.cell")?;
            let param_doc = ParamDocument::from_file("seeds/ZnO.param")?;
            
            // 3. Modify using builders
            cell_doc.hubbard_u = Some(HubbardU::builder()
                .unit(HubbardUUnit::ElectronVolt)
                .add_atom_u(AtomHubbardU::builder()
                    .species("Zn")
                    .add_orbital(OrbitalU::D(u))
                    .build()?)
                .build()?);
            
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
            let result = TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .add_monitor(MonitoringHook {
                    name: "convergence".into(),
                    command: "./scripts/check_convergence.sh".into(),
                    trigger: HookTrigger::OnComplete,
                })
                .execute()?;
            
            Ok(result)
        });
        
        workflow.add_task(task)?;
    }
    
    workflow.run()?;
    Ok(())
}
```

### Domain-Specific Helper (Optional, in Layer 3)

```rust
// Users can create their own helpers in Layer 3
pub struct CastepTask {
    seed_dir: PathBuf,
    seed_name: String,
}

impl CastepTask {
    pub fn new(seed_dir: &str, seed_name: &str) -> Self {
        Self {
            seed_dir: PathBuf::from(seed_dir),
            seed_name: seed_name.to_string(),
        }
    }
    
    /// Helper to create a task with cell modification
    pub fn create_task<F>(
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
            files::create_workdir(&workdir)?;
            
            let mut cell_doc = CellDocument::from_file(
                format!("{}/{}.cell", seed_dir.display(), seed_name)
            )?;
            let param_doc = ParamDocument::from_file(
                format!("{}/{}.param", seed_dir.display(), seed_name)
            )?;
            
            modify_cell(&mut cell_doc)?;
            
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
        }))
    }
}

// Usage:
fn main() -> Result<()> {
    let mut workflow = Workflow::new("hubbard_u_sweep")?;
    let castep = CastepTask::new("./seeds", "ZnO");
    
    for u in vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task = castep.create_task(
            &format!("scf_U{}", u),
            &format!("runs/U{}", u),
            move |cell_doc| {
                cell_doc.hubbard_u = Some(HubbardU::builder()
                    .unit(HubbardUUnit::ElectronVolt)
                    .add_atom_u(AtomHubbardU::builder()
                        .species("Zn")
                        .add_orbital(OrbitalU::D(u))
                        .build()?)
                    .build()?);
                Ok(())
            },
        )?;
        
        workflow.add_task(task)?;
    }
    
    workflow.run()?;
    Ok(())
}
```

## Comparison

### Old Design (Adapter-Based)

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

### New Design (Utilities-Based)

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

## Answer to Original Question

**Are software-specific adapters necessary?**

**No.** With the Rust approach where Layer 3 uses parser libraries directly:
- All software-specific logic is in parser libraries (castep-cell-io, vasp-io)
- Layer 3 knows what software it's running (it's project-specific)
- Layer 2 should provide generic utilities, not software-specific adapters

**What's the correct abstraction level for Layer 2?**

**Generic execution utilities:**
- File I/O (read, write, copy)
- Process execution (run command, monitor, hooks)
- Workdir management (create, clean)
- No software-specific code at all

**This balances:**
- Development difficulty: Much simpler, no trait implementations
- API friendliness: Layer 3 gets simple utilities, full control
- Flexibility: Works with any software, any parser library
- Maintainability: Layer 2 is stable, rarely changes

## Recommendation

**Eliminate the adapter pattern entirely.** Rename Layer 2 to `workflow_utils` and provide:
1. `TaskExecutor` - generic process execution
2. `files` module - generic file I/O
3. `MonitoringHook` - generic monitoring

Let Layer 3 (project crates) handle all software-specific logic using parser libraries directly.
