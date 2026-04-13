use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
use workflow_core::{
    state::{StateStore, TaskStatus},
    ExecutionMode, JsonStateStore, Task, Workflow, WorkflowError,
};
use workflow_utils::{create_dir, write_file};

#[test]
fn test_hubbard_u_sweep_with_mock_castep() {
    let dir = tempdir().unwrap();
    let state_path = dir.path().join(".hubbard_u.workflow.json");

    let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
    let path_val = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mut wf = Workflow::new("hubbard_u");

    for u in [0.0_f64, 1.0, 2.0] {
        let task_id = format!("scf_U{:.1}", u);
        let abs_workdir = dir.path().join(format!("runs/U{:.1}", u));
        let workdir_for_setup = abs_workdir.clone();
        let path_clone = path_val.clone();

        let mut env = HashMap::new();
        env.insert("PATH".to_string(), path_clone);

        wf.add_task(
            Task::new(
                &task_id,
                ExecutionMode::Direct {
                    command: "mock_castep".into(),
                    args: vec!["ZnO".into()],
                    env,
                    timeout: None,
                },
            )
            .workdir(abs_workdir.clone())
            .setup(move |_| {
                create_dir(&workdir_for_setup).map_err(|e| {
                    WorkflowError::Io(std::io::Error::other(e.to_string()))
                })?;
                write_file(
                    workdir_for_setup.join("ZnO.cell"),
                    "%BLOCK LATTICE_CART\n  3.25 0.0 0.0\n  0.0 3.25 0.0\n  0.0 0.0 5.21\n%ENDBLOCK LATTICE_CART\n",
                )
                .map_err(|e| WorkflowError::Io(std::io::Error::other(e.to_string())))?;
                write_file(
                    workdir_for_setup.join("ZnO.param"),
                    "task : SinglePoint\n",
                )
                .map_err(|e| WorkflowError::Io(std::io::Error::other(e.to_string())))?;
                Ok(())
            }),
        )
        .unwrap();
    }

    let runner = Arc::new(workflow_utils::SystemProcessRunner);
    let executor = Arc::new(workflow_utils::ShellHookExecutor);
    let mut state = Box::new(JsonStateStore::new("hubbard_u", state_path.clone()));

    wf.run(state.as_mut(), runner, executor).unwrap();

    // Reload from disk to verify final state (run() saves on every status change)
    let state = JsonStateStore::load(&state_path).unwrap();
    for u in [0.0_f64, 1.0, 2.0] {
        let task_id = format!("scf_U{:.1}", u);
        assert!(
            matches!(state.get_status(&task_id), Some(TaskStatus::Completed)),
            "Expected {task_id} to be Completed"
        );
        let castep_file = dir.path().join(format!("runs/U{:.1}/ZnO.castep", u));
        assert!(castep_file.exists(), "Expected {castep_file:?} to exist");
    }
}
