mod config;
mod job_script;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use clap::Parser;
use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, OrbitalU, Species};
use castep_cell_io::{CellDocument, ParamDocument};
use config::{parse_u_values, ChainConfig};
use job_script::generate_job_script;
use workflow_utils::prelude::*;

fn build_scf_task(
    config: &ChainConfig,
    seed_cell: &str,
    seed_param: &str,
) -> std::result::Result<Task, WorkflowError> {
    let id = "scf".to_string();
    let workdir = PathBuf::from("runs/scf_dos");

    let mode = if config.local {
        ExecutionMode::direct(&config.castep_command, &[&config.seed_name])
    } else {
        ExecutionMode::Queued
    };

    // Clone values for move closures (owned values only, no references to config)
    let element = config.element.clone();
    let seed_name = config.seed_name.clone();
    let orbital_char = config.orbital;
    let u_value = config.u_value;
    let local = config.local;

    // Pre-compute job script outside closures to avoid capturing config ref
    let queued_script = if local {
        None
    } else {
        Some(generate_job_script(config, &seed_name, &seed_name))
    };

    let setup_seed_cell = seed_cell.to_owned();
    let setup_seed_param = seed_param.to_owned();
    let setup_element = element.clone();
    let setup_seed_name = seed_name.clone();

    let boxed_setup: Box<dyn Fn(&Path) -> std::result::Result<(), Box<dyn Error + Send + Sync>> + Send + Sync> = Box::new(move |path: &Path| {
        create_dir(path)?;

        // Parse and modify cell document -- inject Hubbard U
        let mut cell_doc: CellDocument = parse(&setup_seed_cell)?;
        let species = Species::Symbol(setup_element.clone());
        let orbital = match orbital_char {
            's' => OrbitalU::S(u_value),
            'p' => OrbitalU::P(u_value),
            'd' => OrbitalU::D(u_value),
            'f' => OrbitalU::F(u_value),
            _ => return Err(Box::new(WorkflowError::InvalidConfig(
                format!("invalid orbital: {}", orbital_char)
            ))),
        };
        let atom_u = AtomHubbardU::builder()
            .species(species)
            .orbitals(vec![orbital])
            .build();
        let hubbard_u = HubbardU::builder()
            .atom_u_values(vec![atom_u])
            .build();
        cell_doc.hubbard_u = Some(hubbard_u);

        // Parse param document (no modification needed for SCF)
        let param_doc: ParamDocument = parse(&setup_seed_param)?;

        // Serialize to file strings
        let cell_str = to_string_many_spaced(&cell_doc.to_cell_file());
        let param_str = to_string_many_spaced(&param_doc.to_cell_file());

        // Write .cell and .param files
        write_file(path.join(format!("{}.cell", setup_seed_name)), &cell_str)?;
        write_file(path.join(format!("{}.param", setup_seed_name)), &param_str)?;

        // Write job script if queued
        if let Some(ref script) = queued_script {
            write_file(path.join(JOB_SCRIPT_NAME), script)?;
        }

        Ok(())
    });

    let collect_seed_name = seed_name;
    let boxed_collect: Box<dyn Fn(&Path) -> std::result::Result<(), Box<dyn Error + Send + Sync>> + Send + Sync> = Box::new(move |path: &Path| {
        let output_str = read_file(path.join(format!("{}.castep", collect_seed_name)))?;
        if !output_str.contains("Total time") {
            return Err(Box::new(WorkflowError::InvalidConfig(
                "CASTEP output missing 'Total time' marker".into(),
            )));
        }
        Ok(())
    });

    let mut task = Task::new(id, mode).workdir(workdir);
    task.setup = Some(boxed_setup);
    task.collect = Some(boxed_collect);

    Ok(task)
}

