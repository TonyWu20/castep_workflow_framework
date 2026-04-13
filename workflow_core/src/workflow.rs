use crate::dag::Dag;
use crate::error::WorkflowError;
use crate::monitoring::HookExecutor;
use crate::process::{ProcessHandle, ProcessRunner};
use crate::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};
use crate::task::{ExecutionMode, Task};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
}

impl Workflow {
    /// Creates a new Workflow instance.
    pub fn new(name: impl Into<String>) -> Self {
        let max_parallel = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        Self {
            name: name.into(),
            tasks: HashMap::new(),
            max_parallel,
        }
    }

    /// Sets the maximum parallel execution limit.
    pub fn with_max_parallel(mut self, n: usize) -> Result<Self, WorkflowError> {
        if n == 0 {
            return Err(WorkflowError::InvalidConfig(
                "max_parallel must be at least 1".into(),
            ));
        }
        self.max_parallel = n;
        Ok(self)
    }

    pub fn add_task(&mut self, task: Task) -> Result<(), WorkflowError> {
        if self.tasks.contains_key(&task.id) {
            return Err(WorkflowError::DuplicateTaskId(task.id.clone()));
        }
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    pub fn dry_run(&self) -> Result<Vec<String>, WorkflowError> {
        Ok(self.build_dag()?.topological_order())
    }

    /// Runs the workflow with dependency injection for state, runner, and hook executor.
    pub fn run(
        &mut self,
        state: &mut JsonStateStore,
        runner: Arc<dyn ProcessRunner>,
        _hook_executor: Arc<dyn HookExecutor>,
    ) -> Result<WorkflowSummary, WorkflowError> {
        let dag = self.build_dag()?;

        // Initialize state for all tasks
        for id in dag.task_ids() {
            if state.get_status(id).is_none() {
                state.set_status(id, TaskStatus::Pending);
            }
        }
        state.save()?;

        let mut handles: HashMap<String, (Box<dyn ProcessHandle>, Instant)> = HashMap::new();
        let workflow_start = Instant::now();

        loop {
            // Poll finished tasks
            let mut finished: Vec<String> = Vec::new();
            for (id, handle) in handles.iter_mut() {
                if !handle.0.is_running() {
                    finished.push(id.clone());
                }
            }

            // Remove and process finished tasks
            for id in finished {
                if let Some((mut handle, start)) = handles.remove(&id) {
                    let _duration = start.elapsed();

                    // Execute the process and handle result
                    if let Ok(process_result) = handle.wait() {
                        match process_result.exit_code {
                            Some(0) => state.mark_completed(&id),
                            _ => state.mark_failed(
                                &id,
                                format!("exit code {}", process_result.exit_code.unwrap_or(-1)),
                            ),
                        }
                    } else {
                        state.mark_failed(&id, "process terminated".to_string());
                    }
                    state.save()?;
                }
            }

            // Skip propagation logic
            let mut changed = true;
            while changed {
                changed = false;
                let to_skip: Vec<String> = dag
                    .task_ids()
                    .filter(|id| matches!(state.get_status(*id), Some(TaskStatus::Pending)))
                    .filter(|id| {
                        self.tasks
                            .get(*id)
                            .map(|t| {
                                t.dependencies.iter().any(|dep| {
                                    matches!(
                                        state.get_status(dep.as_str()),
                                        Some(TaskStatus::Failed { .. })
                                            | Some(TaskStatus::Skipped)
                                            | Some(TaskStatus::SkippedDueToDependencyFailure)
                                    )
                                })
                            })
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect();
                if !to_skip.is_empty() {
                    changed = true;
                    for id in to_skip {
                        state.mark_skipped_due_to_dep_failure(&id);
                    }
                }
            }
            state.save()?;

            // Dispatch ready tasks
            let done_set: HashSet<String> = state
                .all_tasks()
                .iter()
                .filter(|(_, v)| {
                    matches!(
                        v,
                        TaskStatus::Completed
                            | TaskStatus::Skipped
                            | TaskStatus::SkippedDueToDependencyFailure
                    )
                })
                .map(|(k, _)| k.clone())
                .collect();

            for id in dag.ready_tasks(&done_set) {
                if handles.len() >= self.max_parallel {
                    break;
                }
                if matches!(state.get_status(&id), Some(TaskStatus::Pending)) {
                    // Take task from HashMap (consume it)
                    if let Some(task) = self.tasks.remove(&id) {
                        state.mark_running(&id);

                        // Execute setup closure if present
                        if let Some(setup) = &task.setup {
                            if let Err(e) = setup(&task.workdir) {
                                state.mark_failed(&id, e.to_string());
                                state.save()?;
                                continue;
                            }
                        }

                        // Spawn process via runner
                        match &task.mode {
                            ExecutionMode::Direct {
                                command,
                                args,
                                env,
                                timeout: _,
                            } => {
                                let handle = runner.spawn(&task.workdir, command, args, env)?;
                                handles.insert(id.clone(), (handle, Instant::now()));
                            }
                            ExecutionMode::Queued { .. } => {
                                return Err(WorkflowError::Io(std::io::Error::other(
                                    "queued execution not yet implemented",
                                )));
                            }
                        }
                    }
                }
            }

            // Check if all done
            let all_done = dag.task_ids().all(|id| {
                matches!(
                    state.get_status(id),
                    Some(TaskStatus::Completed)
                        | Some(TaskStatus::Failed { .. })
                        | Some(TaskStatus::Skipped)
                        | Some(TaskStatus::SkippedDueToDependencyFailure)
                )
            });

            if all_done && handles.is_empty() {
                break;
            }

            std::thread::sleep(Duration::from_millis(50));
        }

        // Build WorkflowSummary
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        let mut skipped = Vec::new();

        for (id, status) in state.all_tasks() {
            match status {
                TaskStatus::Completed => succeeded.push(id.clone()),
                TaskStatus::Failed { error } => failed.push((id.clone(), error.clone())),
                TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure => {
                    skipped.push(id.clone())
                }
                _ => {}
            }
        }

        Ok(WorkflowSummary {
            succeeded,
            failed,
            skipped,
            duration: workflow_start.elapsed(),
        })
    }

    fn build_dag(&self) -> Result<Dag, WorkflowError> {
        let mut dag = Dag::new();
        for id in self.tasks.keys() {
            dag.add_node(id.clone())?;
        }
        for task in self.tasks.values() {
            for dep in &task.dependencies {
                dag.add_edge(dep, &task.id)?;
            }
        }
        Ok(dag)
    }
}

/// Summary of workflow execution results.
#[derive(Debug, Clone)]
pub struct WorkflowSummary {
    pub succeeded: Vec<String>,
    pub failed: Vec<(String, String)>, // (task_id, error_message)
    pub skipped: Vec<String>,
    pub duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::ShellHookExecutor;
    use crate::process::SystemProcessRunner;
    use std::collections::HashMap;
    use std::io::Write;

    #[test]
    fn single_task_completes() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir().unwrap();

        let mut wf = Workflow::new("wf_single").with_max_parallel(4)?;

        wf.add_task(Task::new(
            "a",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        ))
        .unwrap();

        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner);
        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
        let state_path = dir.path().join(".wf_single.workflow.json");
        let mut state = Box::new(JsonStateStore::new("wf_single", state_path));

        let summary = wf.run(state.as_mut(), runner, executor)?;
        assert_eq!(summary.succeeded.len(), 1);
        assert!(summary.failed.is_empty());
        Ok(())
    }

    #[test]
    fn chain_respects_order() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir().unwrap();
        let log_file = dir.path().join("log.txt");
        let log_for_a = log_file.clone();
        let log_for_b = log_file.clone();

        let mut wf = Workflow::new("wf_chain").with_max_parallel(4)?;

        wf.add_task(
            Task::new(
                "a",
                ExecutionMode::Direct {
                    command: "true".into(),
                    args: vec![],
                    env: HashMap::new(),
                    timeout: None,
                },
            )
            .setup(move |_| {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_for_a)?;
                writeln!(f, "a")?;
                Ok(())
            }),
        )
        .unwrap();

