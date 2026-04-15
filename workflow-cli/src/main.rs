use clap::{Parser, Subcommand};
use workflow_core::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};

#[derive(Parser)]
#[command(name = "workflow-cli", about = "Workflow state inspection tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status { state_file: String },
    Retry {
        state_file: String,
        #[arg(required = true)]
        task_ids: Vec<String>,
    },
    Inspect {
        state_file: String,
        task_id: Option<String>,
    },
}

fn load_state(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load(path)
        .map_err(|_| anyhow::anyhow!("error: state file not found: {}", path))
}

fn cmd_status(state: &JsonStateStore) -> String {
    let mut tasks: Vec<(String, TaskStatus)> = state.all_tasks();
    tasks.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    for (id, status) in &tasks {
        match status.clone() {
            TaskStatus::Failed { error } => out.push_str(&format!("{}: Failed ({})\n", id, error)),
            other => out.push_str(&format!("{}: {:?}\n", id, other)),
        }
    }
    let s = state.summary();
    out.push_str(&format!(
        "Summary: {} completed, {} failed, {} skipped, {} pending",
        s.completed, s.failed, s.skipped, s.pending
    ));
    out
}

fn cmd_inspect(state: &dyn StateStore, task_id: Option<&str>) -> anyhow::Result<String> {
    match task_id {
        Some(id) => match state.get_status(id) {
            None => anyhow::bail!("task '{}' not found", id),
            Some(TaskStatus::Failed { error }) =>
                Ok(format!("task: {}\nstatus: Failed\nerror: {}", id, error)),
            Some(s) => Ok(format!("task: {}\nstatus: {:?}", id, s)),
        },
        None => {
            let mut tasks: Vec<(String, TaskStatus)> = state.all_tasks();
            tasks.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(tasks.iter()
                .map(|(id, s)| format!("{}: {:?}", id, s))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }
}

fn cmd_retry(state: &mut dyn StateStore, task_ids: &[String]) -> anyhow::Result<()> {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }
    let to_reset: Vec<String> = state
        .all_tasks()
        .into_iter()
        .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
        .map(|(id, _)| id)
        .collect();
    for id in to_reset {
        state.mark_pending(&id);
    }
    state.save().map_err(|e| anyhow::anyhow!("failed to save state: {}", e))?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status { state_file } => {
            let state = load_state(&state_file)?;
            println!("{}", cmd_status(&state));
            Ok(())
        }
        Commands::Retry { state_file, task_ids } => {
            let mut state = load_state(&state_file)?;
            cmd_retry(&mut state, &task_ids)?;
            Ok(())
        }
        Commands::Inspect { state_file, task_id } => {
            let state = load_state(&state_file)?;
            match cmd_inspect(&state, task_id.as_deref()) {
                Ok(out) => { println!("{}", out); Ok(()) }
                Err(e) => { eprintln!("{}", e); std::process::exit(1); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use workflow_core::state::StateStoreExt;

    fn make_state(dir: &std::path::Path) -> JsonStateStore {
        let mut s = JsonStateStore::new("test_wf", dir.join("state.json"));
        s.mark_completed("task_a");
        s.mark_failed("task_b", "exit code 1".into());
        s.mark_skipped_due_to_dep_failure("task_c");
        s.save().unwrap();
        s
    }

    #[test]
    fn retry_resets_failed_and_skipped_dep() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = make_state(dir.path());
        // task_b=Failed, task_c=SkippedDueToDependencyFailure, task_a=Completed
        let _ = cmd_retry(&mut s, &["task_b".to_string()]);
        assert!(matches!(s.get_status("task_b"), Some(TaskStatus::Pending)));
        assert!(matches!(s.get_status("task_c"), Some(TaskStatus::Pending)));
        assert!(matches!(s.get_status("task_a"), Some(TaskStatus::Completed))); // unchanged
    }

    #[test]
    fn status_output_format() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        let out = cmd_status(&s);
        assert!(out.contains("task_a: Completed"));
        assert!(out.contains("task_b: Failed (exit code 1)"));
        assert!(out.contains("Summary: 1 completed, 1 failed, 1 skipped, 0 pending"));
    }

    #[test]
    fn inspect_single_task() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        let out = cmd_inspect(&s, Some("task_b")).unwrap();
        assert_eq!(out, "task: task_b\nstatus: Failed\nerror: exit code 1");
    }

    #[test]
    fn inspect_unknown_task_errors() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        assert!(cmd_inspect(&s, Some("nonexistent")).is_err());
    }
}
