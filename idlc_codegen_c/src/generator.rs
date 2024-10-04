// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::Node;

use idlc_codegen::MINKIDL_HEADER_COMMENT;

use crate::{
    globals::{emit_const, emit_include, emit_struct},
    interface::{emit_interface_impl, emit_interface_invoke},
};

pub struct Generator {
    is_no_typed_objects: bool,
}

impl Generator {
    pub fn new(is_no_typed_objects: bool) -> Self {
        Self {
            is_no_typed_objects,
        }
    }
}

impl idlc_codegen::SplitInvokeGenerator for Generator {
    fn generate_implementation(&self, mir: &idlc_mir::Mir) -> String {
        let mut result = String::new();
        result.push_str(&generate_common());

        for node in &mir.nodes {
            match node.as_ref() {
                Node::Include(i) => {
                    result.push_str(&emit_include(i));
                }
                Node::Const(c) => {
                    result.push_str(&emit_const(c));
                }
                Node::Struct(s) => {
                    result.push_str(&emit_struct(s.as_ref()));
                }
                Node::Interface(i) => {
                    result.push_str(&emit_interface_impl(i, self.is_no_typed_objects));
                }
            }
        }

        result
    }

    fn generate_invoke(&self, mir: &idlc_mir::Mir) -> String {
        let mut result = generate_common();

        let input_name = &mir.tag.file_stem().unwrap().to_str().unwrap();
        result.push_str(&format!("#include \"{}.h\"\n", input_name));

        for node in &mir.nodes {
            match node.as_ref() {
                Node::Include(i) => {
                    result.push_str(&emit_include(i));
                }
                Node::Interface(i) => {
                    result.push_str(&emit_interface_invoke(i, self.is_no_typed_objects));
                }
                _ => (),
            }
        }

        result
    }
}

fn generate_common() -> String {
    format!(
        r#"// {MINKIDL_HEADER_COMMENT}
#pragma once

#include <stdint.h>
#include "object.h"
"#
    )
    .to_string()
}
