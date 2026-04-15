use std::time::Duration;
use workflow_utils::TaskExecutor;

#[test]
fn test_executor_basic() {
    let result = TaskExecutor::new("/tmp")
        .command("echo")
        .arg("hello")
        .execute()
        .unwrap();
    assert!(result.success());
    assert!(result.stdout.contains("hello"));
}

#[test]
fn test_executor_exit_code() {
    let result = TaskExecutor::new("/tmp")
        .command("sh")
        .args(vec!["-c".into(), "exit 42".into()])
        .execute()
        .unwrap();
    assert_eq!(result.exit_code, Some(42));
    assert!(!result.success());
}

#[test]
fn test_executor_with_env() {
    let result = TaskExecutor::new("/tmp")
        .command("sh")
        .args(vec!["-c".into(), "echo $MY_VAR".into()])
        .env("MY_VAR", "testvalue")
        .execute()
        .unwrap();
    assert!(result.success());
    assert!(result.stdout.contains("testvalue"));
}

#[test]
fn test_executor_spawn_and_terminate() {
    let mut handle = TaskExecutor::new("/tmp")
        .command("sleep")
        .arg("60")
        .spawn()
        .unwrap();
    assert!(handle.is_running());
    handle.terminate().unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert!(!handle.is_running());
}

#[test]
fn test_execution_handle_pid() {
    let handle = TaskExecutor::new("/tmp")
        .command("echo")
        .arg("hello")
        .spawn()
        .unwrap();
    let pid = handle.pid();
    assert!(pid > 0);
}

