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
use workflow_utils::{
    create_dir, read_file, write_file, QueuedRunner, SchedulerKind, ShellHookExecutor,
    SystemProcessRunner, JOB_SCRIPT_NAME,
};

use config::{parse_u_values, SweepConfig};
use job_script::generate_job_script;

/// Build a single Task for the given Hubbard U value.
fn build_one_task(
    config: &SweepConfig,
    u: f64,
    seed_cell: &str,
    seed_param: &str,
) -> Result<Task, WorkflowError> {
    let task_id = format!("scf_U{u:.1}");
    let workdir = std::path::PathBuf::from(format!("runs/U{u:.1}"));
    let seed_cell = seed_cell.to_owned();
    let seed_param = seed_param.to_owned();
    let element = config.element.clone();
    let orbital = config.orbital;
    let seed_name_setup = config.seed_name.clone();
    let seed_name_collect = config.seed_name.clone();
    let is_local = config.local;

    // Only generate job script for SLURM mode
    let job_script = if !is_local {
        Some(generate_job_script(config, &task_id, &config.seed_name))
    } else {
        None
    };

    let mode = if is_local {
        ExecutionMode::direct(&config.castep_command, &[&config.seed_name])
    } else {
        ExecutionMode::Queued
    };

    let task = Task::new(&task_id, mode)
        .workdir(workdir)
        .setup(move |workdir| -> Result<(), WorkflowError> {
            create_dir(workdir)?;

            // Parse seed cell and inject HubbardU
            let mut cell_doc: CellDocument =
                parse(&seed_cell).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;

            let orbital_u = match orbital {
                'd' => OrbitalU::D(u),
                'f' => OrbitalU::F(u),
                c => {
                    return Err(WorkflowError::InvalidConfig(format!(
                        "unsupported orbital '{c}'"
                    )))
                }
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
            write_file(
                workdir.join(format!("{seed_name_setup}.cell")),
                &cell_text,
            )?;
            write_file(
                workdir.join(format!("{seed_name_setup}.param")),
                &seed_param,
            )?;
            // Only write job script for SLURM mode
            if let Some(ref script) = job_script {
                write_file(workdir.join(JOB_SCRIPT_NAME), script)?;
            }
            Ok(())
        })
        .collect(move |workdir| -> Result<(), WorkflowError> {
            let castep_out = workdir.join(format!("{seed_name_collect}.castep"));
            if !castep_out.exists() {
                return Err(WorkflowError::InvalidConfig(format!(
                    "missing output: {}",
                    castep_out.display()
                )));
            }
            let content = read_file(&castep_out)?;
            if !content.contains("Total time") {
                return Err(WorkflowError::InvalidConfig(
                    "CASTEP output appears incomplete (no 'Total time' marker)".into(),
                ));
            }
            Ok(())
        });

    Ok(task)
}

/// Build all sweep tasks from the config.
fn build_sweep_tasks(config: &SweepConfig) -> Result<Vec<Task>, anyhow::Error> {
    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");
    let u_values = parse_u_values(&config.u_values).map_err(anyhow::Error::msg)?;

    u_values
        .into_iter()
        .map(|u| build_one_task(config, u, seed_cell, seed_param).map_err(Into::into))
        .collect()
}

fn main() -> Result<()> {
    workflow_core::init_default_logging().ok();
    let config = SweepConfig::parse();

    let tasks = build_sweep_tasks(&config)?;

    let mut workflow = Workflow::new("hubbard_u_sweep_slurm")
        .with_max_parallel(config.max_parallel)?
        .with_log_dir("logs");

    if !config.local {
        workflow = workflow.with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
    }

    for task in tasks {
        workflow.add_task(task)?;
    }

    // Dry-run mode: print topological order and exit
    if config.dry_run {
        let order = workflow.dry_run()?;
        println!("Dry-run topological order:");
        for task_id in &order {
            println!("  {task_id}");
        }
        return Ok(());
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

