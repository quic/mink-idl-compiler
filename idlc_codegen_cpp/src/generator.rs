use idlc_codegen_c::globals::{emit_const, emit_struct};
use idlc_mir::Node;

use crate::interface::{emit_interface_impl, emit_interface_invoke};

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
                    result.push_str(&emit_const(c));
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
    r#"#pragma once
// AUTOGENERATED FILE: DO NOT EDIT

#include <cstdint>
#include <stdint.h>
#include "object.h"
#include "proxy_base.hpp"


"#
    .to_string()
}
