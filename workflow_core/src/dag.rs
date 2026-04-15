use crate::error::WorkflowError;
use petgraph::algo;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

pub struct Dag {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl Default for Dag {
    fn default() -> Self {
        Self::new()
    }
}

impl Dag {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: String) -> Result<(), WorkflowError> {
        if self.node_map.contains_key(&id) {
            return Err(WorkflowError::DuplicateTaskId(id));
        }
        let ni = self.graph.add_node(id.clone());
        self.node_map.insert(id, ni);
        Ok(())
    }

    /// Edge: from (dep) → to (dependent). Errors if node missing or cycle created.
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<(), WorkflowError> {
        let &f = self
            .node_map
            .get(from)
            .ok_or_else(|| WorkflowError::UnknownDependency {
                task: from.to_string(),
                dependency: to.to_string(),
            })?;
        let &t = self
            .node_map
            .get(to)
            .ok_or_else(|| WorkflowError::UnknownDependency {
                task: to.to_string(),
                dependency: from.to_string(),
            })?;
        let eid = self.graph.add_edge(f, t, ());
        if algo::toposort(&self.graph, None).is_err() {
            self.graph.remove_edge(eid);
            return Err(WorkflowError::CycleDetected);
        }
        Ok(())
    }

    pub fn topological_order(&self) -> Vec<String> {
        algo::toposort(&self.graph, None)
            .unwrap_or_default()
            .into_iter()
            .map(|ni| self.graph[ni].clone())
            .collect()
    }

    /// Tasks not in `completed` whose every incoming neighbour is in `completed`.
    pub fn ready_tasks(&self, completed: &HashSet<String>) -> Vec<String> {
        self.node_map
            .keys()
            .filter(|id| {
                !completed.contains(*id)
                    && self
                        .graph
                        .neighbors_directed(self.node_map[*id], petgraph::Direction::Incoming)
                        .all(|ni| completed.contains(&self.graph[ni]))
            })
            .cloned()
            .collect()
    }

    pub fn successors(&self, id: &str) -> Vec<String> {
        self.node_map
            .get(id)
            .map(|&ni| {
                self.graph
                    .neighbors(ni)
                    .map(|s| self.graph[s].clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn task_ids(&self) -> impl Iterator<Item = &String> {
        self.node_map.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::WorkflowError;

    fn make_dag(nodes: &[&str], edges: &[(&str, &str)]) -> Dag {
        let mut dag = Dag::new();
        for &id in nodes {
            dag.add_node(id.to_owned()).unwrap();
        }
        for &(f, t) in edges {
            dag.add_edge(f, t).unwrap();
        }
        dag
    }

    #[test]
    fn topo_order_respects_deps() {
        let dag = make_dag(&["a", "b", "c"], &[("a", "b"), ("b", "c")]);
        let order = dag.topological_order();
        let pa = order.iter().position(|x| x == "a").unwrap();
        let pb = order.iter().position(|x| x == "b").unwrap();
        let pc = order.iter().position(|x| x == "c").unwrap();
        assert!(pa < pb && pb < pc);
    }

    #[test]
    fn unknown_dep_errors() {
        let mut dag = Dag::new();
        dag.add_node("b".to_owned()).unwrap();
        assert!(matches!(
            dag.add_edge("missing", "b").unwrap_err(),
            WorkflowError::UnknownDependency { task: _, dependency: _ }
        ));
    }

    #[test]
    fn cycle_detection() {
        let mut dag = make_dag(&["a", "b"], &[("a", "b")]);
        assert!(matches!(dag.add_edge("b", "a").unwrap_err(), WorkflowError::CycleDetected));
    }

    #[test]
    fn ready_tasks_test() {
        let dag = make_dag(&["a", "b"], &[("a", "b")]);
        let completed: HashSet<String> = ["a".to_owned()].into();
        let ready = dag.ready_tasks(&completed);
        assert_eq!(ready, vec!["b"]);
    }

    #[test]
    fn duplicate_node_errors() {
        let mut dag = Dag::new();
        dag.add_node("x".to_owned()).unwrap();
        assert!(matches!(
            dag.add_node("x".to_owned()).unwrap_err(),
            WorkflowError::DuplicateTaskId(_)
        ));
    }
}
