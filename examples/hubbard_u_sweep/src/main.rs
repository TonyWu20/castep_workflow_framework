use anyhow::Result;
use workflow_core::Workflow;
use workflow_utils::{TaskExecutor, create_dir, write_file};

fn main() -> Result<()> {
    let mut workflow = Workflow::builder()
        .name("hubbard_u_sweep".to_string())
        .state_dir("./".into())
        .build()?;

    for u in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0] {
        let task_id = format!("scf_U{:.1}", u);
        let workdir = format!("runs/U{:.1}", u);

        let task = workflow_core::Task::new(&task_id, move || {
            create_dir(&workdir)?;

            // TODO: replace with castep-cell-io builders
            let cell_content = format!(
                "%BLOCK LATTICE_CART\n  3.25 0.0 0.0\n  0.0 3.25 0.0\n  0.0 0.0 5.21\n%ENDBLOCK LATTICE_CART\n"
            );
            write_file(format!("{}/ZnO.cell", workdir), &cell_content)?;
            write_file(format!("{}/ZnO.param", workdir), "task : SinglePoint\n")?;

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