fn build_dos_task(
    config: &ChainConfig,
    scf_task_id: &str,
    seed_cell: &str,
    seed_param: &str,
) -> std::result::Result<Task, WorkflowError> {
    let id = "dos".to_string();
    let workdir = PathBuf::from("runs/scf_dos");

    let mode = if config.local {
        ExecutionMode::direct(&config.castep_command, &["ZnO_DOS"])
    } else {
        ExecutionMode::Queued
    };

    // Pre-compute job script outside closures to avoid capturing config ref
    let queued_script = if config.local {
        None
    } else {
        Some(generate_job_script(config, &id, "ZnO_DOS"))
    };

    let setup_seed_cell = seed_cell.to_owned();
    let setup_seed_param = seed_param.to_owned();
    let _setup_local = config.local;
    let setup_scf_task = scf_task_id.to_owned();

    let boxed_setup: Box<dyn Fn(&Path) -> std::result::Result<(), Box<dyn Error + Send + Sync>> + Send + Sync> = Box::new(move |path: &Path| {
        create_dir(path)?;

        // Parse seed cell and write as ZnO_DOS.cell (reuse seed cell for DOS restart)
        let cell_doc: CellDocument = parse(&setup_seed_cell)?;
        let cell_str = to_string_many_spaced(&cell_doc.to_cell_file());
        write_file(path.join("ZnO_DOS.cell"), &cell_str)?;

        // Parse seed param, set Task::BandStructure
        let mut param_doc: ParamDocument = parse(&setup_seed_param)?;
        param_doc.general.task = Some(castep_cell_io::param::general::Task::BandStructure);
        let param_str = to_string_many_spaced(&param_doc.to_cell_file());
        write_file(path.join("ZnO_DOS.param"), &param_str)?;

        // Copy checkpoint from SCF run
        copy_file(path.join("ZnO.check"), path.join("ZnO_DOS.check"))?;

        // Write job script if queued
        if let Some(ref script) = queued_script {
            write_file(path.join(JOB_SCRIPT_NAME), script)?;
        }

        Ok(())
    });

    let boxed_collect: Box<dyn Fn(&Path) -> std::result::Result<(), Box<dyn Error + Send + Sync>> + Send + Sync> = Box::new(move |_: &Path| {
        let output_str = read_file(PathBuf::from("ZnO_DOS.castep"))?;
        if !output_str.contains("Total time") {
            return Err(Box::new(WorkflowError::InvalidConfig(
                "CASTEP output missing 'Total time' marker".into(),
            )));
        }
        Ok(())
    });

    let mut task = Task::new(id, mode)
        .workdir(workdir)
        .depends_on(setup_scf_task);
    task.setup = Some(boxed_setup);
    task.collect = Some(boxed_collect);

    Ok(task)
}

fn main() {
    println!("scf_dos_chain: skeleton");
}

#[cfg(test)]
mod tests {
    use crate::config::ChainConfig;
    use crate::{build_dos_task, build_scf_task};
    use clap::Parser;

    fn default_config() -> ChainConfig {
        ChainConfig::parse_from(["test"])
    }

    fn test_seeds() -> (&'static str, &'static str) {
        (include_str!("../seeds/ZnO.cell"), include_str!("../seeds/ZnO.param"))
    }

    /// SCF task has ID 'scf'
    #[test]
    fn scf_task_id() {
        let config = default_config();
        let (cell, param) = test_seeds();
        let task = build_scf_task(&config, cell, param).unwrap();
        assert_eq!(task.id, "scf");
    }

    /// DOS task has ID 'dos'
    #[test]
    fn dos_task_id() {
        let config = default_config();
        let (cell, param) = test_seeds();
        let task = build_dos_task(&config, "scf", cell, param).unwrap();
        assert_eq!(task.id, "dos");
    }

    /// DOS task depends on SCF task
    #[test]
    fn dos_depends_on_scf() {
        let config = default_config();
        let (cell, param) = test_seeds();
        let scf = build_scf_task(&config, cell, param).unwrap();
        let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();
        assert!(dos.dependencies.contains(&"scf".to_string()));
    }

    /// SCF task has no dependencies
    #[test]
    fn scf_no_dependencies() {
        let config = default_config();
        let (cell, param) = test_seeds();
        let task = build_scf_task(&config, cell, param).unwrap();
        assert!(task.dependencies.is_empty());
    }

    /// DOS task has exactly 1 dependency
    #[test]
    fn dos_dependency_count() {
        let config = default_config();
        let (cell, param) = test_seeds();
        let scf = build_scf_task(&config, cell, param).unwrap();
        let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();
        assert_eq!(dos.dependencies.len(), 1);
    }

    /// Both tasks share the same workdir
    #[test]
    fn shared_workdir() {
        let config = default_config();
        let (cell, param) = test_seeds();
        let scf = build_scf_task(&config, cell, param).unwrap();
        let dos = build_dos_task(&config, &scf.id, cell, param).unwrap();
        assert_eq!(scf.workdir, dos.workdir, "SCF and DOS tasks should share the same workdir");
        let wd = scf.workdir.to_string_lossy();
        assert!(wd.contains("scf_dos"), "workdir should contain 'scf_dos': {wd}");
    }

    /// SCF task has correct workdir path
    #[test]
    fn scf_workdir_path() {
        let config = default_config();
        let (cell, param) = test_seeds();
        let task = build_scf_task(&config, cell, param).unwrap();
        let wd = task.workdir.to_string_lossy();
        assert!(wd.ends_with("scf_dos") || wd.contains("scf_dos"), "workdir should end with or contain 'scf_dos': {wd}");
    }
}
