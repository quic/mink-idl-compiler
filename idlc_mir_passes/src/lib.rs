//! Contains passes made on the MIR before validating that it's a valid MIR.
//!
//! This involves
//! 1. Interface function name should not be duplicated
//! 2. Interface error name should not be duplicated
//! 3. Argument names within a method must be unique

use idlc_ast::Ident;

pub trait MirCompilerPass<'mir> {
    type Output;

    fn run_pass(&'mir mut self) -> Result<Self::Output, Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Const already defined: {} {}", first.ident, second.ident)]
    AlreadyDefinedConst { first: Ident, second: Ident },
    #[error("Method already defined: {} {}", first.ident, second.ident)]
    AlreadyDefinedMethod { first: Ident, second: Ident },
    #[error("Error already defined: {} {}", first.ident, second.ident)]
    AlreadyDefinedError { first: Ident, second: Ident },
    #[error("Duplicated argument names within a method: {} {}", first.ident, second.ident)]
    DuplicateArgumentName { first: Ident, second: Ident },
}

pub mod interface_verifier;
