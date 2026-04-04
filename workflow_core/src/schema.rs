//! TOML workflow schema types and sweep expansion.
//!
//! # Example workflow TOML
//!
//! ```toml
//! [workflow]
//! name = "u_convergence"
//! working_dir = "/scratch/runs"
//!
//! [executors.local]
//! type = "local"
//! parallelism = 4
//!
//! [executors.slurm]
//! type = "slurm"
//! partition = "compute"
//! ntasks = 32
//! walltime = "04:00:00"
//!
//! # Single task, no sweep
//! [[tasks]]
//! id = "preprocess"
//! code = "script"
//! executor = "local"
//! workdir = "runs/pre"
//!
//! # Zip sweep: params advance together (u=2,c=500), (u=3,c=600), (u=4,c=700)
//! [[tasks]]
//! id = "scf_U{u}_E{cutoff}"
//! code = "castep"
//! executor = "slurm"
//! workdir = "runs/U{u}_E{cutoff}"
//! depends_on = ["preprocess"]
//!
//! [tasks.sweep]
//! mode = "zip"
//! params = [
//!     { name = "u",      values = [2.0, 3.0, 4.0] },
//!     { name = "cutoff", values = [500, 600, 700]  },
//! ]
//!
//! [tasks.inputs]
//! HUBBARD_U = "{u}"
//! CUT_OFF_ENERGY = "{cutoff}"
//!
//! # Product sweep: all combinations — 2×2 = 4 tasks
//! [[tasks]]
//! id = "dos_U{u}_E{cutoff}"
//! code = "castep"
//! executor = "slurm"
//! workdir = "runs/dos/U{u}_E{cutoff}"
//! depends_on = ["scf_U{u}_E{cutoff}"]
//!
//! [tasks.sweep]
//! mode = "product"
//! params = [
//!     { name = "u",      values = [2.0, 4.0] },
//!     { name = "cutoff", values = [500, 700]  },
//! ]
//! ```

