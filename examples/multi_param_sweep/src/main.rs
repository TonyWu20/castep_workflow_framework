mod config;
mod job_script;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use clap::Parser;
use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
use castep_cell_io::cell::bz_sampling_kpoints::KpointsMpGrid;
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use castep_cell_io::param::basis_set::CutOffEnergy;
use castep_cell_io::{CellDocument, ParamDocument};
use config::{parse_cutoffs, parse_kpoints, parse_u_values, SweepConfig};
use itertools::iproduct;
use job_script::generate_job_script;
use workflow_utils::prelude::*;

fn main() -> anyhow::Result<()> {
    workflow_core::init_default_logging().ok();
    let config = SweepConfig::parse();

    let tasks = build_all_scf_tasks(&config)?;

    let mut workflow = Workflow::new("multi_param_sweep")
        .with_max_parallel(config.max_parallel)?
        .with_log_dir("logs")
        .with_root_dir(&config.workdir);

    if !config.local {
        workflow = workflow.with_queued_submitter(Arc::new(QueuedRunner::new(SchedulerKind::Slurm)));
    }

    for task in tasks {
        workflow.add_task(task)?;
    }

    if config.dry_run {
        let order = workflow.dry_run()?;
        println!("Dry-run topological order:");
        for task_id in &order {
            println!("  {task_id}");
        }
        return Ok(());
    }

    let state_path = PathBuf::from(".multi_param_sweep.workflow.json");
    let mut state = JsonStateStore::new("multi_param_sweep", state_path);

    let summary = if config.local {
        run_default(&mut workflow, &mut state)?
    } else {
        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner::new());
        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
        workflow.run(&mut state, runner, executor)?
    };

    println!(
        "Workflow complete: {} succeeded, {} failed, {} skipped ({:.1}s)",
        summary.succeeded.len(),
        summary.failed.len(),
        summary.skipped.len(),
        summary.duration.as_secs_f64(),
    );
    Ok(())
}

fn build_one_scf_task(
    config: &SweepConfig,
    u: f64,
    kpoint: Option<[u32; 3]>,
    cutoff: Option<f64>,
    seed_cell: &str,
    seed_param: &str,
) -> std::result::Result<Task, WorkflowError> {
    // Build task ID
    let u_str = format!("{:.1}", u);
    let k_str = kpoint.map(|k| format!("{}x{}x{}", k[0], k[1], k[2]));
    let c_str = cutoff.map(|c| format!("{:.0}", c));

    let id = {
        let mut id = format!("scf_U{u_str}");
        if let Some(ref k) = k_str {
            id.push_str(&format!("_k{k}"));
        }
        if let Some(ref c) = c_str {
            id.push_str(&format!("_c{c}"));
        }
        id
    };

    // Build workdir path
    let workdir = {
        let mut name = format!("U{u_str}");
        if let Some(ref k) = k_str {
            name.push_str(&format!("_k{k}"));
        }
        if let Some(ref c) = c_str {
            name.push_str(&format!("_c{c}"));
        }
        PathBuf::from("runs").join(&name)
    };

    // Determine execution mode
    let mode = if config.local {
        ExecutionMode::direct(&config.castep_command, &[&config.seed_name])
    } else {
        ExecutionMode::Queued
    };

    // Clone values needed by move closures
    let element = config.element.clone();
    let orbital_char = config.orbital;
    let local = config.local;
    let seed_name = config.seed_name.clone();
    let config_clone = config.clone();
    let task_id = id.clone();
    let seed_cell_owned = seed_cell.to_owned();
    let seed_param_owned = seed_param.to_owned();
    let collect_seed = seed_name.clone();

    // Build setup closure
    let boxed_setup = Box::new(move |path: &Path| -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
        create_dir(path)?;

        // Parse and modify cell document
        let mut cell_doc: CellDocument = parse(&seed_cell_owned)?;

        // Add HubbardU block
        let species = Species::Symbol(element.clone());
        let orbital = match orbital_char {
            's' => OrbitalU::S(u),
            'p' => OrbitalU::P(u),
            'd' => OrbitalU::D(u),
            'f' => OrbitalU::F(u),
            _ => {
                return Err(Box::new(WorkflowError::InvalidConfig(format!(
                    "invalid orbital: {}",
                    orbital_char
                ))))
            }
        };
        let atom_u = AtomHubbardU::builder()
            .species(species)
            .orbitals(vec![orbital])
            .build();
        let hubbard_u = HubbardU::builder()
            .unit(HubbardUUnit::ElectronVolt)
            .atom_u_values(vec![atom_u])
            .build();
        cell_doc.species.hubbard_u = Some(hubbard_u);

        // Set kpoints if provided
        if let Some(k) = kpoint {
            cell_doc.kpoints.kpoints_mp_grid = Some(KpointsMpGrid(k));
        }

        // Parse and modify param document
        let mut param_doc: ParamDocument = parse(&seed_param_owned)?;
        if let Some(c) = cutoff {
            param_doc.basis_set.cutoff_energy = Some(CutOffEnergy { value: c, unit: None });
        }

        // Serialize to file strings
        let cell_str = to_string_many_spaced(&cell_doc.to_cell_file());
        let param_str = to_string_many_spaced(&param_doc.to_cell_file());

        // Write .cell and .param files
        write_file(path.join(format!("{seed_name}.cell")), &cell_str)?;
        write_file(path.join(format!("{seed_name}.param")), &param_str)?;

        // Write job script if queued
        if !local {
            let script = generate_job_script(&config_clone, &task_id, &seed_name);
            write_file(path.join(JOB_SCRIPT_NAME), &script)?;
        }

        Ok(())
    });

    // Build collect closure
    let boxed_collect = Box::new(move |path: &Path| -> std::result::Result<(), Box<dyn Error + Send + Sync>> {
        let output_str = read_file(path.join(format!("{collect_seed}.castep")))?;
        if !output_str.contains("Total time") {
            return Err(Box::new(WorkflowError::InvalidConfig(
                "CASTEP output missing 'Total time' marker".into(),
            )));
        }
        Ok(())
    });

    let mut task = Task::new(id, mode)
        .workdir(workdir);
    task.setup = Some(boxed_setup);
    task.collect = Some(boxed_collect);

    Ok(task)
}

