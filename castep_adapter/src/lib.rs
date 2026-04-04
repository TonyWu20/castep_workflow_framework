use anyhow::Result;
use async_trait::async_trait;
use workflow_core::executor::{Executor, ExecutorFactory, JobHandle, JobStatus};
use workflow_core::schema::{ConcreteTask, ExecutorDef};
use workflow_core::executors::local::LocalExecutor;
use workflow_core::executors::slurm::{SlurmExecutor, ProcessRunner};
use std::fmt::Write as _;
use std::path::PathBuf;

/// Wraps an executor to defer filesystem I/O to submit time.
struct CastepExecutor {
    inner: Box<dyn Executor + Sync>,
    workdir: PathBuf,
    param_path: PathBuf,
    param_content: String,
}

#[async_trait]
impl Executor for CastepExecutor {
    async fn submit(&self) -> Result<JobHandle> {
        tokio::fs::create_dir_all(&self.workdir).await?;
        tokio::fs::write(&self.param_path, &self.param_content).await?;
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

        Ok(Box::new(CastepExecutor {
            inner,
            workdir,
            param_path,
            param_content: param,
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
