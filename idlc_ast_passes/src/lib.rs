//! Contains passes made on the AST before validating that it's a valid AST.
//!
//! This involves
//! 1. Finding if there are cyclic includes and if not creating a topological
//!    ordering the includes to resolve external symbols.
//! 2. Gathering all the interfaces that the AST depends on and generating ASTs
//!    for those (possibly parallely)
//! 3. Creating a list of symbols that require externally resolving and create a
//!    datastructure containing dependencies from different includes that are
//!    used.
//!        - warn on unused includes.
//!        - error on name clases in includes. This
//!        - generating symbol level includes instead of interfaces in cases
//!          where the language being transpiled to supports it, like Rust.
//! 4. Creating a dependency tree data structure that contain symbols required
//!    from each external include.

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use idlc_ast::{Ident, Interface, Node, Struct};

/// Compilation unit is split into a hashmap here.
#[derive(Default, Debug)]
pub struct ASTStore {
    ast_store: RefCell<HashMap<String, Rc<Node>>>,
    symbols: RefCell<HashMap<Symbol, Rc<Node>>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Symbol {
    Struct(String),
    Interface(String),
}

impl ASTStore {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn gather_symbols_from_ast(ast: &Node, map: &mut HashMap<Symbol, Rc<Node>>) {
        let Node::CompilationUnit(_, nodes) = ast else { unreachable!("ICE: Cannot find root node in AST from file. {ast:?}") };
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

    pub fn get_or_insert(&self, file_path: &str) -> Option<Rc<Node>> {
        if !self.ast_store.borrow().contains_key(file_path) {
            let node =
                Node::from_file(file_path).expect("ICE: Cannot find root node in AST from file.");
            Self::gather_symbols_from_ast(&node, &mut self.symbols.borrow_mut());
            self.ast_store
                .borrow_mut()
                .insert(file_path.to_string(), Rc::new(node));
        }

        Some(Rc::clone(self.ast_store.borrow().get(file_path).unwrap()))
    }

    pub fn insert(&self, name: &str, node: &Rc<Node>) {
        Self::gather_symbols_from_ast(node, &mut self.symbols.borrow_mut());
        self.ast_store
            .borrow_mut()
            .insert(name.to_string(), Rc::clone(node));
    }

    pub fn struct_lookup(&self, name: &str) -> Option<Rc<Struct>> {
        self.symbols
            .borrow()
            .get(&Symbol::Struct(name.to_string()))
            .map(|node| {
                let Node::Struct(s) = node.as_ref() else {unreachable!("ICE: Struct node expected.")};
                Rc::new(s.clone())
            })
    }

    pub fn iface_lookup(&self, name: &str) -> Option<Rc<Interface>> {
        self.symbols
            .borrow()
            .get(&Symbol::Interface(name.to_string()))
            .map(|node| {
                let Node::Interface(i) = node.as_ref() else {unreachable!("ICE: Interface node expected.")};
                Rc::new(i.clone())
            })
    }
}

pub trait CompilerPass<'ast> {
    type Output;

    fn run_pass(&'ast mut self, ast: &'ast Node) -> Result<Self::Output, Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cylical imports found {0:?}")]
    CyclicalInclude(Vec<String>),
    #[error("Input AST doesn't contain AstNode::CompilationUnit")]
    AstDoesntContainRoot,
    #[error("Error parsing AST")]
    AstParse(#[from] idlc_ast::Error),
    #[error("Duplicate definition of '{}'", occ1.ident)]
    DuplicateDefinition { occ1: Ident, occ2: Ident },
    #[error("Couldn't find defintions for the symbol `{0}`")]
    UnresolvedSymbol(String),
    #[error("Struct requirements not met: `{0}`")]
    StructVerifier(#[from] struct_verifier::Error),
}

mod graph;

pub mod cycles;
pub mod includes;
pub mod struct_verifier;
