# Phase 5A: Production Hubbard U Sweep on SLURM

**Date:** 2026-04-22
**Status:** Draft

## Context

The framework has all the mechanical pieces for real HPC work (DAG, SLURM/PBS submission, crash recovery, signal handling), but nobody has used it for a real calculation yet. The existing `hubbard_u_sweep` example runs locally via `ExecutionMode::Direct` — it does not test SLURM submission, log persistence, or collect closures.

**Goal:** Build a real production Hubbard U sweep project for ZnO on SLURM, validate the full execution path, and surface API friction that will inform Phase 5B improvements.

---

## New Workspace Member

**Path:** `examples/hubbard_u_sweep_slurm/`

Keep the existing `hubbard_u_sweep` example as the simple Direct-mode reference — don't modify it.

**Directory structure:**
```
examples/hubbard_u_sweep_slurm/
  Cargo.toml
  src/
    main.rs          # clap CLI entry point, workflow wiring
    config.rs        # SweepConfig struct (clap + env vars)
    job_script.rs    # job.sh SLURM script generation
  seeds/
    ZnO.cell         # copy from ../hubbard_u_sweep/seeds/
    ZnO.param        # copy from ../hubbard_u_sweep/seeds/
```

**Workspace Cargo.toml change** (`/Users/tony/programming/castep_workflow_framework/Cargo.toml`):
Add `"examples/hubbard_u_sweep_slurm"` to `[workspace.members]`.

**Crate Cargo.toml:**
```toml
[package]
name = "hubbard_u_sweep_slurm"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hubbard_u_sweep_slurm"
path = "src/main.rs"

[dependencies]
anyhow = { workspace = true }
clap = { workspace = true }
castep-cell-fmt = "0.1.0"
castep-cell-io = "0.4.0"
workflow_core = { path = "../../workflow_core", features = ["default-logging"] }
workflow_utils = { path = "../../workflow_utils" }
```

---

## Configuration: `src/config.rs`

SLURM configuration via `clap` with `#[arg(env = "...")]` for cluster-specific values.
Tony sets env vars once in `.bashrc`; per-sweep args override as needed.

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "hubbard_u_sweep_slurm")]
pub struct SweepConfig {
    /// SLURM account name
    #[arg(long, env = "SLURM_ACCOUNT")]
    pub account: String,

    /// SLURM partition
    #[arg(long, env = "SLURM_PARTITION", default_value = "standard")]
    pub partition: String,

    /// Number of MPI tasks (cores) per job
    #[arg(long, default_value_t = 16)]
    pub ntasks: u32,

    /// Walltime per job (HH:MM:SS)
    #[arg(long, default_value = "01:00:00")]
    pub walltime: String,

    /// Module load commands, comma-separated (e.g. "castep/24.1,intel/2024")
    #[arg(long, env = "CASTEP_MODULES", value_delimiter = ',')]
    pub modules: Vec<String>,

    /// CASTEP executable command (e.g. "castep.mpi" or "mpirun -np 16 castep.mpi")
    #[arg(long, env = "CASTEP_COMMAND", default_value = "castep.mpi")]
    pub castep_command: String,

    /// Seed name (CASTEP input file prefix, without extension)
    #[arg(long, default_value = "ZnO")]
    pub seed_name: String,

    /// U values to sweep, comma-separated (eV)
    #[arg(long, default_value = "0.0,1.0,2.0,3.0,4.0,5.0")]
    pub u_values: String,

    /// Maximum number of concurrent SLURM jobs
    #[arg(long, default_value_t = 4)]
    pub max_parallel: usize,

    /// Element to apply Hubbard U to
    #[arg(long, default_value = "Zn")]
    pub element: String,

    /// Orbital for Hubbard U: 'd' or 'f'
    #[arg(long, default_value = "d")]
    pub orbital: char,
}

impl SweepConfig {
    pub fn parse_u_values(&self) -> Vec<f64> {
        self.u_values
            .split(',')
            .filter_map(|s| s.trim().parse::<f64>().ok())
            .collect()
    }

