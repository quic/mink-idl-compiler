// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

mod ast;
mod parser;

macro_rules! valid {
    ($class: ident, [$($input: expr,)+]) => {{
        $(
            match $crate::pst::IDLParser::parse($crate::pst::Rule::$class, $input) {
                Ok(_) => {},
                Err(e) => panic!("Expected success, but expr: {:?} generated: {:?}", $input, e),
            }
        )+
    }};
}

macro_rules! invalid {
    ($class: ident, [$($input: expr,)+]) => {{
        $(
            match $crate::pst::IDLParser::parse($crate::pst::Rule::$class, $input) {
                Err(_) => {},
                Ok(o) => panic!("Expected failure, but expr: {:?} generated: {:?}", $input, o),
            }
        )+
    }};
}

pub(super) use {invalid, valid};
