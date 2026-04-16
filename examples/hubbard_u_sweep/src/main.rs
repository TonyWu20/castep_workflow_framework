use anyhow::Result;
use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use castep_cell_io::CellDocument;
use std::collections::HashMap;
use std::sync::Arc;
use workflow_core::state::JsonStateStore;
use workflow_core::task::{ExecutionMode, Task};
use workflow_core::workflow::Workflow;
use workflow_core::{HookExecutor, ProcessRunner, WorkflowError};
use workflow_utils::{create_dir, write_file, ShellHookExecutor, SystemProcessRunner};

fn main() -> Result<()> {
    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");

    let mut workflow = Workflow::new("hubbard_u_sweep").with_max_parallel(4)?;

    // Loop through U values: [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]
    for u in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{:.1}", u);
        let workdir = std::path::PathBuf::from(format!("runs/U{:.1}", u));
        let seed_cell = seed_cell.to_owned();
        let seed_param = seed_param.to_owned();

        let task = Task::new(
            &task_id,
            ExecutionMode::Direct {
                command: "castep".into(),
                args: vec!["ZnO".into()],
                env: HashMap::new(),
                timeout: None,
            },
        )
        .workdir(workdir.clone())
        .setup(move |workdir| {
            create_dir(workdir.to_str().unwrap())?;

            let mut cell_doc: CellDocument =
                parse(&seed_cell).map_err(|e| WorkflowError::InvalidConfig(e.to_string()))?;

            let atom_u = AtomHubbardU::builder()
                .species(Species::Symbol("Zn".to_string()))
                .orbitals(vec![OrbitalU::D(u)])
                .build();
            let hubbard_u = HubbardU::builder()
                .unit(HubbardUUnit::ElectronVolt)
                .atom_u_values(vec![atom_u])
                .build();
            cell_doc.hubbard_u = Some(hubbard_u);

            let output = to_string_many_spaced(&cell_doc.to_cell_file());
            write_file(format!("{}/ZnO.cell", workdir.display()), &output)
                .map_err(|e| WorkflowError::Io(std::io::Error::other(e.to_string())))?;
            write_file(format!("{}/ZnO.param", workdir.display()), &seed_param)
                .map_err(|e| WorkflowError::Io(std::io::Error::other(e.to_string())))?;
            Ok(())
        });

        workflow.add_task(task)?;
    }

    let state_path = std::path::PathBuf::from(".hubbard_u_sweep.workflow.json");
    let mut state = JsonStateStore::new("hubbard_u_sweep", state_path);
    let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner);
    let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);

    workflow.run(&mut state, runner, executor)?;
    Ok(())
}
