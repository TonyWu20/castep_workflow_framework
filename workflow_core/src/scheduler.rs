//! Scheduler skeleton — drives the pipeline execution loop.

use anyhow::Result;
use tokio_util::sync::CancellationToken;
use crate::executor::{ExecutorRegistry, JobStatus};
use crate::pipeline::Pipeline;
use crate::state::{StateDb, TaskState, TaskRecord};
use std::collections::HashMap;

/// Drives execution of a [`Pipeline`], persisting state to [`StateDb`].
///
/// The scheduler polls all active jobs on a timer, promotes tasks whose
/// dependencies have completed, and skips tasks whose upstream paths have
/// all failed. On resume, tasks already marked [`crate::state::TaskState::Completed`]
/// in the state DB are skipped automatically.
pub struct Scheduler {
    pub pipeline: Pipeline,
    pub registry: ExecutorRegistry,
    pub state_db: StateDb,
    token: Option<CancellationToken>,
}

impl Scheduler {
    pub fn new(pipeline: Pipeline, registry: ExecutorRegistry, state_db: StateDb) -> Self {
        Self { pipeline, registry, state_db, token: None }
    }

    /// Attach a cancellation token; on cancel, running jobs are killed and marked `Failed(-1)`.
    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.token = Some(token);
        self
    }

    /// Run the pipeline to completion (or until all tasks are settled).
    ///
    /// Independent branches continue running when one branch fails.
    /// State is persisted to the DB after every poll cycle for resume support.
    pub async fn run(&mut self) -> Result<()> {
        loop {
            // 1. Load all task states from DB
            let records = self.state_db.load().await?;
            let mut state_map: HashMap<String, TaskRecord> = records.iter()
                .map(|r| (r.id.clone(), r.clone()))
                .collect();

            // Initialize missing tasks as Pending
            for task in self.pipeline.graph.node_weights() {
                state_map.entry(task.id.clone())
                    .or_insert_with(|| TaskRecord {
                        id: task.id.clone(),
                        state: TaskState::Pending,
                        handle: None,
                    });
            }

            // 2. Promote Pending → Ready if all deps completed
            let ready_ids: Vec<String> = self.pipeline.graph.node_weights()
                .filter_map(|task| {
                    if state_map.get(&task.id).map(|r| r.state == TaskState::Pending).unwrap_or(false) {
                        let all_deps_done = task.depends_on.iter().all(|dep_id| {
                            matches!(state_map.get(dep_id).map(|r| &r.state),
                                Some(TaskState::Completed) | Some(TaskState::Skipped))
                        });
                        if all_deps_done { Some(task.id.clone()) } else { None }
                    } else {
                        None
                    }
                })
                .collect();
            for id in ready_ids {
                if let Some(record) = state_map.get_mut(&id) {
                    record.state = TaskState::Ready;
                }
            }

            // 3. Submit Ready tasks (respecting parallelism cap)
            let mut submitted_count: HashMap<String, usize> = HashMap::new();
            let mut submit_ids = Vec::new();
            for task in self.pipeline.graph.node_weights() {
                if state_map.get(&task.id).map(|r| r.state == TaskState::Ready).unwrap_or(false) {
                    let can_submit = match &task.executor_def {
                        crate::schema::ExecutorDef::Local { parallelism } => {
                            let count = submitted_count.entry(task.executor.clone()).or_insert(0);
                            *count < *parallelism
                        }
                        crate::schema::ExecutorDef::Slurm { .. } => true,
                    };
                    if can_submit {
                        if let Ok(executor) = self.registry.build(task) {
                            if let Ok(handle) = executor.submit().await {
                                submit_ids.push((task.id.clone(), handle));
                                *submitted_count.entry(task.executor.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
            for (id, handle) in submit_ids {
                if let Some(record) = state_map.get_mut(&id) {
                    record.state = TaskState::Submitted;
                    record.handle = Some(handle);
                }
            }

            // 4. Poll all Submitted/Running handles → update state
            let mut poll_updates = Vec::new();
            for (id, record) in state_map.iter() {
                match &record.state {
                    TaskState::Submitted | TaskState::Running => {
                        if let Some(handle) = &record.handle {
                            if let Some(task) = self.pipeline.graph.node_weights()
                                .find(|t| t.id == *id) {
                                if let Ok(executor) = self.registry.build(task) {
                                    if let Ok(status) = executor.poll(handle).await {
                                        let new_state = match status {
                                            JobStatus::Running => TaskState::Running,
                                            JobStatus::Completed => TaskState::Completed,
                                            JobStatus::Failed(code) => TaskState::Failed(code),
                                        };
                                        poll_updates.push((id.clone(), new_state));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            for (id, new_state) in poll_updates {
                if let Some(record) = state_map.get_mut(&id) {
                    record.state = new_state;
                }
            }

            // 5. Mark tasks whose dependency failed as Skipped (transitively)
            let mut changed = true;
            while changed {
                changed = false;
                let skip_ids: Vec<String> = self.pipeline.graph.node_weights()
                    .filter_map(|task| {
                        if let Some(record) = state_map.get(&task.id) {
                            if record.state == TaskState::Pending || record.state == TaskState::Ready {
                                let has_failed_dep = task.depends_on.iter().any(|dep_id| {
                                    matches!(state_map.get(dep_id).map(|r| &r.state),
                                        Some(TaskState::Failed(_)))
                                });
                                if has_failed_dep {
                                    return Some(task.id.clone());
                                }
                            }
                        }
                        None
                    })
                    .collect();
                if !skip_ids.is_empty() {
                    changed = true;
                    for id in skip_ids {
                        if let Some(record) = state_map.get_mut(&id) {
                            record.state = TaskState::Skipped;
                        }
                    }
                }
            }

            // 6. Persist all state changes
            for record in state_map.values() {
                self.state_db.upsert(record).await?;
            }

            // 7. Check if all tasks are terminal
            let all_terminal = state_map.values().all(|r| {
                matches!(r.state, TaskState::Completed | TaskState::Failed(_) | TaskState::Skipped)
            });
            if all_terminal {
                return Ok(());
            }

            // 8. Sleep 100ms or cancel
            let cancelled = tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => false,
                _ = async {
                    if let Some(t) = &self.token { t.cancelled().await }
                    else { std::future::pending::<()>().await }
                } => true,
            };
            if cancelled {
                let running: Vec<(String, crate::executor::JobHandle)> = state_map.values()
                    .filter(|r| matches!(r.state, TaskState::Submitted | TaskState::Running))
                    .filter_map(|r| r.handle.clone().map(|h| (r.id.clone(), h)))
                    .collect();
                for (id, handle) in running {
                    let task = self.pipeline.graph.node_weights().find(|t| t.id == id).unwrap();
                    if let Ok(executor) = self.registry.build(task) {
                        let _ = executor.cancel(&handle).await;
                    }
                    state_map.get_mut(&id).unwrap().state = TaskState::Failed(-1);
                }
                for rec in state_map.values() {
                    self.state_db.upsert(rec).await?;
                }
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ConcreteTask, ExecutorDef};
    use crate::executor::{Executor, JobHandle, JobStatus};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    struct MockExecutor {
        _submitted: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl Executor for MockExecutor {
        async fn submit(&self) -> Result<JobHandle> {
            Ok(JobHandle { raw: "mock".into() })
        }
        async fn poll(&self, _h: &JobHandle) -> Result<JobStatus> {
            Ok(JobStatus::Completed)
        }
        async fn cancel(&self, _h: &JobHandle) -> Result<()> {
            Ok(())
        }
    }

    fn make_task(id: &str, deps: Vec<&str>) -> ConcreteTask {
        ConcreteTask {
            id: id.into(),
            code: "test".into(),
            executor: "local".into(),
            workdir: "/tmp".into(),
            depends_on: deps.into_iter().map(String::from).collect(),
            inputs: HashMap::new(),
            executor_def: ExecutorDef::Local { parallelism: 4 },
        }
    }

    #[tokio::test]
    async fn cancellation_marks_running_tasks_failed() {
        use tokio_util::sync::CancellationToken;

        struct SlowExecutor(Arc<Mutex<bool>>);
        #[async_trait]
        impl Executor for SlowExecutor {
            async fn submit(&self) -> Result<JobHandle> { Ok(JobHandle { raw: "x".into() }) }
            async fn poll(&self, _h: &JobHandle) -> Result<JobStatus> { Ok(JobStatus::Running) }
            async fn cancel(&self, _h: &JobHandle) -> Result<()> {
                *self.0.lock().unwrap() = true;
                Ok(())
            }
        }
        struct SlowFactory(Arc<Mutex<bool>>);
        impl crate::executor::ExecutorFactory for SlowFactory {
            fn code_name(&self) -> &'static str { "test" }
            fn build(&self, _t: &ConcreteTask) -> Result<Box<dyn Executor>> {
                Ok(Box::new(SlowExecutor(self.0.clone())))
            }
        }

        let cancelled = Arc::new(Mutex::new(false));
        let pipeline = Pipeline::from_tasks(vec![make_task("a", vec![])]).unwrap();
        let db = StateDb::open(std::path::Path::new(":memory:")).await.unwrap();
        let mut registry = ExecutorRegistry::default();
        registry.register(SlowFactory(cancelled.clone()));

        let token = CancellationToken::new();
        let token_clone = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            token_clone.cancel();
        });

        let mut sched = Scheduler::new(pipeline, registry, db).with_cancellation(token);
        sched.run().await.unwrap();

        assert!(*cancelled.lock().unwrap());
        let records = sched.state_db.load().await.unwrap();
        assert!(matches!(
            records.iter().find(|r| r.id == "a").unwrap().state,
            TaskState::Failed(-1)
        ));
    }

    #[tokio::test]
    async fn three_task_chain_completes() {
        let tasks = vec![
            make_task("a", vec![]),
            make_task("b", vec!["a"]),
            make_task("c", vec!["b"]),
        ];
        let pipeline = Pipeline::from_tasks(tasks).unwrap();
        let db = StateDb::open(std::path::Path::new(":memory:")).await.unwrap();

        let mut registry = ExecutorRegistry::default();
        struct TestFactory;
        impl crate::executor::ExecutorFactory for TestFactory {
            fn code_name(&self) -> &'static str { "test" }
            fn build(&self, _task: &ConcreteTask) -> Result<Box<dyn Executor>> {
                Ok(Box::new(MockExecutor {
                    _submitted: Arc::new(Mutex::new(vec![])),
                }))
            }
        }
        registry.register(TestFactory);

        let mut sched = Scheduler::new(pipeline, registry, db);
        sched.run().await.unwrap();
    }
}
