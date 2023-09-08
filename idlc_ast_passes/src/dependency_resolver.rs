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

use idlc_ast::{Interface, Node, Struct};

use idlc_ast::visitor::{walk_all, Visitor};

/// DependencyResolver structure
/// Compilation unit is split into a hashmap here.
#[derive(Debug)]
pub struct DependencyResolver {
    ast_store: RefCell<HashMap<PathBuf, Rc<Node>>>,
    symbols: RefCell<HashMap<Symbol, Rc<Node>>>,
    current: Option<PathBuf>,
    cycle: Option<Cycle<String>>,
    graph: Graph<String>,
    include_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Symbol {
    Struct(String),
    Interface(String),
}

impl Visitor<'_> for DependencyResolver {
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

        let inc_ast = self.get_or_insert(&cano_path).unwrap();
        walk_all(self, &inc_ast);
        self.current = Some(current);
    }
}

impl CompilerPass<'_> for DependencyResolver {
    type Output = Vec<String>;

    fn run_pass(&'_ mut self, ast: &'_ idlc_ast::Node) -> Result<Self::Output, crate::Error> {
        self.check_includes(ast)
    }
}

impl DependencyResolver {
    /// takes the vector of include paths as an argument which will be saved in include_paths field
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

    pub fn new() -> Self {
        Self::with_includes(&[])
    }

    /// returns the AST corresponding to the given path
    /// returns None if no such AST exists
    pub fn get_ast(&self, path: &Path) -> Option<Rc<Node>> {
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
            eprintln!(
                "Warning: Found multiple files with the same name '{}' in different paths: ",
                path.display()
            );
            for f in &set_found_files {
                eprintln!("{}", f.display());
            }
            eprintln!("Selecting '{}'", selected_path.display());
        }
        selected_path.to_path_buf()
    }

    fn check_includes(&mut self, ast: &'_ idlc_ast::Node) -> Result<Vec<String>, crate::Error> {
        walk_all(self, ast);
        if let Some(cycle) = self.cycle.take() {
            return Err(crate::Error::CyclicalInclude(cycle));
        }

        Ok(self.graph.toposort().unwrap())
    }

    #[inline]
    fn gather_symbols_from_ast(ast: &Node, map: &mut HashMap<Symbol, Rc<Node>>) {
        let Node::CompilationUnit(_, nodes) = ast else {
            unreachable!("ICE: Cannot find root node in AST from file. {ast:?}")
        };
        for node in nodes {
            assert_eq!(
                match node.as_ref() {
                    Node::Struct(s) =>
                        map.insert(Symbol::Struct(s.ident.to_string()), Rc::clone(node)),
                    Node::Interface(i) => {
                        map.insert(Symbol::Interface(i.ident.to_string()), Rc::clone(node))
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
    pub fn get_or_insert(&self, file_path: &Path) -> Option<Rc<Node>> {
        let mut include_path = PathBuf::from(file_path);
        if !self.ast_store.borrow().contains_key(file_path) {
            include_path = file_path
                .canonicalize()
                .expect("Failed to canonicalize path.");
            let node = Node::from_file(include_path.clone()).unwrap_or_else(|e| {
                println!("Parsing failed: \n{e}\n");
                std::process::exit(0);
            });
            Self::insert_canonical(self, &include_path, &Rc::new(node.clone()));
        }

        Some(Rc::clone(
            self.ast_store.borrow().get(&include_path).unwrap(),
        ))
    }

    ///  inserts canonicalized path into the AST store along with its AST
    pub fn insert_canonical(&self, canonical: &Path, node: &Rc<Node>) {
        Self::gather_symbols_from_ast(node, &mut self.symbols.borrow_mut());
        self.ast_store
            .borrow_mut()
            .insert(canonical.to_path_buf(), Rc::clone(node));
    }

    /// returns the interface corresponding to the given name
    pub fn struct_lookup(&self, name: &str) -> Option<Rc<Struct>> {
        self.symbols
            .borrow()
            .get(&Symbol::Struct(name.to_string()))
            .map(|node| {
                let Node::Struct(s) = node.as_ref() else {
                    unreachable!("ICE: Struct node expected.")
                };
                Rc::new(s.clone())
            })
    }

    /// returns the interface corresponding to the given name
    pub fn iface_lookup(&self, name: &str) -> Option<Rc<Interface>> {
        self.symbols
            .borrow()
            .get(&Symbol::Interface(name.to_string()))
            .map(|node| {
                let Node::Interface(i) = node.as_ref() else {
                    unreachable!("ICE: Interface node expected.")
                };
                Rc::new(i.clone())
            })
    }
}
