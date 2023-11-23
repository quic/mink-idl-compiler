//! Implementation of include cycle detection and symbol lookup
//!
use crate::graph::{Cycle, Graph};

use crate::CompilerPass;

use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use idlc_ast::{Ast, Interface, Node, Struct};

use idlc_ast::visitor::{walk_all, Visitor};
use idlc_errors::warn;

/// `DependencyResolver` structure
/// Compilation unit is split into a hashmap here.
#[derive(Debug)]
pub struct IDLStore {
    ast_store: RefCell<HashMap<PathBuf, Rc<Ast>>>,
    symbols: RefCell<HashMap<Symbol, (Rc<Node>, PathBuf)>>,
    current: Option<PathBuf>,
    cycle: Option<Cycle<String>>,
    graph: Graph<String>,
    include_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Symbol {
    Struct(String),
    Interface(String),
    Const(String),
}

impl Visitor<'_> for IDLStore {
    fn visit_root_ident(&mut self, root_ident: &'_ Path) {
        self.current = Some(root_ident.to_path_buf());
    }

    fn visit_include(&mut self, include: &'_ Path) {
        let current = self.current.take().unwrap();
        let cano_path = self.change_to_canonical(&current, include);
        self.graph.add_edge(
            current.clone().display().to_string(),
            cano_path.display().to_string(),
        );
        self.cycle = self.graph.cycle();
        if self.cycle.is_some() {
            return;
        }

        let inc_ast = self.get_or_insert(&cano_path);
        walk_all(self, &inc_ast);
        self.current = Some(current);
    }
}

impl CompilerPass<'_> for IDLStore {
    type Output = Vec<String>;

    fn run_pass(&'_ mut self, ast: &'_ idlc_ast::Ast) -> Result<Self::Output, crate::Error> {
        self.check_includes(ast)
    }
}

impl IDLStore {
    /// takes the vector of include paths as an argument which will be saved in `include_paths` field
    #[must_use]
    pub fn with_includes(include_paths: &[PathBuf]) -> Self {
        Self {
            ast_store: RefCell::new(HashMap::new()),
            symbols: RefCell::new(HashMap::new()),
            current: None,
            cycle: None,
            graph: Graph::new(),
            include_paths: include_paths.to_vec(),
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self::with_includes(&[])
    }

    /// returns the AST corresponding to the given path
    /// returns None if no such AST exists
    pub fn get_ast(&self, path: &Path) -> Option<Rc<Ast>> {
        Some(Rc::clone(
            self.ast_store.borrow().get(&path.to_path_buf()).unwrap(),
        ))
    }

    fn change_to_canonical<'a>(&mut self, current: &'a Path, path: &'a Path) -> PathBuf {
        if self.ast_store.borrow().contains_key(path) {
            return path.to_path_buf();
        }

        if !path.parent().unwrap().display().to_string().is_empty() {
            let uncanonicalized = current.parent().unwrap().join(path);
            let Ok(canonicalized) = uncanonicalized.canonicalize() else {
                panic!("{uncanonicalized:?} cannot be found!");
            };

            return canonicalized;
        }

        let mut set_found_files = std::collections::HashSet::new();
        for include_path in &self.include_paths {
            let inc_path = include_path.join(path);
            if inc_path.exists() {
                set_found_files.insert(inc_path.canonicalize().unwrap());
            }
        }

        let selected_path = set_found_files.iter().next().expect("File not found");

        if set_found_files.len() > 1 {
            warn!(
                "Found multiple files with the same name '{}' in different paths: ",
                path.display()
            );
            for f in &set_found_files {
                warn!("{}", f.display());
            }
            warn!("Selecting '{}'", selected_path.display());
        }
        selected_path.clone()
    }

    fn check_includes(&mut self, ast: &'_ idlc_ast::Ast) -> Result<Vec<String>, crate::Error> {
        walk_all(self, ast);
        if let Some(cycle) = self.cycle.take() {
            return Err(crate::Error::CyclicalInclude(cycle));
        }

        Ok(self.graph.toposort().unwrap())
    }

    #[inline]
    fn gather_symbols_from_ast(ast: &Ast, map: &mut HashMap<Symbol, (Rc<Node>, PathBuf)>) {
        let tag = &ast.tag;
        for node in &ast.nodes {
            assert_eq!(
                match node.as_ref() {
                    Node::Struct(s) => map.insert(
                        Symbol::Struct(s.ident.to_string()),
                        (Rc::clone(node), tag.clone())
                    ),
                    Node::Interface(i) => {
                        map.insert(
                            Symbol::Struct(i.ident.to_string()),
                            (
                                Rc::new(Node::Struct(Struct::new_object(&i.ident))),
                                tag.clone(),
                            ),
                        )
                        .or_else(|| {
                            map.insert(
                                Symbol::Interface(i.ident.to_string()),
                                (Rc::clone(node), tag.clone()),
                            )
                        })
                    }
                    Node::Const(c) => {
                        map.insert(
                            Symbol::Const(c.ident.to_string()),
                            (Rc::clone(node), tag.clone()),
                        )
                    }
                    _ => None,
                },
                None,
                "Duplicate symbol detected!"
            );
        }
    }

    /// returns the AST corresponding to the given path
    /// if doesn't exist, inserts it into the store and returns it
    pub fn get_or_insert(&self, file_path: &Path) -> Rc<Ast> {
        let mut include_path = PathBuf::from(file_path);
        if !self.ast_store.borrow().contains_key(file_path) {
            include_path = file_path
                .canonicalize()
                .expect("Failed to canonicalize path.");
            let node = idlc_ast::from_file(include_path.clone()).unwrap_or_else(|e| {
                idlc_errors::unrecoverable!("Parsing failed: \n{e}\n");
            });
            Self::insert_canonical(self, &include_path, &node);
        }

        Rc::clone(self.ast_store.borrow().get(&include_path).unwrap())
    }

    ///  inserts canonicalized path into the AST store along with its AST
    pub fn insert_canonical(&self, canonical: &Path, ast: &Ast) {
        Self::gather_symbols_from_ast(ast, &mut self.symbols.borrow_mut());
        self.ast_store
            .borrow_mut()
            .insert(canonical.to_path_buf(), ast.clone().into());
    }

    /// returns the interface corresponding to the given name
    pub fn struct_lookup(&self, name: &str) -> Option<(Rc<Struct>, PathBuf)> {
        self.symbols
            .borrow()
            .get(&Symbol::Struct(name.to_string()))
            .map(|(node, tag)| {
                let Node::Struct(s) = node.as_ref() else {
                    unreachable!("ICE: Struct node expected.")
                };
                (Rc::new(s.clone()), tag.clone())
            })
    }

    /// returns the interface corresponding to the given name
    pub fn iface_lookup(&self, name: &str) -> Option<Rc<Interface>> {
        self.symbols
            .borrow()
            .get(&Symbol::Interface(name.to_string()))
            .map(|(node, _)| {
                let Node::Interface(i) = node.as_ref() else {
                    unreachable!("ICE: Interface node expected.")
                };
                Rc::new(i.clone())
            })
    }
}
