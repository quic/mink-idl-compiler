// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

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
//!       - warn on unused includes.
//!       - error on name clases in includes. This
//!       - generating symbol level includes instead of interfaces in cases
//!         where the language being transpiled to supports it, like Rust.
//! 4. Creating a dependency tree data structure that contain symbols required
//!    from each external include.

use idlc_ast::{Ast, Ident};

pub trait CompilerPass<'ast> {
    type Output;

    fn run_pass(&'ast mut self, ast: &'ast Ast) -> Result<Self::Output, Error>;
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
pub mod functions;
pub mod idl_store;
pub mod struct_verifier;
