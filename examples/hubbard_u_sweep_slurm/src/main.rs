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
    SystemProcessRunner,
};

use config::SweepConfig;
use job_script::generate_job_script;

fn main() -> Result<()> {
    workflow_core::init_default_logging().ok();
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
        let seed_name_setup = seed_name.clone();
        let seed_name_collect = seed_name.clone();
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
                    c => {
                        return Err(WorkflowError::InvalidConfig(format!(
                            "unsupported orbital '{}'",
                            c
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
                    workdir.join(format!("{}.cell", seed_name_setup)),
                    &cell_text,
                )?;
                write_file(
                    workdir.join(format!("{}.param", seed_name_setup)),
                    &seed_param,
                )?;
                write_file(workdir.join("job.sh"), &job_script)?;
                Ok(())
            })
            .collect(move |workdir| -> Result<(), WorkflowError> {
                // Verify CASTEP completed successfully.
                // Note: WorkflowError::InvalidConfig is reused here because no
                // CollectFailed variant exists yet. Migrate when Phase 5B adds one.
                let castep_out = workdir.join(format!("{}.castep", seed_name_collect));
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

        workflow.add_task(task)?;
    }

    // Dry-run mode: print topological order and exit
    if config.dry_run {
        let order = workflow.dry_run()?;
        println!("Dry-run topological order:");
        for task_id in &order {
            println!("  {}", task_id);
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