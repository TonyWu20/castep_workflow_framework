use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCell, ToCellFile};
use castep_cell_io::cell::species::QuantizationAxis;
use castep_cell_io::param::exchange_correlation::SpinPolarized;
use castep_cell_io::{CellDocument, ParamDocument};
use itertools::iproduct;
use workflow_utils::prelude::*;

/// Maximum number of simultaneous Slurm jobs the workflow may submit.
const MAX_PARALLEL: usize = 4;

// --------------------------------------------------------------------------
// Seed definitions – embedded at compile time from the workspace-level seeds/
// --------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Seed {
    name: &'static str,
    cell: &'static str,
    param: &'static str,
    slurm_script: &'static str,
}

const SEEDS: [Seed; 2] = [
    Seed {
        name: "ZnO_LR",
        cell: include_str!("../../../seeds/ZnO_LR/ZnO_LR.cell"),
        param: include_str!("../../../seeds/ZnO_LR/ZnO_LR.param"),
        slurm_script: include_str!("../../../seeds/ZnO_LR/slurm_job_ZnO_LR.sh"),
    },
    Seed {
        name: "NiO",
        cell: include_str!("../../../seeds/NiO/NiO.cell"),
        param: include_str!("../../../seeds/NiO/NiO.param"),
        slurm_script: include_str!("../../../seeds/NiO/slurm_job_NiO.sh"),
    },
];

// --------------------------------------------------------------------------
// Entry-point
// --------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    workflow_core::init_default_logging().ok();

    let tasks = build_all_tasks()?;

    let mut workflow = Workflow::new("ZnO_NiO_hubbard_U_spin_debug_runs")
        .with_max_parallel(MAX_PARALLEL)?
        .with_log_dir("logs")
        .with_root_dir(".")
        .with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));

    for task in tasks {
        workflow.add_task(task)?;
    }

    // Support --dry-run so users can preview the task graph without submitting.
    if std::env::args().any(|a| a == "--dry-run") {
        let order = workflow.dry_run()?;
        println!("Dry-run topological order ({} tasks):", order.len());
        for task_id in &order {
            println!("  {task_id}");
        }
        return Ok(());
    }

    let state_path = PathBuf::from(".ZnO_NiO_hubbard_U_spin_debug_runs.workflow.json");
    let mut state = JsonStateStore::new("ZnO_NiO_hubbard_U_spin_debug_runs", state_path);

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

// --------------------------------------------------------------------------
// Task construction
// --------------------------------------------------------------------------

/// Build the full Cartesian-product sweep: 2 seeds × 2 Hubbard-U states ×
/// 2 quantization-axis states × 2 spin-polarised states = 16 independent tasks.
fn build_all_tasks() -> anyhow::Result<Vec<Task>> {
    let hubbard_options = [true, false];
    let qaxis_options = [true, false];
    let spin_options = [true, false];

    let mut tasks = Vec::with_capacity(SEEDS.len() * 2 * 2 * 2);
    for (&seed, &has_hubbard, &has_qaxis, &is_spin_pol) in
        iproduct!(&SEEDS, &hubbard_options, &qaxis_options, &spin_options)
    {
        tasks.push(build_one_task(seed, has_hubbard, has_qaxis, is_spin_pol)?);
    }
    Ok(tasks)
}

