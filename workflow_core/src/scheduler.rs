//! Scheduler skeleton — drives the pipeline execution loop.

use anyhow::Result;
use tokio_util::sync::CancellationToken;
use crate::executor::{Executor, ExecutorRegistry, JobStatus};
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
    state_map: HashMap<String, TaskRecord>,
    live_executors: HashMap<String, Box<dyn Executor>>,
    token: Option<CancellationToken>,
}

impl Scheduler {
    pub fn new(pipeline: Pipeline, registry: ExecutorRegistry, state_db: StateDb) -> Self {
        Self {
            pipeline,
            registry,
            state_db,
            state_map: HashMap::new(),
            live_executors: HashMap::new(),
            token: None,
        }
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
        // Load state once at startup
        let records = self.state_db.load().await?;
        self.state_map = records.iter()
            .map(|r| (r.id.clone(), r.clone()))
            .collect();

        // Initialize missing tasks as Pending
        for task in self.pipeline.tasks() {
            self.state_map.entry(task.id.clone())
                .or_insert_with(|| TaskRecord {
                    id: task.id.clone(),
                    state: TaskState::Pending,
                    handle: None,
                    submitted_at: None,
                });
        }

        // Fix Bug 2: Re-hydrate live_executors for tasks already Submitted/Running in DB.
        // On a fresh start live_executors is empty; without this, those tasks are polled
        // forever with no executor and the scheduler never terminates.
        for task in self.pipeline.tasks() {
            if let Some(record) = self.state_map.get(&task.id) {
                if matches!(record.state, TaskState::Submitted | TaskState::Running) {
                    if let Ok(executor) = self.registry.build(task) {
                        self.live_executors.insert(task.id.clone(), executor);
                    }
                }
            }
        }

        loop {
            // 1. Promote Pending → Ready if all deps completed
            let ready_ids: Vec<String> = self.pipeline.tasks()
                .filter_map(|task| {
                    if self.state_map.get(&task.id).map(|r| r.state == TaskState::Pending).unwrap_or(false) {
                        let all_deps_done = task.depends_on.iter().all(|dep_id| {
                            matches!(self.state_map.get(dep_id).map(|r| &r.state),
                                Some(TaskState::Completed) | Some(TaskState::Skipped) | Some(TaskState::TimedOut))
                        });
                        if all_deps_done { Some(task.id.clone()) } else { None }
                    } else {
                        None
                    }
                })
                .collect();
            for id in ready_ids {
                if let Some(record) = self.state_map.get_mut(&id) {
                    record.state = TaskState::Ready;
                }
            }

            // 2. Submit Ready tasks (respecting parallelism cap)
            // Fix Bug 1: seed with already-active tasks so the cap is respected across cycles.
            let mut submitted_count: HashMap<String, usize> = HashMap::new();
            for task in self.pipeline.tasks() {
                if matches!(
                    self.state_map.get(&task.id).map(|r| &r.state),
                    Some(TaskState::Submitted) | Some(TaskState::Running)
                ) {
                    *submitted_count.entry(task.executor.clone()).or_insert(0) += 1;
                }
            }
            let mut submit_ids = Vec::new();
            for task in self.pipeline.tasks() {
                if self.state_map.get(&task.id).map(|r| r.state == TaskState::Ready).unwrap_or(false) {
                    let can_submit = match &task.executor_def {
                        crate::schema::ExecutorDef::Local { parallelism } => {
                            let count = submitted_count.entry(task.executor.clone()).or_insert(0);
                            *count < *parallelism
                        }
                        crate::schema::ExecutorDef::Slurm { .. } => true,
                    };
                    if can_submit {
                        if let Ok(executor) = self.registry.build(task) {
                            match executor.submit().await {
                                Ok(handle) => {
                                    self.live_executors.insert(task.id.clone(), executor);
                                    submit_ids.push((task.id.clone(), handle));
                                    *submitted_count.entry(task.executor.clone()).or_insert(0) += 1;
                                }
                                Err(_) => {
                                    if let Some(record) = self.state_map.get_mut(&task.id) {
                                        record.state = TaskState::Failed(-1);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for (id, handle) in submit_ids {
                if let Some(record) = self.state_map.get_mut(&id) {
                    record.state = TaskState::Submitted;
                    record.handle = Some(handle);
                    record.submitted_at = Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs()
                    );
                }
            }

            // 3. Poll all Submitted/Running handles → update state
            let mut poll_updates = Vec::new();
            for (id, record) in self.state_map.iter() {
                match &record.state {
                    TaskState::Submitted | TaskState::Running => {
                        if let Some(handle) = &record.handle {
                            if let Some(executor) = self.live_executors.get(id) {
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
                    _ => {}
                }
            }
            for (id, new_state) in poll_updates {
                if let Some(record) = self.state_map.get_mut(&id) {
                    record.state = new_state;
                }
            }

            // 3b: Cancel tasks that have exceeded their wall-time limit.
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let timeout_ids: Vec<String> = self.pipeline.tasks()
                .filter_map(|task| {
                    let limit = task.wall_time_secs?;
                    let record = self.state_map.get(&task.id)?;
                    if !matches!(record.state, TaskState::Submitted | TaskState::Running) {
                        return None;
                    }
                    let start = record.submitted_at?;
                    if now_secs.saturating_sub(start) >= limit { Some(task.id.clone()) } else { None }
                })
                .collect();
            for id in timeout_ids {
                if let Some(executor) = self.live_executors.remove(&id) {
                    if let Some(handle) = self.state_map.get(&id).and_then(|r| r.handle.as_ref()) {
                        let _ = executor.cancel(handle).await;
                    }
                }
                if let Some(record) = self.state_map.get_mut(&id) {
                    record.state = TaskState::TimedOut;
                }
            }

            // 4. Mark tasks whose dependency failed as Skipped (transitively)
            let mut changed = true;
            while changed {
                changed = false;
                let skip_ids: Vec<String> = self.pipeline.tasks()
                    .filter_map(|task| {
                        if let Some(record) = self.state_map.get(&task.id) {
                            if record.state == TaskState::Pending || record.state == TaskState::Ready {
                                let has_failed_dep = task.depends_on.iter().any(|dep_id| {
                                    matches!(self.state_map.get(dep_id).map(|r| &r.state),
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
                        if let Some(record) = self.state_map.get_mut(&id) {
                            record.state = TaskState::Skipped;
                        }
                    }
                }
            }

            // 5. Persist only changed records
            for record in self.state_map.values() {
                self.state_db.upsert(record).await?;
            }

            // 6. Check if all tasks are terminal
            let all_terminal = self.state_map.values().all(|r| {
                matches!(r.state, TaskState::Completed | TaskState::Failed(_) | TaskState::Skipped | TaskState::TimedOut)
            });
            if all_terminal {
                return Ok(());
            }

            // 7. Sleep 100ms or cancel
            let cancelled = tokio::select! {
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => false,
                _ = async {
                    if let Some(t) = &self.token { t.cancelled().await }
                    else { std::future::pending::<()>().await }
                } => true,
            };
            if cancelled {
                let running: Vec<(String, crate::executor::JobHandle)> = self.state_map.values()
                    .filter(|r| matches!(r.state, TaskState::Submitted | TaskState::Running))
                    .filter_map(|r| r.handle.clone().map(|h| (r.id.clone(), h)))
                    .collect();
                for (id, handle) in running {
                    if let Some(executor) = self.live_executors.get(&id) {
                        let _ = executor.cancel(&handle).await;
                    }
                    self.state_map.get_mut(&id).unwrap().state = TaskState::Failed(-1);
                }
                for rec in self.state_map.values() {
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

    // Completes immediately.
    struct MockExecutor;
    #[async_trait]
    impl Executor for MockExecutor {
        async fn submit(&self) -> Result<JobHandle> { Ok(JobHandle { raw: "mock".into() }) }
        async fn poll(&self, _h: &JobHandle) -> Result<JobStatus> { Ok(JobStatus::Completed) }
        async fn cancel(&self, _h: &JobHandle) -> Result<()> { Ok(()) }
    }

    fn make_task(id: &str, deps: Vec<&str>, parallelism: usize) -> ConcreteTask {
        ConcreteTask {
            id: id.into(),
            code: "test".into(),
            executor: "local".into(),
            workdir: "/tmp".into(),
            depends_on: deps.into_iter().map(String::from).collect(),
            inputs: HashMap::new(),
            executor_def: ExecutorDef::Local { parallelism },
            wall_time_secs: None,
        }
    }

    struct MockFactory;
    impl crate::executor::ExecutorFactory for MockFactory {
        fn code_name(&self) -> &'static str { "test" }
        fn build(&self, _t: &ConcreteTask) -> Result<Box<dyn Executor>> {
            Ok(Box::new(MockExecutor))
        }
    }

    fn mock_registry() -> ExecutorRegistry {
        let mut r = ExecutorRegistry::default();
        r.register(MockFactory);
        r
    }

    #[tokio::test]
    async fn three_task_chain_completes() {
        let tasks = vec![
            make_task("a", vec![], 4),
            make_task("b", vec!["a"], 4),
            make_task("c", vec!["b"], 4),
        ];
        let pipeline = Pipeline::from_tasks(tasks).unwrap();
        let db = StateDb::open(std::path::Path::new(":memory:")).await.unwrap();
        let mut sched = Scheduler::new(pipeline, mock_registry(), db);
        sched.run().await.unwrap();
        let records = sched.state_db.load().await.unwrap();
        assert!(records.iter().all(|r| r.state == TaskState::Completed));
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
        let pipeline = Pipeline::from_tasks(vec![make_task("a", vec![], 4)]).unwrap();
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

    /// Bug 1 regression: parallelism cap must count already-active tasks, not just
    /// tasks submitted this cycle. With parallelism=2 and 4 ready tasks, at most 2
    /// should ever be Submitted/Running simultaneously.
    #[tokio::test]
    async fn parallelism_cap_is_respected() {
        // Latch executors: stay Running until we release them.
        let latches: Vec<Arc<Mutex<bool>>> = (0..4).map(|_| Arc::new(Mutex::new(false))).collect();
        let peak_concurrent = Arc::new(Mutex::new(0usize));
        let current_running = Arc::new(Mutex::new(0usize));

        struct CountingExecutor {
            done: Arc<Mutex<bool>>,
            peak: Arc<Mutex<usize>>,
            current: Arc<Mutex<usize>>,
        }
        #[async_trait]
        impl Executor for CountingExecutor {
            async fn submit(&self) -> Result<JobHandle> {
                let mut cur = self.current.lock().unwrap();
                *cur += 1;
                let mut peak = self.peak.lock().unwrap();
                if *cur > *peak { *peak = *cur; }
                Ok(JobHandle { raw: "c".into() })
            }
            async fn poll(&self, _h: &JobHandle) -> Result<JobStatus> {
                if *self.done.lock().unwrap() {
                    *self.current.lock().unwrap() -= 1;
                    Ok(JobStatus::Completed)
                } else {
                    Ok(JobStatus::Running)
                }
            }
            async fn cancel(&self, _h: &JobHandle) -> Result<()> { Ok(()) }
        }

        let peak_c = peak_concurrent.clone();
        let cur_c = current_running.clone();
        let latches_c = latches.clone();
        struct CountingFactory {
            latches: Vec<Arc<Mutex<bool>>>,
            idx: Mutex<usize>,
            peak: Arc<Mutex<usize>>,
            current: Arc<Mutex<usize>>,
        }
        impl crate::executor::ExecutorFactory for CountingFactory {
            fn code_name(&self) -> &'static str { "test" }
            fn build(&self, _t: &ConcreteTask) -> Result<Box<dyn Executor>> {
                let mut i = self.idx.lock().unwrap();
                let done = self.latches[*i].clone();
                *i += 1;
                Ok(Box::new(CountingExecutor {
                    done,
                    peak: self.peak.clone(),
                    current: self.current.clone(),
                }))
            }
        }

        // 4 independent tasks, parallelism = 2
        let tasks: Vec<_> = (0..4).map(|i| make_task(&format!("t{i}"), vec![], 2)).collect();
        let pipeline = Pipeline::from_tasks(tasks).unwrap();
        let db = StateDb::open(std::path::Path::new(":memory:")).await.unwrap();
        let mut registry = ExecutorRegistry::default();
        registry.register(CountingFactory {
            latches: latches_c,
            idx: Mutex::new(0),
            peak: peak_c,
            current: cur_c,
        });

        // Release all latches after a short delay so the scheduler can observe concurrency.
        let latches_release = latches.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            for l in &latches_release { *l.lock().unwrap() = true; }
        });

        let mut sched = Scheduler::new(pipeline, registry, db);
        sched.run().await.unwrap();

        assert!(
            *peak_concurrent.lock().unwrap() <= 2,
            "peak concurrent was {}, expected ≤ 2",
            *peak_concurrent.lock().unwrap()
        );
    }

    /// Bug 2 regression: after a restart, tasks that were Submitted/Running in the DB
    /// must be re-hydrated and polled to completion — not silently skipped forever.
    #[tokio::test]
    async fn resume_rehydrates_submitted_tasks() {
        // Use a temp file so the DB survives across two Scheduler instances.
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let db_path = tmp.path().to_path_buf();

        // First run: submit task "a", then drop the scheduler mid-flight (simulate crash).
        // We do this by pre-seeding the DB directly with a Submitted record.
        {
            let db = StateDb::open(&db_path).await.unwrap();
            db.upsert(&TaskRecord {
                id: "a".into(),
                state: TaskState::Submitted,
                handle: Some(crate::executor::JobHandle { raw: "mock".into() }),
                submitted_at: None,
            }).await.unwrap();
        }

        // Second run: new scheduler, same DB. Should pick up "a" as Submitted,
        // re-hydrate an executor, poll it to Completed, and terminate.
        let pipeline = Pipeline::from_tasks(vec![make_task("a", vec![], 4)]).unwrap();
        let db = StateDb::open(&db_path).await.unwrap();
        let mut sched = Scheduler::new(pipeline, mock_registry(), db);
        sched.run().await.unwrap();

        let records = sched.state_db.load().await.unwrap();
        assert_eq!(
            records.iter().find(|r| r.id == "a").unwrap().state,
            TaskState::Completed
        );
    }

    // Always returns Running; used for timeout tests.
    struct SlowExecutor;
    #[async_trait]
    impl Executor for SlowExecutor {
        async fn submit(&self) -> Result<JobHandle> { Ok(JobHandle { raw: "slow".into() }) }
        async fn poll(&self, _h: &JobHandle) -> Result<JobStatus> { Ok(JobStatus::Running) }
        async fn cancel(&self, _h: &JobHandle) -> Result<()> { Ok(()) }
    }

    struct SlowFactory;
    impl crate::executor::ExecutorFactory for SlowFactory {
        fn code_name(&self) -> &'static str { "slow" }
        fn build(&self, _t: &ConcreteTask) -> Result<Box<dyn Executor>> {
            Ok(Box::new(SlowExecutor))
        }
    }

    struct FastFactory;
    impl crate::executor::ExecutorFactory for FastFactory {
        fn code_name(&self) -> &'static str { "fast" }
        fn build(&self, _t: &ConcreteTask) -> Result<Box<dyn Executor>> {
            Ok(Box::new(MockExecutor))
        }
    }

    #[tokio::test]
    async fn task_with_zero_wall_time_times_out() {
        let mut task = make_task("a", vec![], 4);
        task.code = "slow".into();
        task.wall_time_secs = Some(0);

        let pipeline = Pipeline::from_tasks(vec![task]).unwrap();
        let db = StateDb::open(std::path::Path::new(":memory:")).await.unwrap();
        let mut registry = ExecutorRegistry::default();
        registry.register(SlowFactory);

        let mut sched = Scheduler::new(pipeline, registry, db);
        sched.run().await.unwrap();

        let records = sched.state_db.load().await.unwrap();
        assert_eq!(
            records.iter().find(|r| r.id == "a").unwrap().state,
            TaskState::TimedOut
        );
    }

    #[tokio::test]
    async fn child_of_timed_out_parent_still_runs() {
        let mut parent = make_task("parent", vec![], 4);
        parent.code = "slow".into();
        parent.wall_time_secs = Some(0);

        let mut child = make_task("child", vec!["parent"], 4);
        child.code = "fast".into();

        let pipeline = Pipeline::from_tasks(vec![parent, child]).unwrap();
        let db = StateDb::open(std::path::Path::new(":memory:")).await.unwrap();
        let mut registry = ExecutorRegistry::default();
        registry.register(SlowFactory);
        registry.register(FastFactory);

        let mut sched = Scheduler::new(pipeline, registry, db);
        sched.run().await.unwrap();

        let records = sched.state_db.load().await.unwrap();
        assert_eq!(
            records.iter().find(|r| r.id == "parent").unwrap().state,
            TaskState::TimedOut
        );
        assert_eq!(
            records.iter().find(|r| r.id == "child").unwrap().state,
            TaskState::Completed
        );
    }

    #[tokio::test]
    async fn task_within_wall_time_limit_completes() {
        let mut task = make_task("a", vec![], 4);
        task.code = "fast".into();
        task.wall_time_secs = Some(3600);

        let pipeline = Pipeline::from_tasks(vec![task]).unwrap();
        let db = StateDb::open(std::path::Path::new(":memory:")).await.unwrap();
        let mut registry = ExecutorRegistry::default();
        registry.register(FastFactory);

        let mut sched = Scheduler::new(pipeline, registry, db);
        sched.run().await.unwrap();

        let records = sched.state_db.load().await.unwrap();
        assert_eq!(
            records.iter().find(|r| r.id == "a").unwrap().state,
            TaskState::Completed
        );
    }
}
