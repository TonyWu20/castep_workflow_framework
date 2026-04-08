use std::path::PathBuf;
use tempfile::tempdir;
use workflow_core::{
    state::{TaskStatus, WorkflowState},
    Task, Workflow,
};
use workflow_utils::TaskExecutor;

#[test]
fn test_hubbard_u_sweep_with_mock_castep() {
    let dir = tempdir().unwrap();
    let mut workflow = Workflow::resume("hubbard_u", dir.path()).unwrap();

    let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // Register 3 tasks (U=0.0, 1.0, 2.0)
    for u in [0.0_f64, 1.0, 2.0] {
        let task_id = format!("scf_U{:.1}", u);
        let workdir = format!("runs/U{:.1}", u);
        let abs_workdir = dir.path().join(&workdir);
        let path_clone = path.clone();

        let task = Task::new(&task_id, move || {
            // Create workflow files
            let cell_content = "%BLOCK LATTICE_CART\n  3.25 0.0 0.0\n  0.0 3.25 0.0\n  0.0 0.0 5.21\n%ENDBLOCK LATTICE_CART\n".to_string();
            let param_content = "task : SinglePoint\n";

            workflow_utils::create_dir(&abs_workdir)?;
            workflow_utils::write_file(abs_workdir.join("ZnO.cell"), &cell_content)?;
            workflow_utils::write_file(abs_workdir.join("ZnO.param"), param_content)?;

            let result = TaskExecutor::new(&abs_workdir)
                .env("PATH", &path_clone)
                .command("mock_castep")
                .arg("ZnO")
                .execute()?;

            if !result.success() {
                anyhow::bail!("castep failed: {:?}\n{}", result.exit_code, result.stderr);
            }

            Ok(())
        });

        workflow.add_task(task).unwrap();
    }

    workflow.run().unwrap();

    // Verify all tasks completed
    let state_path = dir.path().join(".hubbard_u.workflow.json");
    let state = WorkflowState::load(&state_path).unwrap();

    assert!(matches!(
        state.tasks.get("scf_U0.0"),
        Some(&TaskStatus::Completed)
    ));
    assert!(matches!(
        state.tasks.get("scf_U1.0"),
        Some(&TaskStatus::Completed)
    ));
    assert!(matches!(
        state.tasks.get("scf_U2.0"),
        Some(&TaskStatus::Completed)
    ));

    // Verify castep output files exist
    for u in [0.0_f64, 1.0, 2.0] {
        let castep_file = dir.path().join(format!("runs/U{:.1}/ZnO.castep", u));
        assert!(castep_file.exists(), "Expected {castep_file:?} to exist");
    }
}