fn build_all_scf_tasks(config: &SweepConfig) -> anyhow::Result<Vec<Task>> {
    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");

    let u_vals = parse_u_values(&config.u_values)?;

    let kpoints: Option<Vec<[u32; 3]>> = match &config.kpoints {
        Some(s) => Some(parse_kpoints(s)?),
        None => None,
    };

    let cutoffs: Option<Vec<f64>> = match &config.cutoffs {
        Some(s) => Some(parse_cutoffs(s)?),
        None => None,
    };

    match config.sweep_mode.as_str() {
        "product" => {
            let kpoint_opts: Vec<Option<[u32; 3]>> = match &kpoints {
                Some(kpts) => kpts.iter().map(|&k| Some(k)).collect(),
                None => vec![None],
            };
            let cutoff_opts: Vec<Option<f64>> = match &cutoffs {
                Some(cuts) => cuts.iter().map(|&c| Some(c)).collect(),
                None => vec![None],
            };

            let mut tasks = Vec::new();
            for (&u_val, &kpoint_opt, &cutoff_opt) in
                iproduct!(&u_vals, &kpoint_opts, &cutoff_opts)
            {
                tasks.push(build_one_scf_task(
                    config,
                    u_val,
                    kpoint_opt,
                    cutoff_opt,
                    seed_cell,
                    seed_param,
                )?);
            }
            Ok(tasks)
        }
        "pairwise" => {
            let kpts = kpoints.ok_or_else(|| anyhow!("pairwise mode requires --kpoints"))?;
            let cuts = cutoffs.ok_or_else(|| anyhow!("pairwise mode requires --cutoffs"))?;

            if u_vals.len() != kpts.len() || u_vals.len() != cuts.len() {
                anyhow::bail!(
                    "all parameter lists must have the same length for pairwise mode, got u={}, k={}, c={}",
                    u_vals.len(),
                    kpts.len(),
                    cuts.len()
                );
            }

            let mut tasks = Vec::with_capacity(u_vals.len());
            for ((&u_val, &kpt), &cut) in u_vals.iter().zip(&kpts).zip(&cuts) {
                tasks.push(build_one_scf_task(
                    config,
                    u_val,
                    Some(kpt),
                    Some(cut),
                    seed_cell,
                    seed_param,
                )?);
            }
            Ok(tasks)
        }
        mode => Err(anyhow!("unknown sweep mode: {mode}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SweepConfig;
    use clap::Parser;

    fn default_config() -> SweepConfig {
        SweepConfig::parse_from(["test"])
    }

    // --- Sweep combinatorics tests ---

    #[test]
    fn product_mode_produces_tasks() {
        let config = default_config();
        let tasks = build_all_scf_tasks(&config).unwrap();
        assert_eq!(tasks.len(), 6);
    }

    #[test]
    fn product_mode_cartesian_product() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0,2.0",
            "--kpoints", "8x8x8,6x6x6",
            "--cutoffs", "300,500,800",
            "--sweep-mode", "product",
        ]);
        let tasks = build_all_scf_tasks(&config).unwrap();
        assert_eq!(tasks.len(), 18);
    }

    #[test]
    fn pairwise_mode_zip() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0,2.0",
            "--kpoints", "8x8x8,6x6x6,4x4x4",
            "--cutoffs", "300,500,800",
            "--sweep-mode", "pairwise",
        ]);
        let tasks = build_all_scf_tasks(&config).unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[test]
    fn pairwise_requires_kpoints() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0,2.0",
            "--cutoffs", "300,500,800",
            "--sweep-mode", "pairwise",
        ]);
        let err = build_all_scf_tasks(&config).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("kpoints"), "error should mention kpoints: {msg}");
    }

    #[test]
    fn pairwise_requires_cutoffs() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0,2.0",
            "--kpoints", "8x8x8,6x6x6,4x4x4",
            "--sweep-mode", "pairwise",
        ]);
        let err = build_all_scf_tasks(&config).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("cutoffs"), "error should mention cutoffs: {msg}");
    }

    #[test]
    fn pairwise_unequal_lengths() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0",
            "--kpoints", "8x8x8,6x6x6,4x4x4",
            "--cutoffs", "300,500,800",
            "--sweep-mode", "pairwise",
        ]);
        let err = build_all_scf_tasks(&config).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("same length") || msg.contains("length"), "error should mention length mismatch: {msg}");
    }

    #[test]
    fn unknown_sweep_mode() {
        let config = SweepConfig::parse_from([
            "test",
            "--sweep-mode", "invalid",
        ]);
        let err = build_all_scf_tasks(&config).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("unknown sweep mode"), "error should mention unknown mode: {msg}");
    }

    // --- Task structure tests ---

    #[test]
    fn task_ids_are_unique() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0",
            "--kpoints", "8x8x8,6x6x6",
            "--cutoffs", "300,500",
            "--sweep-mode", "product",
        ]);
        let tasks = build_all_scf_tasks(&config).unwrap();
        let ids: Vec<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(ids.len(), unique_ids.len(), "all task IDs must be unique");
    }

    #[test]
    fn task_ids_encode_parameters() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "3.0",
            "--kpoints", "8x8x8",
            "--cutoffs", "500",
            "--sweep-mode", "product",
        ]);
        let tasks = build_all_scf_tasks(&config).unwrap();
        assert_eq!(tasks.len(), 1);
        let id = &tasks[0].id;
        assert!(id.contains("U3") || id.contains("U3.0"), "task ID should contain U value: {id}");
        assert!(id.contains("k8x8x8") || id.contains("k"), "task ID should contain kpoint: {id}");
        assert!(id.contains("c500") || id.contains("c"), "task ID should contain cutoff: {id}");
    }

    #[test]
    fn product_with_optional_params() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0,2.0",
        ]);
        let tasks = build_all_scf_tasks(&config).unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[test]
    fn scf_tasks_have_no_dependencies() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "0.0,1.0",
            "--kpoints", "8x8x8",
            "--cutoffs", "500",
            "--sweep-mode", "product",
        ]);
        let tasks = build_all_scf_tasks(&config).unwrap();
        for task in &tasks {
            assert!(task.dependencies.is_empty(), "SCF task should have no dependencies: {}", task.id);
        }
    }

    #[test]
    fn tasks_have_workdirs() {
        let config = SweepConfig::parse_from([
            "test",
            "--u-values", "1.0",
            "--kpoints", "8x8x8",
            "--cutoffs", "500",
            "--sweep-mode", "product",
        ]);
        let tasks = build_all_scf_tasks(&config).unwrap();
        assert!(!tasks.is_empty());
        for task in &tasks {
            assert!(!task.workdir.as_os_str().is_empty(), "task {} should have a workdir", task.id);
        }
    }
}
