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

use std::collections::HashMap;

use crate::ast::{Ident, Node};

/// Compilation unit is split into a hashmap here.
pub struct ASTStore(HashMap<String, Vec<Node>>);

impl ASTStore {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get_or_insert(&mut self, file_path: &str) -> Result<&[Node], Error> {
        if !self.0.contains_key(file_path) {
            let Node::CompilationUnit(_, nodes) = Node::from_file(file_path)? else { unreachable!("ICE: Cannot find root node in AST from file.")};
            self.0.insert(file_path.to_string(), nodes);
        }

        Ok(unsafe { self.0.get(file_path).unwrap_unchecked() })
    }
}

pub trait CompilerPass<'ast, T> {
    fn run_pass(ast: &'ast Node) -> Result<T, Error>;
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
