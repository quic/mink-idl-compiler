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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cylical imports")]
    CyclicalInclude,
    #[error("Input AST doesn't contain AstNode::CompilationUnit")]
    AstDoesntContainRoot,
    #[error("Error parsing AST")]
    AstParse(#[from] crate::ast::Error),
    #[error("Duplicate definition of '{ident}' in '{root}'")]
    DuplicateDefinition { root: String, ident: String },
    #[error("Couldn't find defintions of the following symbols: {0:?}")]
    UnresolvedSymbols(std::collections::HashSet<String>),
}

pub mod duplicate;
pub mod includes;
// mod resolve_symbols;
