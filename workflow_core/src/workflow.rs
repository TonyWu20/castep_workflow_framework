use crate::dag::Dag;
use crate::error::WorkflowError;
use crate::monitoring::{HookContext, HookTrigger, MonitoringHook};
use crate::state::{JsonStateStore, StateStore, StateStoreExt, TaskStatus};
use crate::task::Task;
use bon::bon;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

struct PeriodicHookHandle {
    thread: std::thread::JoinHandle<()>,
    stop_signal: Arc<AtomicBool>,
}

struct PeriodicHookManager {
    handles: HashMap<String, Vec<PeriodicHookHandle>>,
}

impl PeriodicHookManager {
    fn new() -> Self {
        Self {
            handles: HashMap::new(),
        }
    }

    fn spawn_for_task(&mut self, _task_id: String, _hooks: &[MonitoringHook], _ctx: HookContext) {
        // Stub implementation: early return
    }

    fn stop_for_task(&mut self, task_id: &str) {
        if let Some(handles) = self.handles.remove(task_id) {
            for handle in handles {
                handle.stop_signal.store(true, Ordering::Relaxed);
                let _ = handle.thread.join();
            }
        }
    }

    #[allow(dead_code)]
    fn indent_output(output: &str) -> String {
        if output.is_empty() {
            return "<no output>".to_string();
        }
        output
            .lines()
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
    ) -> Result<Self, WorkflowError> {
        let max_parallel = max_parallel.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        });

        if max_parallel == 0 {
            return Err(WorkflowError::InvalidConfig(
                "max_parallel must be at least 1".into(),
            ));
        }

        let state_path = state_dir.join(format!(".{}.workflow.json", name));
        Ok(Self {
            name,
            tasks: HashMap::new(),
            state_path,
            max_parallel,
        })
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

    /// Resume a workflow by name, loading prior state from `state_dir` when `run()` is called.
    pub fn resume(
        name: impl Into<String>,
        state_dir: impl Into<PathBuf>,
    ) -> Result<Self, WorkflowError> {
        Self::builder()
            .name(name.into())
            .state_dir(state_dir.into())
            .build()
    }

    pub fn run(&mut self) -> Result<(), WorkflowError> {
        let dag = self.build_dag()?;
        tracing::debug!("DAG execution order: {:?}", dag.topological_order());

        let mut state = if self.state_path.exists() {
            JsonStateStore::load(&self.state_path)?
        } else {
            JsonStateStore::new(&self.name, self.state_path.clone())
        };
        for id in dag.task_ids() {
            state.tasks_mut().entry(id.clone()).or_insert(TaskStatus::Pending);
        }
        let state = Arc::new(Mutex::new(state));

        tracing::debug!("Registered {} tasks", self.tasks.len());

        let fns: HashMap<String, Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync>> = self
            .tasks
            .iter()
            .map(|(id, t)| {
                tracing::debug!(
                    "Registered task '{}' with {} dependencies",
                    id,
                    t.dependencies.len()
                );
                (id.clone(), Arc::clone(&t.execute_fn))
            })
            .collect();

        let monitors: HashMap<String, Vec<crate::monitoring::MonitoringHook>> = self
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

                        let duration = task_start_times
                            .remove(&id)
                            .map(|start| start.elapsed())
                            .unwrap_or(Duration::from_secs(0));

                        s.mark_completed(&id);

                        tracing::info!(
                            task_id = %id,
                            duration_secs = duration.as_secs(),
                            "Task completed in {}",
                            Self::format_duration(duration)
                        );

                        if let Some(_hooks) = monitors.get(&id) {
                            tracing::debug!("OnComplete hooks for task: {}", id);
                        }
                    }
                    Err(e) => {
                        // Stop periodic hooks first
                        periodic_manager.stop_for_task(&id);

                        let duration = task_start_times
                            .remove(&id)
                            .map(|start| start.elapsed())
                            .unwrap_or(Duration::from_secs(0));

                        let error_context =
                            Self::capture_task_error_context(&task_workdirs[&id], &id, &e);
                        tracing::error!("{}", error_context);

                        tracing::error!(
                            task_id = %id,
                            duration_secs = duration.as_secs(),
                            "Task failed after {}",
                            Self::format_duration(duration)
                        );

                        s.mark_failed(&id, e.to_string());

                        if let Some(_hooks) = monitors.get(&id) {
                            tracing::debug!("OnFailure hooks for task: {}", id);
                        }
                    }
                }
                s.save()?;
            }

            {
                let mut s = state.lock().unwrap();
                let mut changed = true;
                while changed {
                    changed = false;
                    let to_skip: Vec<String> = dag
                        .task_ids()
                        .filter(|id| matches!(s.get_status(*id), Some(TaskStatus::Pending)))
                        .filter(|id| {
                            self.tasks
                                .get(*id)
                                .map(|t| {
                                    t.dependencies.iter().any(|dep| {
                                        matches!(
                                            s.get_status(dep.as_str()),
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
                s.save()?;
            }

            let statuses: HashMap<String, TaskStatus> = state.lock().unwrap().all_task_statuses();
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

                            let _ctx = HookContext {
                                task_id: id.clone(),
                                workdir: task_workdirs[&id].clone(),
                                state: "running".to_string(),
                                exit_code: None,
                            };

                            if !periodic_hooks.is_empty() {
                                tracing::debug!("Periodic hooks for task: {}", id);
                            }

                            tracing::debug!("OnStart hooks for task: {}", id);
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
                        s.get_status(id),
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
        let succeeded = final_state
            .all_task_statuses()
            .values()
            .filter(|status| matches!(status, TaskStatus::Completed))
            .count();
        let failed = final_state
            .all_task_statuses()
            .values()
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
            task_id,
            error,
            workdir.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[test]
    fn single_task_completes() -> Result<(), Box<dyn std::error::Error>> {
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
            Ok::<_, anyhow::Error>(())
        }))?;
        wf.run()?;
        assert!(*flag.lock().unwrap());
        Ok(())
    }

    #[test]
    fn chain_respects_order() -> Result<(), Box<dyn std::error::Error>> {
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
            Ok::<_, anyhow::Error>(())
        }))?;
        wf.add_task(
            Task::new("b", move || {
                l2.lock().unwrap().push("b".into());
                Ok::<_, anyhow::Error>(())
            })
            .depends_on("a"),
        )?;
        wf.run()?;
        assert_eq!(*log.lock().unwrap(), vec!["a", "b"]);
        Ok(())
    }

    #[test]
    fn failed_task_skips_dependent() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let mut wf = Workflow::builder()
            .name("wf_skip".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Err(anyhow::anyhow!("boom"))))?;
        wf.add_task(Task::new("b", || Ok::<_, anyhow::Error>(())).depends_on("a"))?;
        wf.run()?;
        let state = JsonStateStore::load(dir.path().join(".wf_skip.workflow.json")).unwrap();
        assert!(matches!(
            state.get_status("b"),
            Some(TaskStatus::SkippedDueToDependencyFailure)
        ));
        Ok(())
    }

    #[test]
    fn dry_run_returns_topo_order() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let mut wf = Workflow::builder()
            .name("wf_dry".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Ok::<_, anyhow::Error>(())))?;
        wf.add_task(Task::new("b", || Ok::<_, anyhow::Error>(())).depends_on("a"))?;
        let order = wf.dry_run()?;
        let pa = order.iter().position(|x| x == "a").unwrap();
        let pb = order.iter().position(|x| x == "b").unwrap();
        assert!(pa < pb);
        Ok(())
    }

    #[test]
    fn duplicate_task_id_errors() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir().unwrap();
        let mut wf = Workflow::builder()
            .name("wf_dup".to_string())
            .state_dir(dir.path().to_path_buf())
            .max_parallel(4)
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Ok::<_, anyhow::Error>(())))?;
        assert_eq!(
            wf.add_task(Task::new("a", || Ok::<_, anyhow::Error>(()))),
            Err(WorkflowError::DuplicateTaskId("a".to_string()))
        );
        Ok(())
    }

    #[test]
    fn valid_dependency_add() -> Result<(), Box<dyn std::error::Error>> {
        let mut wf = Workflow::builder()
            .name("wf_dep".to_string())
            .build()
            .unwrap();
        wf.add_task(Task::new("a", || Ok::<_, anyhow::Error>(())))?;
        assert!(wf
            .add_task(Task::new("b", || Ok::<_, anyhow::Error>(())).depends_on("a"))
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
        let state = JsonStateStore::load(dir.path().join(".wf_resume.workflow.json")).unwrap();
        assert!(state.get_status("a") == Some(TaskStatus::Completed));
    }
}