    pub fn module_load_lines(&self) -> String {
        self.modules
            .iter()
            .map(|m| format!("module load {}", m))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
```

**Usage on cluster:**
```bash
# Set once in .bashrc:
export SLURM_ACCOUNT=my_project
export SLURM_PARTITION=standard
export CASTEP_MODULES=castep/24.1,intel/2024
export CASTEP_COMMAND="mpirun castep.mpi"

# Run:
hubbard_u_sweep_slurm --u-values "0.0,2.0,4.0,6.0" --ntasks 32 --walltime 02:00:00
```

---

## Job Script Generation: `src/job_script.rs`

`QueuedRunner::submit()` expects `workdir/job.sh` and passes `-o`/`-e` to `sbatch`
directly, so the script does not need `#SBATCH -o/-e` output directives.

```rust
use crate::config::SweepConfig;

pub fn generate_job_script(config: &SweepConfig, task_id: &str) -> String {
    format!(
        "#!/bin/bash\n\
         #SBATCH --job-name={task_id}\n\
         #SBATCH --account={account}\n\
         #SBATCH --partition={partition}\n\
         #SBATCH --ntasks={ntasks}\n\
         #SBATCH --time={walltime}\n\
         \n\
         {modules}\n\
         \n\
         {command} {seed}\n",
        task_id = task_id,
        account = config.account,
        partition = config.partition,
        ntasks = config.ntasks,
        walltime = config.walltime,
        modules = config.module_load_lines(),
        command = config.castep_command,
        seed = config.seed_name,
    )
}
```

---

## Workflow Wiring: `src/main.rs`

```rust
mod config;
mod job_script;

use anyhow::Result;
use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use castep_cell_io::CellDocument;
use clap::Parser;
use std::sync::Arc;
use workflow_core::state::JsonStateStore;
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_core::{HookExecutor, ProcessRunner, WorkflowError};
use workflow_utils::{create_dir, write_file, QueuedRunner, SchedulerKind, ShellHookExecutor, SystemProcessRunner};

use config::SweepConfig;
use job_script::generate_job_script;

fn main() -> Result<()> {
    workflow_core::init_default_logging();
    let config = SweepConfig::parse();
    let u_values = config.parse_u_values();

    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");
    let seed_name = config.seed_name.clone();

    let mut workflow = Workflow::new("hubbard_u_sweep_slurm")
        .with_max_parallel(config.max_parallel)?
        .with_log_dir("logs")
        .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));

    for u in &u_values {
        let u = *u;
        let task_id = format!("scf_U{:.1}", u);
        let workdir = std::path::PathBuf::from(format!("runs/U{:.1}", u));
        let seed_cell = seed_cell.to_owned();
        let seed_param = seed_param.to_owned();
        let element = config.element.clone();
        let orbital = config.orbital;
        let seed_name = seed_name.clone();
        let job_script = generate_job_script(&config, &task_id);

        let task = Task::new(&task_id, ExecutionMode::Queued)
            .workdir(workdir.clone())
            .setup(move |workdir| -> Result<(), WorkflowError> {
                create_dir(workdir)?;

                // Parse seed cell and inject HubbardU
                let mut cell_doc: CellDocument =
                    parse(&seed_cell).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;

                let orbital_u = match orbital {
                    'd' => OrbitalU::D(u),
                    'f' => OrbitalU::F(u),
                    c => return Err(WorkflowError::InvalidConfig(
                        format!("unsupported orbital '{}'", c),
                    )),
                };
                let atom_u = AtomHubbardU::builder()
                    .species(Species::Symbol(element.clone()))
                    .orbitals(vec![orbital_u])
                    .build();
                let hubbard_u = HubbardU::builder()
                    .unit(HubbardUUnit::ElectronVolt)
                    .atom_u_values(vec![atom_u])
                    .build();
                cell_doc.hubbard_u = Some(hubbard_u);

                let cell_text = to_string_many_spaced(&cell_doc.to_cell_file());
                write_file(workdir.join(format!("{}.cell", seed_name)), &cell_text)?;
                write_file(workdir.join(format!("{}.param", seed_name)), &seed_param)?;
                write_file(workdir.join("job.sh"), &job_script)?;
                Ok(())
            })
            .collect(move |workdir| -> Result<(), WorkflowError> {
                // Verify CASTEP completed successfully
                let castep_out = workdir.join(format!("{}.castep", seed_name));
                if !castep_out.exists() {
                    return Err(WorkflowError::InvalidConfig(format!(
                        "missing output: {}",
                        castep_out.display()
                    )));
                }
                let content = std::fs::read_to_string(&castep_out).map_err(WorkflowError::Io)?;
                if !content.contains("Total time") {
                    return Err(WorkflowError::InvalidConfig(
                        "CASTEP output appears incomplete (no 'Total time' marker)".into(),
                    ));
                }
                Ok(())
            });

        workflow.add_task(task)?;
    }

    let state_path = std::path::PathBuf::from(".hubbard_u_sweep_slurm.workflow.json");
    let mut state = JsonStateStore::new("hubbard_u_sweep_slurm", state_path);
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
    let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);

    let summary = workflow.run(&mut state, runner, executor)?;
    println!(
        "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
        summary.succeeded.len(),
        summary.failed.len(),
        summary.skipped.len(),
        summary.duration.as_secs_f64(),
    );
    Ok(())
}
```

**Key ergonomics note:** `write_file()` and `create_dir()` already return `Result<(), WorkflowError>`,
so `?` works directly — no `.map_err()` needed. The existing example's `.map_err()` calls are legacy.

---

## Verification

**Build check:**
```
cargo build -p hubbard_u_sweep_slurm
cargo clippy -p hubbard_u_sweep_slurm
```

**Local dry-run (before cluster):**
Run with a mock castep_command like `echo` and check:
- `runs/U0.0/`, `runs/U1.0/`, ... directories created
- Each workdir has `ZnO.cell` (with correct HubbardU block), `ZnO.param`, `job.sh`
- `job.sh` has correct `#SBATCH` directives

**On-cluster validation:**
- Jobs submit (`sbatch` returns job IDs)
- Polling detects completion via `squeue`
- State file shows all tasks `Completed`
- `workflow-cli status .hubbard_u_sweep_slurm.workflow.json` shows correct summary
- Resume works after Ctrl-C (interrupt and restart)
