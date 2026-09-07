//! NiO SinglePoint test seed -- CASTEP job management plus an SCF-stall
//! watcher, driven by the castep_workflow_framework.
//!
//! The seed comes from `~/castep_jobs/NiO_SinglePoint_finer_grid` (an 8-atom
//! rock-salt NiO single-point run, PBE, 380 eV, fine_grid_scale 3.0, 6000
//! max SCF cycles). The test seed runs locally on a workstation, so no SLURM
//! job script is required for the local path.
//!
//! The watcher is task-agnostic: it tails `<seed>.castep`, tracks SCF cycles
//! and energy gains, and flags the two stall signatures from the CeO2 V_O
//! diagnosis (per-block cycle cap + gain oscillation around the tolerance).
//! It is attached to the task as a periodic MonitoringHook.

mod config;
mod job_script;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use workflow_core::{HookTrigger, MonitoringHook, ProcessRunner};
use workflow_core::prelude::*;
use workflow_utils::prelude::*;

use config::NiOConfig;
use job_script::generate_job_script;

const TASK_ID: &str = "nio_single_point";
const SEED_CELL: &str = include_str!("../seeds/NiO.cell");
const SEED_PARAM: &str = include_str!("../seeds/NiO.param");
const WATCHER: &str = include_str!("../scf_watcher.sh");

fn main() -> Result<()> {
    workflow_core::init_default_logging().ok();
    let config = NiOConfig::parse();

    let task = build_task(&config)?;

    let mut workflow = Workflow::new("nio_single_point")
        .with_max_parallel(1)?
        .with_log_dir("logs");

    if config.slurm {
        workflow =
            workflow.with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
    }

    workflow.add_task(task)?;

    if config.dry_run {
        let order = workflow.dry_run()?;
        println!("Dry-run topological order ({} tasks):", order.len());
        for id in &order {
            println!("  {id}");
        }
        return Ok(());
    }

    if !config.slurm && config.seed_dir.is_none() {
        anyhow::bail!(
            "--seed-dir is required for a local run (pseudopotentials are copied from it). \
             Use --dry-run to inspect the task graph without it."
        );
    }

    let state_path = PathBuf::from(".nio_single_point.workflow.json");
    let mut state = JsonStateStore::new("nio_single_point", state_path);

    let summary = if config.slurm {
        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
        workflow.run(&mut state, runner, executor)?
    } else {
        run_default(&mut workflow, &mut state)?
    };

    println!(
        "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
        summary.succeeded.len(),
        summary.failed.len(),
        summary.skipped.len(),
        summary.duration.as_secs_f64(),
    );
    for f in &summary.failed {
        println!("  failed: {}: {}", f.id, f.error);
    }

    Ok(())
}

