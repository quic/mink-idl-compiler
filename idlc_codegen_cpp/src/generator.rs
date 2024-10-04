// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_codegen_c::globals::emit_struct;
use idlc_codegen_c::types::{change_const_primitive, change_primitive};
use idlc_mir::Node;

use crate::interface::{emit_interface_impl, emit_interface_invoke};

use idlc_codegen::MINKIDL_HEADER_COMMENT;
pub struct Generator;

impl idlc_codegen::SplitInvokeGenerator for Generator {
    fn generate_implementation(&self, mir: &idlc_mir::Mir) -> String {
        let mut result = String::new();
        result.push_str(&generate_common());

        for node in &mir.nodes {
            match node.as_ref() {
                Node::Include(i) => {
                    let inc_name = i.display().to_string().replace(".idl", "");
                    result.push_str(&format!("#include \"{}.hpp\"\n", inc_name));
                }
                Node::Const(c) => {
                    let ident = c.ident.to_string();
                    let cnt_ty = change_primitive(c.r#type);
                    let ty = change_const_primitive(c.r#type);
                    let value = &c.value;

                    result.push_str(&format!(
                        "static const {cnt_ty} {ident} = {ty}({value});\n\n"
                    ));
                }
                Node::Struct(s) => {
                    result.push_str(&emit_struct(s.as_ref()));
                }
                Node::Interface(i) => {
                    result.push_str(&emit_interface_impl(i));
                }
            }
        }

        result
    }

    fn generate_invoke(&self, mir: &idlc_mir::Mir) -> String {
        let mut result = generate_common();

        let input_name = &mir.tag.file_stem().unwrap().to_str().unwrap();
        result.push_str(&format!(
            r#"#include "{input_name}.hpp"
#include "impl_base.hpp"
"#
        ));

        for node in &mir.nodes {
            match node.as_ref() {
                Node::Include(i) => {
                    let inc_name = i.display().to_string().replace(".idl", "");
                    result.push_str(&format!("#include \"{}.hpp\"\n", inc_name));
                }
                Node::Interface(i) => {
                    result.push_str(&emit_interface_invoke(i));
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

#include <cstdint>
#include <stdint.h>
#include "object.h"
#include "proxy_base.hpp"
"#
    )
    .to_string()
}
