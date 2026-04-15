use crate::dag::Dag;
use crate::error::WorkflowError;
use crate::process::{ProcessHandle, ProcessRunner};
use crate::state::{StateStore, StateStoreExt, TaskStatus};
use crate::task::{ExecutionMode, Task, TaskClosure};
use crate::HookExecutor;

/// A handle to a running task with metadata.
pub(crate) type TaskHandle = (
    Box<dyn ProcessHandle>,
    Instant,
    Vec<crate::monitoring::MonitoringHook>,
    Option<TaskClosure>,
    std::path::PathBuf,
);
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    max_parallel: usize,
    pub(crate) interrupt: Arc<AtomicBool>,
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
            interrupt: Arc::new(AtomicBool::new(false)),
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
    ///
    /// # Panics (debug only)
    /// Asserts that the workflow has tasks. Tasks are consumed from the `Workflow` on dispatch;
    /// calling `run()` twice on the same instance will silently process no tasks on the second call.
    /// Construct a new `Workflow` to re-run.
    pub fn run(
        &mut self,
        state: &mut dyn StateStore,
        runner: Arc<dyn ProcessRunner>,
        hook_executor: Arc<dyn HookExecutor>,
    ) -> Result<WorkflowSummary, WorkflowError> {
        debug_assert!(
            !self.tasks.is_empty(),
            "run() called on a Workflow with no tasks — tasks are consumed on dispatch"
        );
        signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&self.interrupt)).ok();
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&self.interrupt)).ok();

        let dag = self.build_dag()?;

        // Initialize state for all tasks
        for id in dag.task_ids() {
            if state.get_status(id).is_none() {
                state.set_status(id, TaskStatus::Pending);
            }
        }
        state.save()?;

        let mut handles: HashMap<String, TaskHandle> = HashMap::new();
        let workflow_start = Instant::now();

        // Task timeout tracking
        let mut task_timeouts: HashMap<String, Duration> = HashMap::new();

        loop {
            // Interrupt check — must be first
            if self.interrupt.load(Ordering::SeqCst) {
                for id in handles.keys() {
                    state.set_status(id, TaskStatus::Pending);
                }
                for (_, (handle, _start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
                    handle.terminate().ok();
                }
                state.save()?;
                return Err(WorkflowError::Interrupted);
            }

            // Poll finished tasks
            let mut finished: Vec<String> = Vec::new();
            for (id, (handle, start, _monitors, _collect_fn, _workdir)) in handles.iter_mut() {
                // Timeout check first
                if let Some(&timeout) = task_timeouts.get(id) {
                    if start.elapsed() >= timeout {
                        handle.terminate().ok();
                        state.mark_failed(
                            id,
                            WorkflowError::TaskTimeout(id.clone()).to_string(),
                        );
                        state.save()?;
                        finished.push(id.clone());
                        continue;
                    }
                }
                if !handle.is_running() {
                    finished.push(id.clone());
                }
            }

            // Remove and process finished tasks
            for id in finished {
                if let Some((mut handle, start, monitors, collect_fn, workdir)) = handles.remove(&id) {
                    // Skip wait() if already marked failed (e.g., timed out)
                    if matches!(state.get_status(&id), Some(TaskStatus::Failed { .. })) {
                        continue;
                    }

                    let _duration = start.elapsed();

                    // Execute the process and handle result
                    let (final_state, exit_code) = if let Ok(process_result) = handle.wait() {
                        match process_result.exit_code {
                            Some(0) => {
                                state.mark_completed(&id);
                                if let Some(ref collect) = collect_fn {
                                    if let Err(e) = collect(&workdir) {
                                        tracing::warn!(
                                            "Collect closure for task '{}' failed: {}",
                                            id,
                                            e
                                        );
                                    }
                                }
                                ("completed", process_result.exit_code)
                            }
                            _ => {
                                state.mark_failed(
                                    &id,
                                    format!("exit code {}", process_result.exit_code.unwrap_or(-1)),
                                );
                                ("failed", process_result.exit_code)
                            }
                        }
                    } else {
                        state.mark_failed(&id, "process terminated".to_string());
                        ("failed", None)
                    };

                    // Fire OnComplete/OnFailure hooks
                    let ctx = crate::monitoring::HookContext {
                        task_id: id.clone(),
                        workdir,
                        state: final_state.to_string(),
                        exit_code,
                    };
                    for hook in &monitors {
                        let should_fire = matches!(
                            (&hook.trigger, final_state),
                            (crate::monitoring::HookTrigger::OnComplete, "completed")
                                | (crate::monitoring::HookTrigger::OnFailure, "failed")
                        );
                        if should_fire {
                            if let Err(e) = hook_executor.execute_hook(hook, &ctx) {
                                tracing::warn!(
                                    "Hook '{}' for task '{}' failed: {}",
                                    hook.name,
                                    id,
                                    e
                                );
                            }
                        }
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
                    .filter(|id| matches!(state.get_status(id), Some(TaskStatus::Pending)))
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
                .into_iter()
                .filter(|(_, v)| {
                    matches!(
                        v,
                        TaskStatus::Completed
                            | TaskStatus::Skipped
                            | TaskStatus::SkippedDueToDependencyFailure
                    )
                })
                .map(|(k, _)| k)
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
                                timeout,
                            } => {
                                // Register timeout if specified
                                if let Some(d) = timeout {
                                    task_timeouts.insert(id.clone(), *d);
                                }

                                let monitors = task.monitors.clone();
                                let task_workdir = task.workdir.clone();
                                let handle = runner.spawn(&task.workdir, command, args, env)?;

                                // Fire OnStart hooks
                                let ctx = crate::monitoring::HookContext {
                                    task_id: id.clone(),
                                    workdir: task_workdir,
                                    state: "running".to_string(),
                                    exit_code: None,
                                };
                                for hook in &monitors {
                                    if matches!(
                                        hook.trigger,
                                        crate::monitoring::HookTrigger::OnStart
                                    ) {
                                        if let Err(e) = hook_executor.execute_hook(hook, &ctx) {
                                            tracing::warn!(
                                                "OnStart hook '{}' for task '{}' failed: {}",
                                                hook.name,
                                                id,
                                                e
                                            );
                                        }
                                    }
                                }

                                handles.insert(id.clone(), (handle, Instant::now(), monitors, task.collect, task.workdir.clone()));
                            }
                            ExecutionMode::Queued { .. } => {
                                return Err(WorkflowError::InvalidConfig(
                                    "Queued execution mode is not yet implemented".into(),
                                ));
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
                TaskStatus::Completed => succeeded.push(id),
                TaskStatus::Failed { error } => failed.push((id, error)),
                TaskStatus::Skipped | TaskStatus::SkippedDueToDependencyFailure => {
                    skipped.push(id)
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
    use crate::state::JsonStateStore;
    use std::collections::HashMap;
    use std::io::Write;

    struct StubRunner;
    impl ProcessRunner for StubRunner {
        fn spawn(
            &self,
            workdir: &std::path::Path,
            command: &str,
            args: &[String],
            env: &HashMap<String, String>,
        ) -> Result<Box<dyn ProcessHandle>, WorkflowError> {
            let child = std::process::Command::new(command)
                .args(args)
                .envs(env)
                .current_dir(workdir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .map_err(WorkflowError::Io)?;
            Ok(Box::new(StubHandle {
                child: Some(child),
                start: std::time::Instant::now(),
            }))
        }
    }

    struct StubHandle {
        child: Option<std::process::Child>,
        start: std::time::Instant,
    }

    impl ProcessHandle for StubHandle {
        fn is_running(&mut self) -> bool {
            match &mut self.child {
                Some(child) => child.try_wait().ok().flatten().is_none(),
                None => false,
            }
        }
        fn terminate(&mut self) -> Result<(), WorkflowError> {
            match &mut self.child {
                Some(child) => child.kill().map_err(WorkflowError::Io),
                None => Ok(()),
            }
        }
        fn wait(&mut self) -> Result<crate::process::ProcessResult, WorkflowError> {
            let child = self
                .child
                .take()
                .ok_or_else(|| WorkflowError::InvalidConfig("wait() called twice".into()))?;
            let output = child.wait_with_output().map_err(WorkflowError::Io)?;
            Ok(crate::process::ProcessResult {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                duration: self.start.elapsed(),
            })
        }
    }

    struct StubHookExecutor;
    impl HookExecutor for StubHookExecutor {
        fn execute_hook(
            &self,
            _hook: &crate::monitoring::MonitoringHook,
            _ctx: &crate::monitoring::HookContext,
        ) -> Result<crate::monitoring::HookResult, WorkflowError> {
            Ok(crate::monitoring::HookResult {
                success: true,
                output: String::new(),
            })
        }
    }

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

        let runner: Arc<dyn ProcessRunner> = Arc::new(StubRunner);
        let executor: Arc<dyn HookExecutor> = Arc::new(StubHookExecutor);
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

        let runner: Arc<dyn ProcessRunner> = Arc::new(StubRunner);
        let executor: Arc<dyn HookExecutor> = Arc::new(StubHookExecutor);
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

        let runner: Arc<dyn ProcessRunner> = Arc::new(StubRunner);
        let executor: Arc<dyn HookExecutor> = Arc::new(StubHookExecutor);
        let state_path = dir.path().join(".wf_skip.workflow.json");
        let mut state = Box::new(JsonStateStore::new("wf_skip", state_path.clone()));

        wf.run(state.as_mut(), runner, executor)?;

        // Verify in-memory state shows skip propagation actually worked
        assert!(matches!(
            state.get_status("b"),
            Some(TaskStatus::SkippedDueToDependencyFailure)
        ));

        let state = JsonStateStore::load(state_path).unwrap();
        // After load, SkippedDueToDependencyFailure resets to Pending for crash recovery
        assert!(matches!(state.get_status("b"), Some(TaskStatus::Pending)));
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

        assert!(matches!(
            wf.add_task(Task::new(
                "a",
                ExecutionMode::Direct {
                    command: "true".into(),
                    args: vec![],
                    env: HashMap::new(),
                    timeout: None,
                },
            )),
            Err(WorkflowError::DuplicateTaskId(_))
        ));
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
            Arc::new(StubRunner),
            Arc::new(StubHookExecutor),
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
            Arc::new(StubRunner),
            Arc::new(StubHookExecutor),
        )?;

        // Task "a" should still be Completed (not re-run)
        assert!(state2.is_completed("a"));
        Ok(())
    }

    #[test]
    fn interrupt_before_run_dispatches_nothing() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir().unwrap();
        let mut wf = Workflow::new("wf_interrupt").with_max_parallel(4)?;
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
        wf.interrupt.store(true, Ordering::SeqCst);
        let mut state = JsonStateStore::new(
            "wf_interrupt",
            dir.path().join(".wf_interrupt.workflow.json"),
        );
        let result = wf.run(&mut state, Arc::new(StubRunner), Arc::new(StubHookExecutor));
        assert!(matches!(result.unwrap_err(), WorkflowError::Interrupted));
        assert!(!matches!(
            state.get_status("a"),
            Some(TaskStatus::Completed)
        ));
        Ok(())
    }

    #[test]
    fn interrupt_mid_run_stops_dispatch() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir().unwrap();
        let mut wf = Workflow::new("wf_interrupt2").with_max_parallel(4)?;
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);
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
                flag_clone.store(true, Ordering::SeqCst);
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
            .depends_on("a"),
        )
        .unwrap();
        wf.interrupt = Arc::clone(&flag);
        let mut state = JsonStateStore::new(
            "wf_interrupt2",
            dir.path().join(".wf_interrupt2.workflow.json"),
        );
        let result = wf.run(&mut state, Arc::new(StubRunner), Arc::new(StubHookExecutor));
        assert!(matches!(result.unwrap_err(), WorkflowError::Interrupted));
        Ok(())
    }
}
