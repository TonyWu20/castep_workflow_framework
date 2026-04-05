//! CASTEP workflow adapter.
//!
//! Provides [`CastepFactory`] which builds executors for CASTEP tasks. The adapter handles:
//!
//! - Writing `.param` files from `task.inputs`
//! - Propagating `.check` checkpoint files from dependency tasks
//! - Delegating execution to local or Slurm executors
//!
//! # `.check` file propagation
//!
//! CASTEP property calculations (DOS, band structure, optical properties) require the binary
//! checkpoint (`.check` file) from a parent SCF or geometry optimization calculation. The
//! adapter automatically copies these files from dependency workdirs before job submission.
//!
//! This enables workflows like:
//!
//! ```toml
//! [[tasks]]
//! id = "scf_U{u}"
//! code = "castep"
//! [tasks.sweep]
//! params = [{ name = "u", values = [2, 3, 4] }]
//!
//! [[tasks]]
//! id = "dos_U{u}"
//! code = "castep"
//! attached_to = "scf_U{u}"  # Inherits sweep, gets .check file automatically
//! [tasks.inputs]
//! TASK = "spectral"
//! ```
//!
//! Each `dos_U{u}` task will have `scf_U{u}.check` copied into its workdir before execution.

use anyhow::Result;
use async_trait::async_trait;
use workflow_core::executor::{Executor, ExecutorFactory, JobHandle, JobStatus};
use workflow_core::schema::{ConcreteTask, ExecutorDef};
use workflow_core::executors::local::LocalExecutor;
use workflow_core::executors::slurm::{SlurmExecutor, ProcessRunner};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::PathBuf;

/// CASTEP executor that defers filesystem I/O to submit time.
///
/// Writes `.param` files and copies `.check` checkpoint files from dependency workdirs
/// before delegating to the underlying executor (local or Slurm).
struct CastepExecutor {
    /// Underlying executor (LocalExecutor or SlurmExecutor).
    inner: Box<dyn Executor + Sync>,
    /// Task ID, used as the CASTEP seed name.
    task_id: String,
    /// Working directory for this task.
    workdir: PathBuf,
    /// Path to the `.param` file to write.
    param_path: PathBuf,
    /// Content of the `.param` file.
    param_content: String,
    /// Working directories of dependency tasks, keyed by task ID.
    ///
    /// Used to locate `.check` files from upstream CASTEP calculations.
    dep_workdirs: HashMap<String, PathBuf>,
}

#[async_trait]
impl Executor for CastepExecutor {
    /// Submit the CASTEP job after preparing the working directory.
    ///
    /// Performs the following setup:
    /// 1. Creates the working directory
    /// 2. Writes the `.param` file
    /// 3. Copies `.check` checkpoint files from dependency workdirs (if present)
    /// 4. Delegates to the underlying executor
    ///
    /// # `.check` file propagation
    ///
    /// CASTEP property calculations (DOS, band structure, etc.) require the `.check` binary
    /// checkpoint from the parent SCF calculation. This method automatically copies
    /// `<dep_workdir>/<dep_id>.check` → `<workdir>/<task_id>.check` for each dependency.
    ///
    /// If a dependency's `.check` file doesn't exist, it's silently skipped (the dependency
    /// may not be a CASTEP task, or may not have completed successfully).
    async fn submit(&self) -> Result<JobHandle> {
        tokio::fs::create_dir_all(&self.workdir).await?;
        tokio::fs::write(&self.param_path, &self.param_content).await?;

        // Copy .check files from dependencies
        for (dep_id, dep_wd) in &self.dep_workdirs {
            let src = dep_wd.join(format!("{dep_id}.check"));
            if tokio::fs::try_exists(&src).await.unwrap_or(false) {
                let dst = self.workdir.join(format!("{}.check", self.task_id));
                tokio::fs::copy(&src, &dst).await?;
            }
        }

        self.inner.submit().await
    }

    async fn poll(&self, handle: &JobHandle) -> Result<JobStatus> {
        self.inner.poll(handle).await
    }

    async fn cancel(&self, handle: &JobHandle) -> Result<()> {
        self.inner.cancel(handle).await
    }
}

pub struct CastepFactory;

impl ExecutorFactory for CastepFactory {
    fn code_name(&self) -> &'static str { "castep" }

    fn build(&self, task: &ConcreteTask) -> Result<Box<dyn Executor>> {
        let workdir = PathBuf::from(&task.workdir);
        let mut param = String::new();
        for (k, v) in &task.inputs {
            let val = match v {
                toml::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            writeln!(param, "{} : {}", k, val)?;
        }
        let param_path = workdir.join(format!("{}.param", task.id));

        let inner: Box<dyn Executor + Sync> = match &task.executor_def {
            ExecutorDef::Local { .. } => {
                Box::new(LocalExecutor::new("castep", vec![task.id.clone()], &workdir))
            }
            ExecutorDef::Slurm { partition, ntasks, walltime } => {
                let jobscript = format!(
                    "#!/bin/bash\n#SBATCH --partition={partition}\n#SBATCH --ntasks={ntasks}\n#SBATCH --time={walltime}\ncastep {id}\n",
                    id = task.id
                );
                Box::new(SlurmExecutor::new(jobscript, &workdir, Box::new(ProcessRunner)))
            }
        };

        let dep_workdirs: HashMap<String, PathBuf> = task.dep_workdirs.iter()
            .map(|(id, wd)| (id.clone(), PathBuf::from(wd)))
            .collect();
        Ok(Box::new(CastepExecutor {
            inner,
            task_id: task.id.clone(),
            workdir,
            param_path,
            param_content: param,
            dep_workdirs,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_task(executor_def: ExecutorDef) -> ConcreteTask {
        ConcreteTask {
            id: "test_scf".into(),
            code: "castep".into(),
            executor: "local".into(),
            workdir: format!("/tmp/castep_test_{}", std::process::id()),
            depends_on: vec![],
            inputs: {
                let mut m = HashMap::new();
                m.insert("CUT_OFF_ENERGY".into(), toml::Value::Integer(400));
                m.insert("TASK".into(), toml::Value::String("SinglePoint".into()));
                m
            },
            executor_def,
            wall_time_secs: None,
            bindings: HashMap::new(),
            dep_workdirs: HashMap::new(),
        }
    }

    #[test]
    fn build_returns_executor_without_io() {
        let task = make_task(ExecutorDef::Local { parallelism: 1 });
        let factory = CastepFactory;
        let _executor = factory.build(&task).unwrap();
        // Verify workdir was not created yet (I/O deferred to submit)
        assert!(!std::path::Path::new(&task.workdir).exists());
    }
}
