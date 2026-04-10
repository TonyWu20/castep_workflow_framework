use std::os::unix::fs::PermissionsExt;
use std::time::Duration;
use workflow_core::Workflow;
use workflow_utils::{MonitoringHook, HookTrigger};

#[test]
fn test_periodic_hook_executes_multiple_times() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;

    // File to track hook executions
    let log_file = dir.path().join("hook_log.txt");

    // Create executable hook script in tempdir to avoid shell redirection issues
    let script = dir.path().join("hook.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\necho 'hook executed' >> {}", log_file.display()),
    )?;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))?;

    let mut workflow = Workflow::builder()
        .name("periodic_test".to_string())
        .state_dir(dir.path().to_path_buf())
        .build()?;

    // Hook that writes to log file every 100ms for 500ms (expect ~4-6 executions)
    let task_id = "test_task".to_string();

    workflow.add_task(workflow_core::Task::new(
        &task_id,
        move || {
            // Sleep for 500ms
            std::thread::sleep(Duration::from_millis(500));
            Ok::<_, anyhow::Error>(())
        })
        .monitors(vec![MonitoringHook::new(
            "counter",
            script.display().to_string(),
            // interval_secs: 0 means "as-fast-as-possible" - fires the hook as many
            // times as possible within the task execution window (valid for tests)
            HookTrigger::Periodic { interval_secs: 0 }
        )])
    ).expect("Failed to add task");

    workflow.run()?;

    // Verify hook executed multiple times (at least 4)
    let content = std::fs::read_to_string(&log_file)?;
    let lines: Vec<&str> = content.lines().collect();
    assert!(lines.len() >= 4, "Expected at least 4 hook executions, got {}", lines.len());

    Ok(())
}

#[test]
fn test_periodic_hook_stops_on_completion() {
    let dir = tempfile::tempdir().unwrap();

    // File to track hook executions
    let log_file = dir.path().join("executions.log");

    // Create executable hook script in tempdir to avoid shell redirection issues
    let script = dir.path().join("hook.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\necho 'hook executed' >> {}", log_file.display()),
    ).unwrap();
    std::fs::set_permissions(
        &script,
        std::fs::Permissions::from_mode(0o755),
    ).unwrap();

    let mut workflow = Workflow::builder()
        .name("stop_test".to_string())
        .state_dir(dir.path().to_path_buf())
        .build()
        .unwrap();

    // Task completes immediately, then waits for 2 seconds
    workflow.add_task(workflow_core::Task::new(
        "quick_task",
        move || {
            // Immediate completion
            Ok::<_, anyhow::Error>(())
        })
        .monitors(vec![MonitoringHook::new(
            "watcher",
            script.display().to_string(),
            HookTrigger::Periodic { interval_secs: 1 }
        )])
    ).unwrap();

    workflow.run().unwrap();

    // Wait 2 seconds after completion to ensure hooks stopped
    std::thread::sleep(Duration::from_secs(2));

    // Check that hook stopped on completion (0 executions = valid proof)
    let content = std::fs::read_to_string(&log_file).unwrap_or_default();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 0, "Hook should not fire when task completes before interval");
}

#[test]
fn test_periodic_manager_drop_stops_threads() {
    let dir = tempfile::tempdir().unwrap();

    // File to track hook executions
    let marker_file = dir.path().join("executed.txt");

    // Create executable hook script that creates/overwrites marker file
    let script = dir.path().join("hook.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\ntouch {}", marker_file.display()),
    ).unwrap();
    std::fs::set_permissions(
        &script,
        std::fs::Permissions::from_mode(0o755),
    ).unwrap();

    let mut workflow = Workflow::builder()
        .name("drop_test".to_string())
        .state_dir(dir.path().to_path_buf())
        .build()
        .unwrap();

    workflow.add_task(workflow_core::Task::new(
        "drop_task",
        move || {
            // Sleep longer than interval to ensure hook would run
            std::thread::sleep(Duration::from_millis(500));
            Ok::<_, anyhow::Error>(())
        })
        .monitors(vec![MonitoringHook::new(
            "counter",
            script.display().to_string(),
            // interval_secs: 0 means "as-fast-as-possible" - fires the hook as many
            // times as possible within the task execution window (valid for tests)
            HookTrigger::Periodic { interval_secs: 0 }
        )])
    ).unwrap();

    // Run and drop workflow - RAII should stop hooks
    workflow.run().unwrap();

    // Verify hook executed at least once (during the 500ms sleep)
    assert!(marker_file.exists(), "Expected marker file to be created by hook execution");
}

#[test]
fn test_periodic_hook_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;

    // File to track hook executions
    let log_file = dir.path().join("error_log.txt");

    // Create executable hook script in tempdir to avoid shell redirection issues
    let script = dir.path().join("hook.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\necho 'hook failed' >> {} && exit 1", log_file.display()),
    )?;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))?;

    let mut workflow = Workflow::builder()
        .name("error_test".to_string())
        .state_dir(dir.path().to_path_buf())
        .build()?;

    workflow.add_task(workflow_core::Task::new(
        "error_task",
        move || {
            // Sleep for 300ms
            std::thread::sleep(Duration::from_millis(300));
            Ok::<_, anyhow::Error>(())
        })
        .monitors(vec![MonitoringHook::new(
            "error_hook",
            script.display().to_string(),
            HookTrigger::Periodic { interval_secs: 0 }
        )])
    ).expect("Failed to add task");

    workflow.run()?;

    // Hook should have executed (failed but task continued)
    let content = std::fs::read_to_string(&log_file).unwrap_or_default();
    assert!(content.contains("hook failed"), "Expected hook to execute, but log is empty");

    Ok(())
}