fn build_task(config: &NiOConfig) -> Result<Task> {
    let seed = config.seed_name.clone();
    let seed_dir = config.seed_dir.clone();
    let slurm = config.slurm;

    // Pseudopotential filenames are pulled from the SPECIES_POT block of the
    // bundled cell. This is task-agnostic: any CASTEP cell that lists its
    // potentials in that block is handled the same way.
    let pots = extract_species_pot_filenames(SEED_CELL);

    let mode = if slurm {
        ExecutionMode::Queued
    } else {
        let (command, args): (String, Vec<String>) = if config.mpi_np > 1 {
            let np = config.mpi_np.to_string();
            (
                "mpirun".to_string(),
                vec![
                    "-np".to_string(),
                    np,
                    config.castep_command.clone(),
                    seed.clone(),
                ],
            )
        } else {
            (config.castep_command.clone(), vec![seed.clone()])
        };
        let mut env = std::collections::HashMap::new();
        env.insert("OMP_NUM_THREADS".to_string(), "1".to_string());
        ExecutionMode::Direct {
            command,
            args,
            env,
            timeout: None,
        }
    };

    let queued_script = if slurm {
        Some(generate_job_script(config, TASK_ID, &seed))
    } else {
        None
    };

    let conf = watcher_conf(config, &seed);

    let setup_seed = seed.clone();
    let setup_seed_dir = seed_dir.clone();
    let setup_pots = pots.clone();
    let setup_conf = conf.clone();
    let setup_queued = queued_script.clone();
    let collect_seed = seed.clone();

    let hook = MonitoringHook::new(
        "scf_stall_watch",
        format!("bash scf_watcher.sh --seed {seed}"),
        HookTrigger::Periodic {
            interval_secs: config.watch_interval,
        },
    );

    let task = Task::new(TASK_ID, mode)
        .workdir(config.workdir.join(TASK_ID))
        .setup(move |path: &Path| -> Result<(), WorkflowError> {
            create_dir(path)?;
            write_file(path.join(format!("{setup_seed}.cell")), SEED_CELL)?;
            write_file(path.join(format!("{setup_seed}.param")), SEED_PARAM)?;
            write_file(path.join("scf_watcher.sh"), WATCHER)?;
            write_file(path.join(".scf_watch.conf"), &setup_conf)?;
            if let Some(ref dir) = setup_seed_dir {
                for pot in &setup_pots {
                    copy_file(dir.join(pot), path.join(pot))?;
                }
            }
            if let Some(ref script) = setup_queued {
                write_file(path.join(JOB_SCRIPT_NAME), script)?;
            }
            Ok(())
        })
        .collect(move |path: &Path| -> Result<(), WorkflowError> {
            let out = read_file(path.join(format!("{collect_seed}.castep")))?;
            if out.contains("Total time") {
                return Ok(());
            }
            let marker = path.join(format!(".scf_stall_detected.{collect_seed}"));
            if marker.exists() {
                let reason = std::fs::read_to_string(&marker).unwrap_or_default();
                return Err(WorkflowError::InvalidConfig(format!(
                    "SCF stall detected -- early-stopped by watcher: {}",
                    reason.trim()
                )));
            }
            Err(WorkflowError::InvalidConfig(
                "CASTEP output missing 'Total time' marker (job did not finish)".into(),
            ))
        })
        .monitors(vec![hook]);

    Ok(task)
}

/// Pull the potential-file names out of the `%BLOCK SPECIES_POT` block of a
/// CASTEP cell. Each entry line is `<element> <file>`; the last token is the
/// file. Lines that are not a filename (no dot) are ignored.
fn extract_species_pot_filenames(cell_text: &str) -> Vec<String> {
    let mut pots = Vec::new();
    let mut in_block = false;
    for line in cell_text.lines() {
        let t = line.trim();
        if t.eq_ignore_ascii_case("%BLOCK SPECIES_POT") {
            in_block = true;
            continue;
        }
        if t.eq_ignore_ascii_case("%ENDBLOCK SPECIES_POT") {
            in_block = false;
            continue;
        }
        if in_block && !t.is_empty() {
            if let Some(name) = t.split_whitespace().last() {
                if name.contains('.') {
                    pots.push(name.to_string());
                }
            }
        }
    }
    pots
}

