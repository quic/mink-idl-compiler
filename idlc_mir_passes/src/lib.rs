// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Contains passes made on the MIR before validating that it's a valid MIR.
//!
//! This involves
//! 1. Interface function name should not be duplicated
//! 2. Interface error name should not be duplicated
//! 3. Interface consts should not be duplicated, error definitions are also
//!    considered consts
//! 4. Interface function should not have the argument of struct array which
//!    has the object inside
//! 5. Cannot have input Object array + any type of input objects.
//!    Same for Output

pub trait MirCompilerPass<'mir> {
    type Output;

    fn run_pass(&'mir mut self) -> Self::Output;
}

pub mod interface_verifier;
