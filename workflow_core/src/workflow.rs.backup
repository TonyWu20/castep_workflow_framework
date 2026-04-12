use crate::dag::Dag;
use crate::state::{TaskStatus, WorkflowState};
use crate::task::Task;
use anyhow::{bail, Result};
use bon::bon;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex};
use std::time::{Duration, Instant};
use workflow_utils::{HookContext, HookTrigger};

struct PeriodicHookHandle {
    thread: std::thread::JoinHandle<()>,
    stop_signal: Arc<AtomicBool>,
}

struct PeriodicHookManager {
    handles: HashMap<String, Vec<PeriodicHookHandle>>,
}

impl PeriodicHookManager {
    fn new() -> Self {
        Self { handles: HashMap::new() }
    }

    fn spawn_for_task(
        &mut self,
        task_id: String,
        hooks: Vec<workflow_utils::MonitoringHook>,
        ctx: HookContext,
    ) {
        let mut task_handles = Vec::new();

        for hook in hooks {
            if let HookTrigger::Periodic { interval_secs } = hook.trigger {
                let stop = Arc::new(AtomicBool::new(false));
                let stop_clone = stop.clone();
                let hook_clone = hook.clone();
                let ctx_clone = ctx.clone();

                let thread = std::thread::spawn(move || {
                    while !stop_clone.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_secs(interval_secs));
                        if stop_clone.load(Ordering::Relaxed) {
                            break;
                        }

                        match hook_clone.execute(&ctx_clone) {
                            Ok(result) => {
                                tracing::info!(
                                    hook_name = %hook_clone.name,
                                    task_id = %ctx_clone.task_id,
                                    "Hook output:\n{}",
                                    Self::indent_output(&result.output)
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    hook_name = %hook_clone.name,
                                    error = %e,
                                    "Hook failed (task continues)"
                                );
                            }
                        }
                    }
                });

                task_handles.push(PeriodicHookHandle {
                    thread,
                    stop_signal: stop,
                });
            }
        }

        if !task_handles.is_empty() {
            self.handles.insert(task_id, task_handles);
        }
    }

    fn stop_for_task(&mut self, task_id: &str) {
        if let Some(handles) = self.handles.remove(task_id) {
            for handle in handles {
                handle.stop_signal.store(true, Ordering::Relaxed);
                let _ = handle.thread.join();
            }
        }
    }

    fn indent_output(output: &str) -> String {
        if output.is_empty() {
            return "<no output>".to_string();
        }
        output.lines()
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Drop for PeriodicHookManager {
    fn drop(&mut self) {
        for (task_id, handles) in self.handles.drain() {
            tracing::debug!("Stopping periodic hooks for task: {}", task_id);
            for handle in handles {
                handle.stop_signal.store(true, Ordering::Relaxed);
                let _ = handle.thread.join();
            }
        }
    }
}

pub struct Workflow {
    pub name: String,
    tasks: HashMap<String, Task>,
    state_path: PathBuf,
    max_parallel: usize,
}

#[bon]
impl Workflow {
    /// Creates a new Workflow instance using the builder pattern.
    #[builder]
    pub fn new(
        name: String,
        #[builder(default = PathBuf::from("."))] state_dir: PathBuf,
        max_parallel: Option<usize>,
    ) -> Result<Self> {
        let max_parallel = max_parallel.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        });

        if max_parallel == 0 {
            bail!("max_parallel must be at least 1");
        }

        let state_path = state_dir.join(format!(".{}.workflow.json", name));
        Ok(Self {
            name,
            tasks: HashMap::new(),
            state_path,
            max_parallel,
        })
    }

    pub fn add_task(&mut self, task: Task) -> Result<()> {
        if self.tasks.contains_key(&task.id) {
            bail!("duplicate task id: {}", task.id);
        }
        self.tasks.insert(task.id.clone(), task);
        Ok(())
    }

    pub fn dry_run(&self) -> Result<Vec<String>> {
        Ok(self.build_dag()?.topological_order())
    }

    /// Resume a workflow by name, loading prior state from `state_dir` when `run()` is called.
    pub fn resume(name: impl Into<String>, state_dir: impl Into<PathBuf>) -> Result<Self> {
        Self::builder()
            .name(name.into())
            .state_dir(state_dir.into())
            .build()
    }

    pub fn run(&mut self) -> Result<()> {
        let dag = self.build_dag()?;
        tracing::debug!("DAG execution order: {:?}", dag.topological_order());

        let mut state = if self.state_path.exists() {
            WorkflowState::load(&self.state_path)?
        } else {
            WorkflowState::new(&self.name)
        };
        for id in dag.task_ids() {
            state.tasks.entry(id.clone()).or_insert(TaskStatus::Pending);
        }
        let state = Arc::new(Mutex::new(state));

        tracing::debug!("Registered {} tasks", self.tasks.len());

        let fns: HashMap<String, Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>> = self
            .tasks
            .iter()
            .map(|(id, t)| {
                tracing::debug!("Registered task '{}' with {} dependencies", id, t.dependencies.len());
                (id.clone(), Arc::clone(&t.execute_fn))
            })
            .collect();

        let monitors: HashMap<String, Vec<workflow_utils::MonitoringHook>> = self
            .tasks
            .iter()
            .map(|(id, t)| (id.clone(), t.monitors.clone()))
            .collect();

        let task_workdirs: HashMap<String, std::path::PathBuf> = self
            .tasks
            .iter()
            .map(|(id, t)| (id.clone(), t.workdir.clone()))
            .collect();

        let mut handles: HashMap<String, std::thread::JoinHandle<anyhow::Result<()>>> =
            HashMap::new();

        let workflow_start = Instant::now();
        let mut task_start_times: HashMap<String, Instant> = HashMap::new();
        let mut periodic_manager = PeriodicHookManager::new();

        tracing::info!(
            workflow_name = %self.name,
            total_tasks = self.tasks.len(),
            max_parallel = self.max_parallel,
            "Starting workflow"
        );

        loop {
            let finished: Vec<String> = handles
                .keys()
                .filter(|id| handles[*id].is_finished())
                .cloned()
                .collect();
            for id in finished {
                // task threads don't hold the lock when they panic — poisoning is not expected
                let mut s = state.lock().unwrap();
                let result = handles
                    .remove(&id)
                    .expect("id was just confirmed present in finished list")
                    .join()
                    .unwrap_or_else(|_| Err(anyhow::anyhow!("thread panicked")));
                match result {
                    Ok(()) => {
                        // Stop periodic hooks first
                        periodic_manager.stop_for_task(&id);

                        let duration = task_start_times.remove(&id)
                            .map(|start| start.elapsed())
                            .unwrap_or(Duration::from_secs(0));

                        s.mark_completed(&id);

                        tracing::info!(
                            task_id = %id,
                            duration_secs = duration.as_secs(),
                            "Task completed in {}",
                            Self::format_duration(duration)
                        );

                        if let Some(hooks) = monitors.get(&id) {
                            let ctx = HookContext {
                                task_id: id.clone(),
                                workdir: task_workdirs[&id].clone(),
                                state: "completed".to_string(),
                                exit_code: Some(0),
                            };
                            for hook in hooks
                                .iter()
                                .filter(|h| matches!(h.trigger, HookTrigger::OnComplete))
                            {
                                let _ = hook.execute(&ctx);
                            }
                        }
                    }
                    Err(e) => {
                        // Stop periodic hooks first
                        periodic_manager.stop_for_task(&id);

                        let duration = task_start_times.remove(&id)
                            .map(|start| start.elapsed())
                            .unwrap_or(Duration::from_secs(0));

                        let error_context = Self::capture_task_error_context(
                            &task_workdirs[&id],
                            &id,
                            &e,
                        );
                        tracing::error!("{}", error_context);

                        tracing::error!(
                            task_id = %id,
                            duration_secs = duration.as_secs(),
                            "Task failed after {}",
                            Self::format_duration(duration)
                        );

                        s.mark_failed(&id, e.to_string());

                        if let Some(hooks) = monitors.get(&id) {
                            let ctx = HookContext {
                                task_id: id.clone(),
                                workdir: task_workdirs[&id].clone(),
                                state: "failed".to_string(),
                                exit_code: None,
                            };
                            for hook in hooks
                                .iter()
                                .filter(|h| matches!(h.trigger, HookTrigger::OnFailure))
                            {
                                let _ = hook.execute(&ctx);
                            }
                        }
                    }
                }
                s.save(&self.state_path)?;
            }

            {
                let mut s = state.lock().unwrap();
                let mut changed = true;
                while changed {
                    changed = false;
                    let to_skip: Vec<String> = dag
                        .task_ids()
                        .filter(|id| matches!(s.tasks.get(*id), Some(TaskStatus::Pending)))
                        .filter(|id| {
                            self.tasks
                                .get(*id)
                                .map(|t| {
                                    t.dependencies.iter().any(|dep| {
                                        matches!(
                                            s.tasks.get(dep.as_str()),
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
                            s.mark_skipped_due_to_dep_failure(&id);
                        }
                    }
                }
                s.save(&self.state_path)?;
            }

            let statuses: HashMap<String, TaskStatus> = state.lock().unwrap().tasks.clone();
            let done_set: HashSet<String> = statuses
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
                if matches!(statuses.get(&id), Some(TaskStatus::Pending)) {
                    if let Some(f) = fns.get(&id).cloned() {
                        state.lock().unwrap().mark_running(&id);

                        task_start_times.insert(id.clone(), Instant::now());

                        tracing::info!(
                            task_id = %id,
                            workdir = %task_workdirs[&id].display(),
                            "Task started"
                        );

                        // Spawn periodic hooks and OnStart hooks
                        if let Some(hooks) = monitors.get(&id) {
                            let periodic_hooks: Vec<_> = hooks
                                .iter()
                                .filter(|h| matches!(h.trigger, HookTrigger::Periodic { .. }))
                                .cloned()
                                .collect();

                            let ctx = HookContext {
                                task_id: id.clone(),
                                workdir: task_workdirs[&id].clone(),
                                state: "running".to_string(),
                                exit_code: None,
                            };

                            if !periodic_hooks.is_empty() {
                                periodic_manager.spawn_for_task(id.clone(), periodic_hooks, ctx.clone());
                            }

                            for hook in hooks
                                .iter()
                                .filter(|h| matches!(h.trigger, HookTrigger::OnStart))
                            {
                                let _ = hook.execute(&ctx);
                            }
                        }

                        let handle = std::thread::spawn(move || f());
                        handles.insert(id, handle);
                    }
                }
            }

            let all_done = {
                let s = state.lock().unwrap();
                dag.task_ids().all(|id| {
                    matches!(
                        s.tasks.get(id),
                        Some(TaskStatus::Completed)
                            | Some(TaskStatus::Failed { .. })
                            | Some(TaskStatus::Skipped)
                            | Some(TaskStatus::SkippedDueToDependencyFailure)
                    )
                })
            };

            if all_done && handles.is_empty() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let total_duration = workflow_start.elapsed();
        let final_state = state.lock().unwrap();
        let succeeded = final_state.tasks.values()
            .filter(|status| matches!(status, TaskStatus::Completed))
            .count();
        let failed = final_state.tasks.values()
            .filter(|status| matches!(status, TaskStatus::Failed { .. }))
            .count();

        tracing::info!(
            workflow_name = %self.name,
            duration_secs = total_duration.as_secs(),
            succeeded = succeeded,
            failed = failed,
            "Workflow completed in {}",
            Self::format_duration(total_duration)
        );

        Ok(())
    }

    fn build_dag(&self) -> Result<Dag> {
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

    fn format_duration(d: Duration) -> String {
        let secs = d.as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, mins, secs)
        } else if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }

    fn capture_task_error_context(workdir: &Path, task_id: &str, error: &anyhow::Error) -> String {
        format!(
            "Task '{}' failed: {}\nWorkdir: {}\n",
            task_id, error, workdir.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[test]
    fn single_task_completes() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let flag = Arc::new(Mutex::new(false));
        let flag2 = flag.clone();
        let mut wf = Workflow::builder()
            .name("wf_single".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", move || {
            *flag2.lock().unwrap() = true;
            Ok::<(), anyhow::Error>(())
        }))?;
        wf.run()?;
        assert!(*flag.lock().unwrap());
        Ok(())
    }

    #[test]
    fn chain_respects_order() -> anyhow::Result<()> {
        let dir = tempfile::tempdir().unwrap();
        let log = Arc::new(Mutex::new(Vec::<String>::new()));
        let l1 = log.clone();
        let l2 = log.clone();
        let mut wf = Workflow::builder()
            .name("wf_chain".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", move || {
            l1.lock().unwrap().push("a".into());
            Ok::<(), anyhow::Error>(())
        }))?;
        wf.add_task(
            Task::new("b", move || {
                l2.lock().unwrap().push("b".into());
                Ok::<(), anyhow::Error>(())
            })
            .depends_on("a"),
        )?;
        wf.run()?;
        assert_eq!(*log.lock().unwrap(), vec!["a", "b"]);
        Ok(())
    }

    #[test]
    fn failed_task_skips_dependent() -> anyhow::Result<()> {
        let dir = tempdir().unwrap();
        let mut wf = Workflow::builder()
            .name("wf_skip".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Err(anyhow::anyhow!("boom"))))?;
        wf.add_task(Task::new("b", || Ok::<(), anyhow::Error>(())).depends_on("a"))?;
        wf.run()?;
        let state = WorkflowState::load(dir.path().join(".wf_skip.workflow.json")).unwrap();
        assert!(matches!(
            state.tasks["b"],
            TaskStatus::SkippedDueToDependencyFailure
        ));
        Ok(())
    }

    #[test]
    fn dry_run_returns_topo_order() -> anyhow::Result<()> {
        let dir = tempdir().unwrap();
        let mut wf = Workflow::builder()
            .name("wf_dry".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Ok::<(), anyhow::Error>(())))?;
        wf.add_task(Task::new("b", || Ok::<(), anyhow::Error>(())).depends_on("a"))?;
        let order = wf.dry_run()?;
        let pa = order.iter().position(|x| x == "a").unwrap();
        let pb = order.iter().position(|x| x == "b").unwrap();
        assert!(pa < pb);
        Ok(())
    }

    #[test]
    fn duplicate_task_id_errors() -> anyhow::Result<()> {
        let dir = tempdir().unwrap();
        let mut wf = Workflow::builder()
            .name("wf_dup".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Ok::<(), anyhow::Error>(())))?;
        assert!(wf
            .add_task(Task::new("a", || Ok::<(), anyhow::Error>(())))
            .is_err());
        Ok(())
    }

    #[test]
    fn valid_dependency_add() -> anyhow::Result<()> {
        let mut wf = Workflow::builder()
            .name("wf_dep".to_string())
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Ok::<(), anyhow::Error>(())))?;
        assert!(wf
            .add_task(Task::new("b", || Ok::<(), anyhow::Error>(())).depends_on("a"))
            .is_ok());
        Ok(())
    }

    #[test]
    fn builder_with_custom_max_parallel() {
        let wf = Workflow::builder()
            .name("test".to_string())
            .state_dir(PathBuf::from("/tmp"))
            .max_parallel(4)
            .build()
            .unwrap();
        assert_eq!(wf.max_parallel, 4);
    }

    #[test]
    fn builder_validation_zero_parallelism() {
        let result = Workflow::builder()
            .name("test".to_string())
            .state_dir(PathBuf::from("/tmp"))
            .max_parallel(0)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn resume_uses_builder() {
        let dir = tempdir().unwrap();
        let wf = Workflow::resume("test", dir.path()).unwrap();
        assert_eq!(wf.name, "test");
    }

    #[test]
    fn resume_loads_existing_state() {
        let dir = tempdir().unwrap();
        let mut wf = Workflow::builder()
            .name("wf_resume".to_string())
            .state_dir(dir.path().to_path_buf())
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Ok::<(), anyhow::Error>(())))
            .unwrap();
        wf.run().unwrap();

        let mut wf2 = Workflow::resume("wf_resume", dir.path()).unwrap();
        wf2.add_task(Task::new("a", || Err(anyhow::anyhow!("should not re-run"))))
            .unwrap();
        wf2.run().unwrap();
        let state = WorkflowState::load(dir.path().join(".wf_resume.workflow.json")).unwrap();
        assert!(matches!(state.tasks["a"], TaskStatus::Completed));
    }
}
