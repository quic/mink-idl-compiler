// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

macro_rules! namespace {
    ($ident: literal) => {
        concat!("crate::object::", $ident)
    };
}
pub(super) const GENERIC_ERROR: &str = namespace!("error::generic");
pub(super) const ERROR_STRUCT: &str = namespace!("Error");
pub(super) const TYPED_OBJECT_TRAIT: &str = namespace!("TypedObject");
pub(super) const INVOKE_FN: &str = namespace!("Invoke");
pub(super) const PACK_COUNTS: &str = namespace!("pack_counts");
pub(super) const OK: i32 = 0;

pub(super) const CONTEXT: &str = namespace!("Ctx");
pub(super) const OP_ID: &str = namespace!("Op");
pub(super) const ARG: &str = namespace!("Arg");
pub(super) const COUNTS: &str = namespace!("Counts");

pub(super) const OP_RELEASE: &str = namespace!("OP_RELEASE");
pub(super) const OP_RETAIN: &str = namespace!("OP_RETAIN");

pub(super) const WRAPPER: &str = namespace!("wrapper");

pub const INTERFACES_BASE: &str = "crate::interfaces";
pub const OBJECT: &str = namespace!("Object");

pub(super) use namespace;