use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WorkflowDef {
    pub workflow: WorkflowMeta,
    #[serde(default)]
    pub executors: HashMap<String, ExecutorDef>,
    #[serde(default)]
    pub tasks: Vec<TaskDef>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowMeta {
    pub name: String,
    pub working_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ExecutorDef {
    Local { parallelism: usize },
    Slurm { partition: String, ntasks: u32, walltime: String },
}

#[derive(Debug, Deserialize)]
pub struct TaskDef {
    pub id: String,
    pub code: String,
    pub executor: String,
    pub workdir: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub sweep: Option<SweepDef>,
    #[serde(default)]
    pub inputs: HashMap<String, toml::Value>,
}

#[derive(Debug, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SweepMode {
    /// All combinations of all parameter values (cartesian product).
    Product,
    /// Parameters advance together pairwise; all value lists must be equal length.
    #[default]
    Zip,
}

#[derive(Debug, Deserialize)]
pub struct ParamSweep {
    pub name: String,
    pub values: Vec<toml::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SweepDef {
    #[serde(default)]
    pub mode: SweepMode,
    pub params: Vec<ParamSweep>,
}

#[derive(Debug, Clone)]
pub struct ConcreteTask {
    pub id: String,
    pub code: String,
    pub executor: String,
    pub workdir: String,
    pub depends_on: Vec<String>,
    pub inputs: HashMap<String, toml::Value>,
    pub executor_def: ExecutorDef,
}

fn val_to_str(v: &toml::Value) -> String {
    match v {
        toml::Value::Float(f)   => format!("{f}"),
        toml::Value::Integer(i) => format!("{i}"),
        other                   => other.to_string(),
    }
}

fn apply_bindings(template: &str, bindings: &[(&str, &str)]) -> String {
    bindings.iter().fold(template.to_owned(), |s, (k, v)| {
        s.replace(&format!("{{{k}}}"), v)
    })
}

fn make_task(base: &TaskDef, bindings: &[(&str, &str)], extra_inputs: HashMap<String, toml::Value>, executor_def: ExecutorDef) -> ConcreteTask {
    let mut inputs = base.inputs.clone();
    inputs.extend(extra_inputs);
    ConcreteTask {
        id:         apply_bindings(&base.id, bindings),
        code:       base.code.clone(),
        executor:   base.executor.clone(),
        workdir:    apply_bindings(&base.workdir, bindings),
        depends_on: base.depends_on.iter().map(|d| apply_bindings(d, bindings)).collect(),
        inputs,
        executor_def,
    }
}

/// Expand a `SweepDef` into a list of binding sets.
/// Each binding set is `Vec<(param_name, value_string, toml_value)>`.
fn expand_bindings(sweep: &SweepDef) -> anyhow::Result<Vec<Vec<(&str, String, toml::Value)>>> {
    match sweep.mode {
        SweepMode::Zip => {
            let len = sweep.params[0].values.len();
            for p in &sweep.params {
                if p.values.len() != len {
                    anyhow::bail!(
                        "zip sweep param '{}' has {} values but expected {}",
                        p.name, p.values.len(), len
                    );
                }
            }
            Ok((0..len).map(|i| {
                sweep.params.iter().map(|p| {
                    (p.name.as_str(), val_to_str(&p.values[i]), p.values[i].clone())
                }).collect()
            }).collect())
        }
        SweepMode::Product => {
            // Cartesian product via fold
            Ok(sweep.params.iter().fold(vec![vec![]], |acc, param| {
                acc.into_iter().flat_map(|combo| {
                    param.values.iter().map(move |v| {
                        let mut c = combo.clone();
                        c.push((param.name.as_str(), val_to_str(v), v.clone()));
                        c
                    }).collect::<Vec<_>>()
                }).collect()
            }))
        }
    }
}

pub fn expand_sweeps(def: WorkflowDef) -> anyhow::Result<Vec<ConcreteTask>> {
    let mut out = Vec::new();
    for task in &def.tasks {
        let executor_def = def.executors.get(&task.executor)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!(
                "task '{}' references unknown executor '{}'", task.id, task.executor
            ))?;
        match &task.sweep {
            None => out.push(ConcreteTask {
                id:         task.id.clone(),
                code:       task.code.clone(),
                executor:   task.executor.clone(),
                workdir:    task.workdir.clone(),
                depends_on: task.depends_on.clone(),
                inputs:     task.inputs.clone(),
                executor_def: executor_def.clone(),
            }),
            Some(sweep) => {
                for binding_set in expand_bindings(sweep)? {
                    let str_bindings: Vec<(String, String)> = binding_set.iter()
                        .map(|(k, v, _)| (k.to_string(), v.clone()))
                        .collect();
                    let refs: Vec<(&str, &str)> = str_bindings.iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    let extra: HashMap<String, toml::Value> = binding_set.into_iter()
                        .map(|(k, _, v)| (k.to_owned(), v))
                        .collect();
                    out.push(make_task(task, &refs, extra, executor_def.clone()));
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sweep_def(mode: SweepMode, params: Vec<(&str, Vec<toml::Value>)>) -> SweepDef {
        SweepDef {
            mode,
            params: params.into_iter().map(|(name, values)| ParamSweep {
                name: name.to_owned(),
                values,
            }).collect(),
        }
    }

    fn ints(vs: &[i64]) -> Vec<toml::Value> {
        vs.iter().map(|&i| toml::Value::Integer(i)).collect()
    }

    #[test]
    fn no_sweep_passes_through() {
        let mut executors = HashMap::new();
        executors.insert("local".into(), ExecutorDef::Local { parallelism: 1 });
        let def = WorkflowDef {
            workflow: WorkflowMeta { name: "w".into(), working_dir: "/tmp".into() },
            executors,
            tasks: vec![TaskDef {
                id: "scf".into(), code: "castep".into(), executor: "local".into(),
                workdir: "runs/scf".into(), depends_on: vec![], sweep: None,
                inputs: HashMap::new(),
            }],
        };
        let tasks = expand_sweeps(def).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "scf");
    }

    #[test]
    fn zip_sweep_produces_paired_tasks() {
        let sweep = make_sweep_def(SweepMode::Zip, vec![
            ("u", ints(&[2, 3, 4])),
            ("c", ints(&[500, 600, 700])),
        ]);
        let bindings = expand_bindings(&sweep).unwrap();
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings[0][0].1, "2");
        assert_eq!(bindings[0][1].1, "500");
        assert_eq!(bindings[2][0].1, "4");
        assert_eq!(bindings[2][1].1, "700");
    }

    #[test]
    fn product_sweep_produces_cartesian_tasks() {
        let sweep = make_sweep_def(SweepMode::Product, vec![
            ("u", ints(&[2, 3])),
            ("c", ints(&[500, 700])),
        ]);
        let bindings = expand_bindings(&sweep).unwrap();
        assert_eq!(bindings.len(), 4);
        let ids: Vec<String> = bindings.iter()
            .map(|b| format!("u{}_c{}", b[0].1, b[1].1))
            .collect();
        assert!(ids.contains(&"u2_c500".to_owned()));
        assert!(ids.contains(&"u2_c700".to_owned()));
        assert!(ids.contains(&"u3_c500".to_owned()));
        assert!(ids.contains(&"u3_c700".to_owned()));
    }

    #[test]
    fn template_substitution_in_id_and_workdir() {
        let mut executors = HashMap::new();
        executors.insert("local".into(), ExecutorDef::Local { parallelism: 1 });
        let def = WorkflowDef {
            workflow: WorkflowMeta { name: "w".into(), working_dir: "/tmp".into() },
            executors,
            tasks: vec![TaskDef {
                id: "scf_U{u}".into(), code: "castep".into(), executor: "local".into(),
                workdir: "runs/U{u}".into(), depends_on: vec!["base_U{u}".into()],
                sweep: Some(make_sweep_def(SweepMode::Zip, vec![("u", ints(&[2, 3]))])),
                inputs: HashMap::new(),
            }],
        };
        let tasks = expand_sweeps(def).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "scf_U2");
        assert_eq!(tasks[0].workdir, "runs/U2");
        assert_eq!(tasks[0].depends_on[0], "base_U2");
        assert_eq!(tasks[1].id, "scf_U3");
    }
}
