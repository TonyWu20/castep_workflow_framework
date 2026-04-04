use anyhow::Result;
use workflow_core::executor::{Executor, ExecutorFactory};
use workflow_core::schema::{ConcreteTask, ExecutorDef};
use workflow_core::executors::local::LocalExecutor;
use workflow_core::executors::slurm::{SlurmExecutor, ProcessRunner};
use std::fmt::Write as _;

pub struct CastepFactory;

impl ExecutorFactory for CastepFactory {
    fn code_name(&self) -> &'static str { "castep" }

    fn build(&self, task: &ConcreteTask) -> Result<Box<dyn Executor>> {
        let workdir = std::path::Path::new(&task.workdir);
        std::fs::create_dir_all(workdir)?;
        let mut param = String::new();
        for (k, v) in &task.inputs {
            let val = match v {
                toml::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            writeln!(param, "{} : {}", k, val)?;
        }
        std::fs::write(workdir.join(format!("{}.param", task.id)), &param)?;

        match &task.executor_def {
            ExecutorDef::Local { .. } => {
                Ok(Box::new(LocalExecutor::new("castep", vec![task.id.clone()], workdir)))
            }
            ExecutorDef::Slurm { partition, ntasks, walltime } => {
                let jobscript = format!(
                    "#!/bin/bash\n#SBATCH --partition={partition}\n#SBATCH --ntasks={ntasks}\n#SBATCH --time={walltime}\ncastep {id}\n",
                    id = task.id
                );
                Ok(Box::new(SlurmExecutor::new(jobscript, workdir, Box::new(ProcessRunner))))
            }
        }
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
    fn build_writes_param_file_and_returns_local_executor() {
        let task = make_task(ExecutorDef::Local { parallelism: 1 });
        let factory = CastepFactory;
        let _executor = factory.build(&task).unwrap();
        let param_path = std::path::Path::new(&task.workdir)
            .join(format!("{}.param", task.id));
        let content = std::fs::read_to_string(&param_path).unwrap();
        assert!(content.contains("CUT_OFF_ENERGY"));
        assert!(content.contains("400"));
        let _ = std::fs::remove_dir_all(&task.workdir);
    }
}
