use clap::{Parser, Subcommand};
use std::io::{self, IsTerminal, Read};
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
        #[arg(required = false, default_value = "-")]
        task_ids: Vec<String>,
    },
    Inspect {
        state_file: String,
        task_id: Option<String>,
    },
}

/// Resolve task IDs from CLI args or stdin.
///
/// - Non-empty `task_ids` with first element != "-" → use as-is
/// - `["-"]` or empty + piped input → read stdin (one ID per line)
/// - Empty + TTY → usage error
fn read_task_ids(task_ids: &[String]) -> anyhow::Result<Vec<String>> {
    if task_ids.first().map(|s| s.as_str()) == Some("-") || task_ids.is_empty() {
        let mut input = String::new();
        if io::stdin().is_terminal() {
            anyhow::bail!(
                "no task IDs specified and stdin is a terminal; \
                 provide IDs as arguments or pipe them via stdin"
            );
        }
        io::stdin().read_to_string(&mut input).map_err(|e| {
            anyhow::anyhow!("failed to read stdin: {}", e)
        })?;
        let ids: Vec<String> = input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect();
        if ids.is_empty() {
            anyhow::bail!("no task IDs found in stdin");
        }
        Ok(ids)
    } else {
        Ok(task_ids.to_vec())
    }
}

fn load_state_raw(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load_raw(path)
        .map_err(|e| anyhow::anyhow!("failed to open state file '{}': {}", path, e))
}

fn load_state_for_resume(path: &str) -> anyhow::Result<JsonStateStore> {
    JsonStateStore::load(path)
        .map_err(|e| anyhow::anyhow!("failed to open state file '{}': {}", path, e))
}

fn cmd_status(state: &dyn StateStore) -> String {
    let mut tasks: Vec<(String, TaskStatus)> = state.all_tasks();
    tasks.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::new();
    for (id, status) in &tasks {
        match status {
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

fn cmd_retry(state: &mut JsonStateStore, task_ids: &[String]) -> anyhow::Result<()> {
    for id in task_ids {
        if state.get_status(id).is_none() {
            eprintln!("warn: task '{}' not found", id);
        } else {
            state.mark_pending(id);
        }
    }

    match state.task_successors() {
        None => {
            eprintln!("warn: state file lacks dependency info; falling back to global reset");
            let to_reset: Vec<String> = state
                .all_tasks()
                .into_iter()
                .filter(|(_, s)| matches!(s, TaskStatus::SkippedDueToDependencyFailure))
                .map(|(id, _)| id)
                .collect();
            for id in to_reset {
                state.mark_pending(&id);
            }
        }
        Some(successors) => {
            let downstream = successors.downstream_of(task_ids);
            let to_reset: Vec<String> = state
                .all_tasks()
                .into_iter()
                .filter(|(id, s)| {
                    matches!(s, TaskStatus::SkippedDueToDependencyFailure)
                        && downstream.contains(id)
                })
                .map(|(id, _)| id)
                .collect();
            for id in to_reset {
                state.mark_pending(&id);
            }
        }
    }

    state.save().map_err(|e| anyhow::anyhow!("failed to save state: {}", e))?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Status { state_file } => {
            let state = load_state_raw(&state_file)?;
            println!("{}", cmd_status(&state));
            Ok(())
        }
        Commands::Retry { state_file, task_ids } => {
            let resolved = read_task_ids(&task_ids)?;
            let mut state = load_state_for_resume(&state_file)?;
            cmd_retry(&mut state, &resolved)?;
            Ok(())
        }
        Commands::Inspect { state_file, task_id } => {
            let state = load_state_raw(&state_file)?;
            let out = cmd_inspect(&state, task_id.as_deref())?;
            println!("{}", out);
            Ok(())
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
        cmd_retry(&mut s, &["task_b".to_string()]).unwrap();
        assert!(matches!(s.get_status("task_b"), Some(TaskStatus::Pending)));
        assert!(matches!(s.get_status("task_c"), Some(TaskStatus::Pending)));
        assert!(matches!(s.get_status("task_a"), Some(TaskStatus::Completed))); // unchanged
    }

    #[test]
    fn read_task_ids_from_vec() {
        let ids = read_task_ids(&["a".to_string(), "b".to_string()]).unwrap();
        assert_eq!(ids, vec!["a", "b"]);
    }

    #[test]
    fn read_task_ids_dash_empty_stdin_errors() {
        // "-" enters stdin mode; with empty stdin it should error (not hang).
        // In cargo test, stdin is a pipe (not a TTY), so read_to_string
        // returns immediately with empty string, triggering the bail.
        let result = read_task_ids(&["-".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no task IDs found"));
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
    fn status_shows_failed_after_load_raw() {
        let dir = tempfile::tempdir().unwrap();
        let s = make_state(dir.path());
        s.save().unwrap();
        let loaded = JsonStateStore::load_raw(dir.path().join("state.json").to_str().unwrap()).unwrap();
        let out = cmd_status(&loaded);
        assert!(out.contains("task_b: Failed (exit code 1)"));
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
