// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

// Name of function which gets added to all auto-generated outputs
pub const VERSION_FUNC_NAME: &str = "api_version";

pub mod mir;
pub mod named_version;

pub use idlc_ast::pst::Error;
pub use mir::*;
pub use named_version::*;
