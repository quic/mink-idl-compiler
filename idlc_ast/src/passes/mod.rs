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

use crate::ast::{Ident, Node, Struct};

/// Compilation unit is split into a hashmap here.
#[derive(Default, Debug)]
pub struct ASTStore {
    ast_store: RefCell<HashMap<String, Rc<Node>>>,
    symbol_map: RefCell<HashMap<String, Struct>>,
}

impl ASTStore {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn gather_symbols_from_ast(ast: &Node, map: &mut HashMap<String, Struct>) {
        let Node::CompilationUnit(_, nodes) = ast else { unreachable!("ICE: Cannot find root node in AST from file. {ast:?}") };
        for node in nodes {
            if let Node::Struct(s) = node.as_ref() {
                if let Some(prev) = map.insert(s.ident.to_string(), s.clone()) {
                    panic!(
                        "Duplicate symbol detected, previously defined @ {:?}, defined again @ {:?}",
                        prev.ident.span, s.ident.span
                    );
                }
            }
        }
    }

    pub fn get_or_insert(&self, file_path: &str) -> Result<Rc<Node>, Error> {
        if !self.ast_store.borrow().contains_key(file_path) {
            let node =
                Node::from_file(file_path).expect("ICE: Cannot find root node in AST from file.");
            Self::gather_symbols_from_ast(&node, &mut self.symbol_map.borrow_mut());
            self.ast_store
                .borrow_mut()
                .insert(file_path.to_string(), Rc::new(node));
        }

        Ok(unsafe { Rc::clone(self.ast_store.borrow().get(file_path).unwrap_unchecked()) })
    }

    pub fn symbol_lookup(&self, name: &str) -> Option<Struct> {
        self.symbol_map.borrow().get(name).cloned()
    }
}

pub trait CompilerPass<'ast> {
    type Output;

    fn run_pass(&'ast mut self, ast: &'ast Node) -> Result<Self::Output, Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cylical imports found")]
    CyclicalInclude,
    #[error("Input AST doesn't contain AstNode::CompilationUnit")]
    AstDoesntContainRoot,
    #[error("Error parsing AST")]
    AstParse(#[from] crate::ast::Error),
    #[error("Duplicate definition of '{}'", occ1.ident)]
    DuplicateDefinition { occ1: Ident, occ2: Ident },
    #[error("Couldn't find defintions of the following symbols: {0:?}")]
    UnresolvedSymbols(std::collections::HashSet<String>),
}

pub mod includes;
// mod resolve_symbols;
