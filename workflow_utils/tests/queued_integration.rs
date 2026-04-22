//! Integration tests for `QueuedRunner` and `QueuedProcessHandle`.
//!
//! These tests verify that `QueuedRunner` correctly implements `QueuedSubmitter`,
//! handles scheduler unavailability gracefully, and that `QueuedProcessHandle`
//! satisfies the `ProcessHandle` trait contract.

use serial_test::serial;
use workflow_core::process::QueuedSubmitter;
use workflow_utils::{QueuedRunner, SchedulerKind};

/// Compile-time verification that `QueuedRunner` implements `QueuedSubmitter`.
#[test]
fn queued_runner_implements_queued_submitter_slurm() {
    let runner = QueuedRunner::new(SchedulerKind::Slurm);
    let _: &dyn QueuedSubmitter = &runner;
}

#[test]
fn queued_runner_implements_queued_submitter_pbs() {
    let runner = QueuedRunner::new(SchedulerKind::Pbs);
    let _: &dyn QueuedSubmitter = &runner;
}

/// When `sbatch` is not installed, `submit` must return `QueueSubmitFailed`,
/// not panic or produce an `Io` error from `Command::output()`.
///
/// This relies on `sh -c "sbatch ..."` exiting non-zero when `sbatch` is absent,
/// which triggers the `!output.status.success()` branch.
#[test]
#[serial]
fn submit_returns_err_when_sbatch_unavailable() {
    use workflow_core::error::WorkflowError;

    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let workdir = dir.path().join("work");
    std::fs::create_dir_all(&workdir).unwrap();
    std::fs::write(workdir.join("job.sh"), "#!/bin/sh\necho hello\n").unwrap();

    // Restrict PATH to an empty directory so `sbatch` cannot be found.
    let empty_bin = dir.path().join("empty_bin");
    std::fs::create_dir_all(&empty_bin).unwrap();

    // Set PATH for this process (tests run sequentially within this file due to
    // the PATH mutation; mark with #[serial] if the suite is parallelised).
    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", empty_bin.display().to_string());

    let runner = QueuedRunner::new(SchedulerKind::Slurm);
    let result = runner.submit(&workdir, "task_a", &log_dir);

    std::env::set_var("PATH", original);

    assert!(
        result.is_err(),
        "submit should fail when sbatch is not on PATH"
    );
    match result {
        Err(WorkflowError::QueueSubmitFailed(_)) => {}
        Err(e) => panic!("expected QueueSubmitFailed, got {:?}", e),
        Ok(_) => panic!("expected error but got Ok"),
    }
}

/// Verify that a successful mock submission returns a `ProcessHandle` whose
/// `wait()` produces an `OnDisk` `OutputLocation` pointing to the expected paths.
#[cfg(unix)]
#[test]
#[serial]
fn submit_with_mock_sbatch_returns_on_disk_handle() {
    use std::os::unix::fs::PermissionsExt;
    use workflow_core::process::OutputLocation;

    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let workdir = dir.path().join("work");
    std::fs::create_dir_all(&workdir).unwrap();
    std::fs::write(workdir.join("job.sh"), "#!/bin/sh\necho hello\n").unwrap();

    // Mock `sbatch` that prints a SLURM-style submission line and exits 0.
    let mock_dir = dir.path().join("mock_bin");
    std::fs::create_dir_all(&mock_dir).unwrap();
    let mock_sbatch = mock_dir.join("sbatch");
    std::fs::write(
        &mock_sbatch,
        "#!/bin/sh\necho 'Submitted batch job 99999'\n",
    )
    .unwrap();
    std::fs::set_permissions(&mock_sbatch, std::fs::Permissions::from_mode(0o755)).unwrap();

    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{}", mock_dir.display(), original));

    let runner = QueuedRunner::new(SchedulerKind::Slurm);
    let mut handle = runner
        .submit(&workdir, "task_a", &log_dir)
        .expect("submit should succeed with mock sbatch");

    std::env::set_var("PATH", original);

    // `wait()` on a QueuedProcessHandle returns immediately with OnDisk paths.
    let result = handle.wait().expect("wait should succeed");
    assert!(
        matches!(result.output, OutputLocation::OnDisk { .. }),
        "output should be OnDisk for queued handles"
    );
    if let OutputLocation::OnDisk {
        stdout_path,
        stderr_path,
    } = result.output
    {
        assert_eq!(
            stdout_path,
            log_dir.join("task_a.stdout"),
            "stdout path should follow <log_dir>/<task_id>.stdout convention"
        );
        assert_eq!(
            stderr_path,
            log_dir.join("task_a.stderr"),
            "stderr path should follow <log_dir>/<task_id>.stderr convention"
        );
    }
}

