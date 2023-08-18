//! Implementation of a directed graph that allows for toposorting and cycle detection.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Graph<T: Hash + Eq + Clone> {
    nodes: HashMap<T, HashSet<T>>,
}
pub type Ordering<T> = Vec<T>;
pub type Cycle<T> = Vec<T>;

impl<T: Hash + Eq + Clone> Graph<T> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_edge(&mut self, from: T, to: T) {
        self.nodes.entry(from).or_default().insert(to);
    }

    pub fn add_node(&mut self, node: T) {
        self.nodes.entry(node).or_default();
    }

    pub fn toposort(&self) -> Result<Ordering<T>, Cycle<T>> {
        let nodes = self.nodes.len();
        let mut visited = HashSet::with_capacity(nodes);
        let mut branch = Vec::with_capacity(nodes);
        let mut order = Vec::with_capacity(nodes);

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.toposort_recursive(node, &mut visited, &mut branch, &mut order)?;
            }
        }

        Ok(order)
    }

    fn toposort_recursive(
        &self,
        node: &T,
        visited: &mut HashSet<T>,
        branch: &mut Vec<T>,
        order: &mut Vec<T>,
    ) -> Result<(), Cycle<T>> {
        if branch.contains(node) {
            let mut cycle: Vec<T> = branch.clone();
            cycle.push(node.clone());
            return Err(cycle);
        }

        if !visited.contains(node) {
            visited.insert(node.clone());
            branch.push(node.clone());
            let index = branch.len() - 1;

            if let Some(neighbors) = self.nodes.get(node) {
                for neighbor in neighbors {
                    self.toposort_recursive(neighbor, visited, branch, order)?;
                }
            }
            branch.remove(index);
            order.push(node.clone());
        }

        Ok(())
    }

    pub fn cycle(&self) -> Option<Cycle<T>> {
        self.toposort().err()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_cycles() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);
        graph.add_edge(4, 5);
        assert!(graph.cycle().is_none());
        assert!(graph.toposort().is_ok());
    }

    #[test]
    fn square() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);
        graph.add_edge(4, 1);
        assert!(graph.cycle().is_some());
        assert!(graph.toposort().is_err());
    }

    #[test]
    fn complex_graph() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 4);
        graph.add_edge(4, 5);
        graph.add_edge(4, 6);
        graph.add_edge(6, 2);
        assert!(graph.cycle().is_some());
        assert!(graph.toposort().is_err());
    }

    #[test]
    fn toposort() {
        let mut graph = Graph::new();
        graph.add_edge(2, 3);
        graph.add_edge(1, 3);
        graph.add_edge(1, 2);

        assert_eq!(graph.toposort().unwrap(), &[3, 2, 1]);
    }
}