fn watcher_conf(config: &NiOConfig, seed: &str) -> String {
    let mut s = String::new();
    s.push_str(&format!("MAX_SCF={}\n", config.watch_max_scf));
    s.push_str("WINDOW=50\n");
    s.push_str(&format!("TOL_MULT={}\n", config.watch_tol_mult));
    s.push_str("TOL=1e-05\n");
    if config.watch_kill {
        s.push_str(&format!("KILL_PATTERN='{} {}'\n", config.castep_command, seed));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn local_config() -> NiOConfig {
        NiOConfig::parse_from(["test"])
    }

    fn tmpdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "nio_single_point_test_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Task id and workdir are wired as expected.
    #[test]
    fn task_id_and_workdir() {
        let config = local_config();
        let task = build_task(&config).unwrap();
        assert_eq!(task.id, TASK_ID);
        assert!(task
            .workdir
            .to_string_lossy()
            .contains("nio_single_point"));
    }

    /// The task carries exactly one periodic watcher hook.
    #[test]
    fn periodic_watcher_attached() {
        let config = local_config();
        let task = build_task(&config).unwrap();
        assert_eq!(task.monitors.len(), 1);
        let hook = &task.monitors[0];
        assert_eq!(hook.name, "scf_stall_watch");
        assert!(hook.command.contains("scf_watcher.sh"));
        match hook.trigger {
            HookTrigger::Periodic { .. } => {}
            _ => panic!("expected a periodic trigger"),
        }
    }

    /// The setup closure writes the seed, the watcher, and the conf file, and
    /// copies pseudopotentials when a seed dir is supplied.
    #[test]
    fn setup_writes_files() {
        let mut config = local_config();
        let seed_dir = tmpdir();
        std::fs::write(seed_dir.join("O_00PBE.usp"), "fake").unwrap();
        std::fs::write(seed_dir.join("Ni_00PBE.uspcc"), "fake").unwrap();
        config.seed_dir = Some(seed_dir.clone());

        let task = build_task(&config).unwrap();
        let work = tmpdir().join("setup_out");
        task.setup.as_ref().unwrap()(&work).unwrap();

        assert!(work.join("NiO.cell").exists());
        assert!(work.join("NiO.param").exists());
        assert!(work.join("scf_watcher.sh").exists());
        assert!(work.join(".scf_watch.conf").exists());
        assert!(work.join("O_00PBE.usp").exists());
        assert!(work.join("Ni_00PBE.uspcc").exists());
    }

    /// SPECIES_POT parsing pulls the two potential files out of the cell.
    #[test]
    fn species_pot_extraction() {
        let pots = extract_species_pot_filenames(SEED_CELL);
        assert!(pots.contains(&"O_00PBE.usp".to_string()));
        assert!(pots.contains(&"Ni_00PBE.uspcc".to_string()));
        assert_eq!(pots.len(), 2);
    }

    /// Collect succeeds when the output has a "Total time" marker.
    #[test]
    fn collect_success_on_total_time() {
        let config = local_config();
        let task = build_task(&config).unwrap();
        let dir = tmpdir().join("collect_ok");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("NiO.castep"), "blah\nTotal time = 10.8 s\n").unwrap();
        task.collect.as_ref().unwrap()(&dir).unwrap();
    }

    /// Collect fails with a stall message when the marker file is present.
    #[test]
    fn collect_flags_stall_marker() {
        let config = local_config();
        let task = build_task(&config).unwrap();
        let dir = tmpdir().join("collect_stall");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("NiO.castep"), "incomplete...\n").unwrap();
        std::fs::write(dir.join(".scf_stall_detected.NiO"), "oscillation stall").unwrap();
        let err = task.collect.as_ref().unwrap()(&dir).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("stall"), "message: {msg}");
    }

    /// Collect fails with a generic message when neither marker is present.
    #[test]
    fn collect_missing_total_time() {
        let config = local_config();
        let task = build_task(&config).unwrap();
        let dir = tmpdir().join("collect_bad");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("NiO.castep"), "no total time here\n").unwrap();
        let err = task.collect.as_ref().unwrap()(&dir).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Total time"), "message: {msg}");
    }

    /// The watcher script exists and is non-empty.
    #[test]
    fn watcher_script_present() {
        assert!(
            WATCHER.trim().len() > 100,
            "watcher script is empty or too short"
        );
        assert!(WATCHER.starts_with("#!"));
    }

    /// Local mode is a Direct execution; SLURM mode is Queued.
    #[test]
    fn execution_mode_selects_correctly() {
        let local = local_config();
        let mut slurm = local_config();
        slurm.slurm = true;

        let local_task = build_task(&local).unwrap();
        assert!(matches!(local_task.mode, ExecutionMode::Direct { .. }));

        let slurm_task = build_task(&slurm).unwrap();
        assert!(matches!(slurm_task.mode, ExecutionMode::Queued));
    }
}
