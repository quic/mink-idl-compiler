use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::{ASTStore, CompilerPass};
use petgraph::algo::toposort;
use petgraph::stable_graph::StableDiGraph;

use crate::ast::visitor::{walk_all, Visitor};

#[track_caller]
#[cold]
#[inline(never)]
fn cannot_find_dep(reason: &str) -> ! {
    eprintln!("{reason}");
    std::process::exit(1)
}

#[track_caller]
#[cold]
#[inline(never)]
fn cyclical_dep_found(inc: &str) -> ! {
    eprintln!("Direct cyclical dependency found `{inc}` depends on `{inc}`");
    std::process::exit(1)
}

pub struct Includes<'a> {
    current: Option<Rc<String>>,
    map: HashMap<Rc<String>, HashSet<String>>,
    visited: HashSet<Rc<String>>,
    ast_store: &'a ASTStore,
}

impl Visitor<'_> for Includes<'_> {
    fn visit_root_ident(&mut self, root_ident: &'_ str) {
        let ident = Rc::new(root_ident.to_string());
        self.current = Some(Rc::clone(&ident));
        self.map.insert(ident, HashSet::new());
    }

    fn visit_include(&mut self, include: &'_ str) {
        let inc = include.to_string();
        let current = self.current.take().unwrap();
        self.visited.insert(Rc::clone(&current));
        if let Some(set) = self.map.get_mut(&current) {
            if self.visited.contains(&inc) {
                cyclical_dep_found(include);
            }
            set.insert(inc);
            let Ok(inc_ast) = self.ast_store.get_or_insert(include) else { cannot_find_dep("Cannot load AST, file not found") };
            walk_all(self, &inc_ast);
            self.current = Some(current);
        } else {
            unreachable!("ICE: Unknown root is trying to add children");
        }
    }
}

impl<'a> Includes<'a> {
    pub fn new(ast_store: &'a ASTStore) -> Self {
        Self {
            current: None,
            map: HashMap::new(),
            visited: HashSet::new(),
            ast_store,
        }
    }

    fn is_success(&self) -> Result<Vec<String>, super::Error> {
        let mut graph = StableDiGraph::new();
        let mut node_map = HashMap::new();
        for key in self.map.keys() {
            node_map.insert(key.as_str(), graph.add_node(key.as_str()));
        }

        for (source, sinks) in &self.map {
            let from = node_map.get(source.as_str()).unwrap();
            for sink in sinks {
                let to = node_map.get(sink.as_str()).unwrap();
                // Dependency inverts the graph.
                graph.add_edge(*to, *from, ());
            }
        }

        toposort(&graph, None)
            .map(|node| node.into_iter().map(|idx| graph[idx].to_string()).collect())
            .map_err(|_| super::Error::CyclicalInclude)
    }
}

impl CompilerPass<'_> for Includes<'_> {
    type Output = Vec<String>;

    fn run_pass(&'_ mut self, ast: &'_ crate::ast::Node) -> Result<Self::Output, super::Error> {
        walk_all(self, ast);
        self.is_success()
    }
}
