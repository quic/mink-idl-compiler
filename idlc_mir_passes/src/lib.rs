//! Contains passes made on the MIR before validating that it's a valid MIR.
//!
//! This involves
//! 1. Interface function name should not be duplicated
//! 2. Interface error name should not be duplicated
//! 3. Interface consts should not be duplicated, error definitions are also
//!    considered consts

pub trait MirCompilerPass<'mir> {
    type Output;

    fn run_pass(&'mir mut self) -> Self::Output;
}

pub mod interface_verifier;
