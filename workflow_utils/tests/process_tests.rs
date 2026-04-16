use workflow_utils::SystemProcessRunner;
use workflow_core::{ProcessRunner};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_system_process_runner_echo() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "echo",
        &["hello".to_string()],
        &HashMap::new(),
    ).unwrap();

    let result = handle.wait().unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert!(result.stdout.contains("hello"));
}

#[test]
fn test_is_running_transitions() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "sleep",
        &["0.1".to_string()],
        &HashMap::new(),
    ).unwrap();

    // Immediately after spawn, should be running
    assert!(handle.is_running());

    // After wait, should not be running
    handle.wait().unwrap();
    assert!(!handle.is_running());
}

#[test]
fn test_terminate_long_running_process() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "sleep",
        &["10".to_string()],  // Longer sleep to ensure we can kill it
        &HashMap::new(),
    ).unwrap();

    assert!(handle.is_running());
    handle.terminate().unwrap();

    // After terminate, wait should succeed (process is dead)
    let result = handle.wait().unwrap();
    // Killed processes return exit code 137 (SIGKILL) or similar, or None if only signal was received
    assert_ne!(result.exit_code, Some(0), "terminated process should not exit successfully");
}

#[test]
fn test_wait_called_twice_errors() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "echo",
        &["test".to_string()],
        &HashMap::new(),
    ).unwrap();

    handle.wait().unwrap();

    // Second wait should error
    let result = handle.wait();
    assert!(result.is_err());
}

#[test]
fn test_terminate_idempotent() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "echo",
        &["test".to_string()],
        &HashMap::new(),
    ).unwrap();

    handle.wait().unwrap();

    // Terminate after wait should succeed (idempotent)
    assert!(handle.terminate().is_ok());
}

#[test]
fn test_capture_output() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "echo",
        &["test output".to_string(), "line2".to_string()],
        &HashMap::new(),
    ).unwrap();

    let result = handle.wait().unwrap();
    assert!(result.stdout.contains("test output"));
    assert!(result.stderr.is_empty());  // echo writes to stdout, not stderr
}

#[test]
fn test_duration_tracking() {
    let runner = SystemProcessRunner;
    let mut handle = runner.spawn(
        &PathBuf::from("/tmp"),
        "sleep",
        &["0.01".to_string()],
        &HashMap::new(),
    ).unwrap();

    let result = handle.wait().unwrap();
    // Duration should be approximately 10ms (give some margin for OS scheduling)
    assert!(result.duration >= std::time::Duration::from_millis(5));
    // 1 second provides headroom for CI load; sleep 0.01 can be delayed
    assert!(result.duration <= std::time::Duration::from_secs(1));
}
