use anyhow::{Context, Result};
use castep_cell_fmt::{parse, ToCellFile, format::to_string_many_spaced};
use castep_cell_io::CellDocument;
use castep_cell_io::cell::species::{AtomHubbardU, HubbardU, HubbardUUnit, OrbitalU, Species};
use workflow_core::Workflow;
use workflow_utils::{TaskExecutor, create_dir, write_file};

fn main() -> Result<()> {
    let seed_cell = include_str!("../seeds/ZnO.cell");
    let seed_param = include_str!("../seeds/ZnO.param");

    let mut workflow = Workflow::builder()
        .name("hubbard_u_sweep".to_string())
        .state_dir("./".into())
        .build()?;

    for u in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{:.1}", u);
        let workdir = format!("runs/U{:.1}", u);

        let task = workflow_core::Task::new(&task_id, move || {
            create_dir(&workdir)?;

            let mut cell_doc: CellDocument = parse(seed_cell).context("failed to parse seed ZnO.cell")?;

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
            write_file(format!("{workdir}/ZnO.cell"), &output)?;
            write_file(format!("{workdir}/ZnO.param"), seed_param)?;

            let result = TaskExecutor::new(&workdir)
                .command("castep")
                .arg("ZnO")
                .execute()?;
            if !result.success() {
                anyhow::bail!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
            }
            Ok(())
        });

        workflow.add_task(task)?;
    }

    workflow.run()
}
