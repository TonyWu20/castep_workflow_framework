//! DAG-based pipeline representation and traversal.

use std::collections::HashMap;
use anyhow::{Result, bail};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Topo;
use petgraph::algo;
use crate::schema::ConcreteTask;

/// A directed acyclic graph of [`ConcreteTask`]s representing a workflow pipeline.
///
/// Edges point from dependency to dependent (parent → child).
pub struct Pipeline {
    /// The underlying petgraph DAG. Nodes are concrete tasks; edges are dependencies.
    pub graph: DiGraph<ConcreteTask, ()>,
    index: HashMap<String, NodeIndex>,
}

impl Pipeline {
    /// Build a `Pipeline` from a flat list of tasks, wiring dependency edges.
    ///
    /// Returns an error if any `depends_on` entry references an unknown task ID.
    pub fn from_tasks(tasks: Vec<ConcreteTask>) -> Result<Self> {
        let mut graph = DiGraph::new();
        let mut index = HashMap::new();

        for task in tasks {
            let id = task.id.clone();
            let ni = graph.add_node(task);
            index.insert(id, ni);
        }

        let edges: Vec<(NodeIndex, NodeIndex)> = graph
            .node_indices()
            .flat_map(|ni| {
                graph[ni].depends_on.iter().filter_map(|dep| {
                    index.get(dep).map(|&dep_ni| (dep_ni, ni))
                }).collect::<Vec<_>>()
            })
            .collect();

        for (from, to) in edges {
            graph.add_edge(from, to, ());
        }

        if algo::is_cyclic_directed(&graph) {
            bail!("pipeline contains a cycle");
        }

        for ni in graph.node_indices() {
            for dep in &graph[ni].depends_on {
                if !index.contains_key(dep) {
                    bail!("task '{}' depends on unknown task '{}'", graph[ni].id, dep);
                }
            }
        }

        Ok(Self { graph, index })
    }

    /// Return task nodes in topological order (dependencies before dependents).
    pub fn topological_order(&self) -> Vec<NodeIndex> {
        let mut topo = Topo::new(&self.graph);
        let mut order = Vec::new();
        while let Some(ni) = topo.next(&self.graph) {
            order.push(ni);
        }
        order
    }

    /// Return all tasks that directly depend on `id` (outgoing neighbours).
    pub fn successors(&self, id: &str) -> Vec<&ConcreteTask> {
        self.index.get(id).map(|&ni| {
            self.graph.neighbors(ni).map(|s| &self.graph[s]).collect()
        }).unwrap_or_default()
    }

    /// Return all tasks that `id` directly depends on (incoming neighbours).
    pub fn predecessors(&self, id: &str) -> Vec<&ConcreteTask> {
        self.index.get(id).map(|&ni| {
            self.graph.neighbors_directed(ni, petgraph::Direction::Incoming)
                .map(|p| &self.graph[p]).collect()
        }).unwrap_or_default()
    }

    /// Return the petgraph [`NodeIndex`] for a task by ID, if it exists.
    pub fn node_index(&self, id: &str) -> Option<NodeIndex> {
        self.index.get(id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, depends_on: Vec<&str>) -> ConcreteTask {
        ConcreteTask {
            id: id.to_owned(),
            code: "castep".into(),
            executor: "local".into(),
            workdir: format!("runs/{id}"),
            depends_on: depends_on.into_iter().map(str::to_owned).collect(),
            inputs: std::collections::HashMap::new(),
            executor_def: crate::schema::ExecutorDef::Local { parallelism: 1 },
        }
    }

    #[test]
    fn builds_dag_and_topo_order_respects_deps() {
        // a → b → c
        let pipeline = Pipeline::from_tasks(vec![
            task("a", vec![]),
            task("b", vec!["a"]),
            task("c", vec!["b"]),
        ]).unwrap();

        let order: Vec<&str> = pipeline.topological_order().iter()
            .map(|&ni| pipeline.graph[ni].id.as_str())
            .collect();

        let pos = |id| order.iter().position(|&x| x == id).unwrap();
        assert!(pos("a") < pos("b"));
        assert!(pos("b") < pos("c"));
    }

    #[test]
    fn unknown_dependency_returns_error() {
        let result = Pipeline::from_tasks(vec![task("b", vec!["nonexistent"])]);
        assert!(result.is_err());
    }

    #[test]
    fn successors_and_predecessors_are_correct() {
        // a → b, a → c
        let pipeline = Pipeline::from_tasks(vec![
            task("a", vec![]),
            task("b", vec!["a"]),
            task("c", vec!["a"]),
        ]).unwrap();

        let mut succ: Vec<&str> = pipeline.successors("a").iter().map(|t| t.id.as_str()).collect();
        succ.sort();
        assert_eq!(succ, vec!["b", "c"]);

        let pred: Vec<&str> = pipeline.predecessors("b").iter().map(|t| t.id.as_str()).collect();
        assert_eq!(pred, vec!["a"]);
    }

    #[test]
    fn cyclic_dependency_returns_error() {
        // a → b → a
        let result = Pipeline::from_tasks(vec![
            task("a", vec!["b"]),
            task("b", vec!["a"]),
        ]);
        assert!(result.is_err());
    }
}
