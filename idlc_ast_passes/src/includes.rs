use crate::graph::{Cycle, Graph};

use super::{ASTStore, CompilerPass};

use idlc_ast::visitor::{walk_all, Visitor};

#[track_caller]
#[cold]
#[inline(never)]
fn cannot_find_dep(reason: &str) -> ! {
    eprintln!("{reason}");
    std::process::exit(1)
}

pub struct Includes<'a> {
    current: Option<String>,
    cycle: Option<Cycle<String>>,
    graph: Graph<String>,
    ast_store: &'a ASTStore,
}

impl Visitor<'_> for Includes<'_> {
    fn visit_root_ident(&mut self, root_ident: &'_ str) {
        self.current = Some(root_ident.to_string());
    }

    fn visit_include(&mut self, include: &'_ str) {
        if self.cycle.is_some() {
            return;
        }

        let current = self.current.take().unwrap();
        self.graph.add_edge(current.clone(), include.to_string());
        self.cycle = self.graph.cycle();
        let Ok(inc_ast) = self.ast_store.get_or_insert(include) else { cannot_find_dep("Cannot load AST, file not found") };
        walk_all(self, &inc_ast);
        self.current = Some(current);
    }
}

impl<'a> Includes<'a> {
    pub fn new(ast_store: &'a ASTStore) -> Self {
        Self {
            ast_store,
            current: None,
            cycle: None,
            graph: Graph::new(),
        }
    }
}

impl CompilerPass<'_> for Includes<'_> {
    type Output = Vec<String>;

    fn run_pass(&'_ mut self, ast: &'_ idlc_ast::Node) -> Result<Self::Output, super::Error> {
        walk_all(self, ast);
        if let Some(cycle) = self.cycle.take() {
            return Err(crate::Error::CyclicalInclude(cycle));
        }

        Ok(self.graph.toposort().unwrap())
    }
}
