use std::rc::Rc;

use idlc_ast::{
    visitor::{walk_all, Visitor},
    Type,
};

use crate::{dependency_resolver::DependencyResolver, graph::Graph, CompilerPass};

pub struct Cycles<'ast> {
    struct_graph: Graph<String>,
    iface_graph: Graph<String>,
    ast_store: &'ast DependencyResolver,
}

impl<'ast> Cycles<'ast> {
    pub fn new(ast_store: &'ast DependencyResolver) -> Self {
        Self {
            ast_store,
            struct_graph: Graph::new(),
            iface_graph: Graph::new(),
        }
    }

    pub fn visit_iface_recurse(&mut self, iface: Rc<idlc_ast::Interface>) {
        if self.iface_graph.cycle().is_some() {
            return;
        }

        if let Some(base) = &iface.base {
            let base = self.ast_store.iface_lookup(base).unwrap();
            self.iface_graph
                .add_edge(iface.ident.to_string(), base.ident.to_string());
            self.visit_iface_recurse(base);
        }
    }

    pub fn visit_struct_recurse(&mut self, r#struct: Rc<idlc_ast::Struct>) {
        if self.struct_graph.cycle().is_some() {
            return;
        }

        for field in &r#struct.fields {
            if let Type::Custom(c) = &field.r#type().0 {
                let custom = self.ast_store.struct_lookup(c).unwrap();
                self.struct_graph
                    .add_edge(r#struct.ident.to_string(), custom.ident.to_string());
                self.visit_struct_recurse(custom);
            }
        }
    }
}

impl<'ast> Visitor<'ast> for Cycles<'ast> {
    fn visit_interface(&mut self, iface: &'ast idlc_ast::Interface) {
        self.visit_iface_recurse(Rc::new(iface.clone()));
        self.iface_graph.add_node(iface.ident.to_string());
    }

    fn visit_struct(&mut self, r#struct: &'ast idlc_ast::Struct) {
        self.visit_struct_recurse(Rc::new(r#struct.clone()));
        self.struct_graph.add_node(r#struct.ident.to_string());
    }
}

impl<'ast> CompilerPass<'ast> for Cycles<'ast> {
    type Output = Vec<String>;

    /// Returns the topological sort of the struct members to create a dependency chain for struct parsing.
    ///
    /// This is not needed for interfaces as no AST passes are done for interfaces.
    fn run_pass(&'ast mut self, ast: &'ast idlc_ast::Node) -> Result<Self::Output, crate::Error> {
        walk_all(self, ast);
        self.iface_graph
            .toposort()
            .map_err(crate::Error::CyclicalInclude)?;
        self.struct_graph
            .toposort()
            .map_err(crate::Error::CyclicalInclude)
    }
}