/// Construct a single sweep task for the given parameter combination.
fn build_one_task(
    seed: Seed,
    has_hubbard: bool,
    has_qaxis: bool,
    is_spin_pol: bool,
) -> anyhow::Result<Task> {
    let task_id = format!(
        "{}_{}_QA-{}_Spin-{}",
        seed.name,
        if has_hubbard { "Hubbard-on" } else { "Hubbard-off" },
        if has_qaxis { "on" } else { "off" },
        if is_spin_pol { "true" } else { "false" },
    );
    let workdir = PathBuf::from("runs").join(&task_id);

    // Owned copies for closure capture.
    let seed_cell = seed.cell.to_owned();
    let seed_param = seed.param.to_owned();
    let seed_slurm = seed.slurm_script.to_owned();
    let seed_name = seed.name.to_owned();
    let tid = task_id.clone();

    let boxed_setup = Box::new(move |path: &Path| -> Result<(), Box<dyn Error + Send + Sync>> {
        create_dir(path)?;

        // --- .cell file ----------------------------------------------------
        let mut cell_doc: CellDocument = parse(&seed_cell)?;
        if !has_hubbard {
            cell_doc.hubbard_u = None;
        }

        let mut cells = cell_doc.to_cell_file();
        if has_qaxis {
            // CellDocument v0.5.0 has no quantization_axis field, so we
            // append the keyword manually. This workaround remains correct
            // even after a future release adds the field.
            cells.push(
                QuantizationAxis {
                    direction: [0.0, 0.0, 1.0],
                }
                .to_cell(),
            );
        }
        let cell_str = to_string_many_spaced(&cells);

        // --- .param file ---------------------------------------------------
        let mut param_doc: ParamDocument = parse(&seed_param)?;
        param_doc.exchange_correlation.spin_polarized = Some(SpinPolarized(is_spin_pol));
        let param_str = to_string_many_spaced(&param_doc.to_cell_file());

        // --- Write input files ---------------------------------------------
        write_file(path.join(format!("{seed_name}.cell")), &cell_str)?;
        write_file(path.join(format!("{seed_name}.param")), &param_str)?;

        // --- Job script ----------------------------------------------------
        // Patch only #SBATCH --job-name= so each task is distinguishable in
        // squeue without touching the HPC paths the user has in their seed
        // slurm scripts.
        let patched_script = seed_slurm
            .lines()
            .map(|line| {
                if line.starts_with("#SBATCH --job-name=") {
                    format!("#SBATCH --job-name=\"{tid}\"")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        write_file(path.join(JOB_SCRIPT_NAME), &patched_script)?;

        Ok(())
    });

    let seed_name2 = seed.name.to_owned();
    let boxed_collect = Box::new(move |path: &Path| -> Result<(), Box<dyn Error + Send + Sync>> {
        let output_str = read_file(path.join(format!("{seed_name2}.castep")))?;
        if !output_str.contains("Total time") {
            return Err(Box::new(WorkflowError::InvalidConfig(
                "CASTEP output missing 'Total time' marker".into(),
            )));
        }
        Ok(())
    });

    let mut task = Task::new(task_id.clone(), ExecutionMode::Queued).workdir(workdir);
    task.setup = Some(boxed_setup);
    task.collect = Some(boxed_collect);

    Ok(task)
}

// --------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// The full Cartesian product must produce exactly 16 tasks.
    #[test]
    fn correct_task_count() {
        let tasks = build_all_tasks().unwrap();
        assert_eq!(tasks.len(), 16);
    }

    /// Every task ID must be unique so that state tracking is unambiguous.
    #[test]
    fn unique_task_ids() {
        let tasks = build_all_tasks().unwrap();
        let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
        let mut dedup = ids.clone();
        dedup.sort();
        dedup.dedup();
        assert_eq!(ids.len(), dedup.len(), "task IDs must be unique");
    }

    /// All tasks are independent (no dependencies between them).
    #[test]
    fn no_dependencies() {
        let tasks = build_all_tasks().unwrap();
        for task in &tasks {
            assert!(
                task.dependencies.is_empty(),
                "task {} should have no dependencies",
                task.id
            );
        }
    }

    /// IDs encode the sweep parameters so they are human-readable in squeue.
    #[test]
    fn ids_encode_parameters() {
        let tasks = build_all_tasks().unwrap();
        // Every possible keyword should appear in at least one task ID.
        let all_ids: String = tasks.iter().map(|t| t.id.as_str()).collect::<Vec<_>>().join(" ");
        for keyword in &["ZnO_LR", "NiO", "Hubbard-on", "Hubbard-off", "QA-on", "QA-off", "Spin-true", "Spin-false"] {
            assert!(all_ids.contains(keyword), "expected '{keyword}' in some task ID");
        }
    }

    /// Every task has a non-empty workdir set.
    #[test]
    fn tasks_have_workdirs() {
        let tasks = build_all_tasks().unwrap();
        for task in &tasks {
            assert!(
                !task.workdir.as_os_str().is_empty(),
                "task {} should have a workdir",
                task.id
            );
        }
    }
}
