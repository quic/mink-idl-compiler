// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

mod generator;
pub mod globals;
pub mod interface;
pub mod types;

pub use generator::Generator;

pub(crate) fn safe_ident_c(ident: &str) -> std::borrow::Cow<'_, str> {
    if idlc_codegen::keywords::is_reserved_for_c(ident) {
        idlc_errors::warn!(
            "Identifier `{ident}` is a reserved C keyword; renamed to `_{ident}` to avoid compilation issues"
        );
        std::borrow::Cow::Owned(format!("_{ident}"))
    } else {
        std::borrow::Cow::Borrowed(ident)
    }
}