        wf.add_task(
            Task::new(
                "b",
                ExecutionMode::Direct {
                    command: "true".into(),
                    args: vec![],
                    env: HashMap::new(),
                    timeout: None,
                },
            )
            .depends_on("a")
            .setup(move |_| {
                let mut f = std::fs::OpenOptions::new().append(true).open(&log_for_b)?;
                writeln!(f, "b")?;
                Ok(())
            }),
        )
        .unwrap();

        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner);
        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
        let state_path = dir.path().join(".wf_chain.workflow.json");
        let mut state = Box::new(JsonStateStore::new("wf_chain", state_path));

        wf.run(state.as_mut(), runner, executor)?;

        let log = std::fs::read_to_string(&log_file).unwrap();
        assert_eq!(log.lines().collect::<Vec<_>>(), vec!["a", "b"]);
        Ok(())
    }

    #[test]
    fn failed_task_skips_dependent() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir().unwrap();

        let mut wf = Workflow::new("wf_skip").with_max_parallel(4)?;

        wf.add_task(Task::new(
            "a",
            ExecutionMode::Direct {
                command: "false".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        ))
        .unwrap();

        wf.add_task(
            Task::new(
                "b",
                ExecutionMode::Direct {
                    command: "true".into(),
                    args: vec![],
                    env: HashMap::new(),
                    timeout: None,
                },
            )
            .depends_on("a"),
        )
        .unwrap();

        let runner: Arc<dyn ProcessRunner> = Arc::new(SystemProcessRunner);
        let executor: Arc<dyn HookExecutor> = Arc::new(ShellHookExecutor);
        let state_path = dir.path().join(".wf_skip.workflow.json");
        let mut state = Box::new(JsonStateStore::new("wf_skip", state_path.clone()));

        wf.run(state.as_mut(), runner, executor)?;

        let state = JsonStateStore::load(state_path).unwrap();
        assert!(matches!(
            state.get_status("b"),
            Some(TaskStatus::SkippedDueToDependencyFailure)
        ));
        Ok(())
    }

    #[test]
    fn dry_run_returns_topo_order() -> Result<(), Box<dyn std::error::Error>> {
        let mut wf = Workflow::new("wf_dry");

        wf.add_task(Task::new(
            "a",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        ))
        .unwrap();

        wf.add_task(
            Task::new(
                "b",
                ExecutionMode::Direct {
                    command: "true".into(),
                    args: vec![],
                    env: HashMap::new(),
                    timeout: None,
                },
            )
            .depends_on("a"),
        )
        .unwrap();

        let order = wf.dry_run()?;
        let pa = order.iter().position(|x| x == "a").unwrap();
        let pb = order.iter().position(|x| x == "b").unwrap();
        assert!(pa < pb);
        Ok(())
    }

    #[test]
    fn duplicate_task_id_errors() -> Result<(), Box<dyn std::error::Error>> {
        let mut wf = Workflow::new("wf_dup");

        wf.add_task(Task::new(
            "a",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        ))
        .unwrap();

        assert_eq!(
            wf.add_task(Task::new(
                "a",
                ExecutionMode::Direct {
                    command: "true".into(),
                    args: vec![],
                    env: HashMap::new(),
                    timeout: None,
                },
            )),
            Err(WorkflowError::DuplicateTaskId("a".to_string()))
        );
        Ok(())
    }

    #[test]
    fn valid_dependency_add() -> Result<(), Box<dyn std::error::Error>> {
        let mut wf = Workflow::new("wf_dep");

        wf.add_task(Task::new(
            "a",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        ))
        .unwrap();

        assert!(wf
            .add_task(
                Task::new(
                    "b",
                    ExecutionMode::Direct {
                        command: "true".into(),
                        args: vec![],
                        env: HashMap::new(),
                        timeout: None,
                    },
                )
                .depends_on("a")
            )
            .is_ok());
        Ok(())
    }

    #[test]
    fn builder_with_custom_max_parallel() {
        let wf = Workflow::new("test").with_max_parallel(4).unwrap();
        assert_eq!(wf.max_parallel, 4);
    }

    #[test]
    fn builder_validation_zero_parallelism() {
        let result = Workflow::new("test").with_max_parallel(0);
        assert!(result.is_err());
    }

    #[test]
    fn resume_loads_existing_state() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join(".wf_resume.workflow.json");

        // First run
        let mut state1 = Box::new(JsonStateStore::new("wf_resume", state_path.clone()));
        let mut wf1 = Workflow::new("wf_resume");
        wf1.add_task(Task::new(
            "a",
            ExecutionMode::Direct {
                command: "true".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        ))
        .unwrap();
        wf1.run(
            state1.as_mut(),
            Arc::new(SystemProcessRunner),
            Arc::new(ShellHookExecutor),
        )?;

        // Second run (resume)
        let mut state2 = Box::new(JsonStateStore::load(&state_path).unwrap());
        let mut wf2 = Workflow::new("wf_resume");
        wf2.add_task(Task::new(
            "a",
            ExecutionMode::Direct {
                command: "false".into(),
                args: vec![],
                env: HashMap::new(),
                timeout: None,
            },
        ))
        .unwrap();
        wf2.run(
            state2.as_mut(),
            Arc::new(SystemProcessRunner),
            Arc::new(ShellHookExecutor),
        )?;

        // Task "a" should still be Completed (not re-run)
        assert!(state2.is_completed("a"));
        Ok(())
    }
}
