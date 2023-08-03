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

use crate::ast::{Ident, Node};

/// Compilation unit is split into a hashmap here.
#[derive(Default, Debug)]
pub struct ASTStore(RefCell<HashMap<String, Rc<Node>>>);

impl ASTStore {
    pub fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }

    pub fn get_or_insert(&self, file_path: &str) -> Result<Rc<Node>, Error> {
        if !self.0.borrow().contains_key(file_path) {
            let node =
                Node::from_file(file_path).expect("ICE: Cannot find root node in AST from file.");
            self.0
                .borrow_mut()
                .insert(file_path.to_string(), Rc::new(node));
        }

        Ok(unsafe { Rc::clone(self.0.borrow().get(file_path).unwrap_unchecked()) })
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

pub mod duplicate;
pub mod includes;
// mod resolve_symbols;
