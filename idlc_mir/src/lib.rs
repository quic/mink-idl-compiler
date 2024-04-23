// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

pub mod mir;

pub use idlc_ast::pst::Error;
pub use mir::*;

pub fn dump(mir: Mir) {
    println!("{mir:#?}");
}
