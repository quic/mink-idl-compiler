// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashMap;

use idlc_codegen::Descriptor;
use idlc_mir::Node;

use idlc_codegen::MINKIDL_HEADER_COMMENT;

use crate::{
    globals::{emit_const, emit_struct},
    interface::emit_interface,
};

pub struct Generator;

impl idlc_codegen::Generator for Generator {
    fn generate(mir: &idlc_mir::Mir) -> Descriptor {
        let mut base = std::path::PathBuf::from(mir.tag.file_name().unwrap().to_str().unwrap());
        base.set_extension("java");
        let mut interfaces = HashMap::new();
        let prologue = &format!(
            r#"// {MINKIDL_HEADER_COMMENT}
package com.qualcomm.qti.mink;
"#
        );
        interfaces.insert(base.clone(), prologue.to_owned());

        let input_name = &mir.tag.file_stem().unwrap().to_str().unwrap();
        let mut includes = String::new();
        for node in &mir.nodes {
            if let Node::Include(i) = node.as_ref() {
                let inc_name = i.display().to_string().replace(".idl", "");
                includes.push_str(&format!("{inc_name},"));
            }
        }
        if !includes.is_empty() {
            includes.pop();
            includes = format!("extends {includes}");
        }
        interfaces.get_mut(&base).unwrap().push_str(&format!(
            r#"public interface {input_name} {includes} {{
    "#
        ));

        for node in &mir.nodes {
            match node.as_ref() {
                Node::Const(c) => {
                    interfaces.get_mut(&base).unwrap().push_str(&emit_const(c));
                }
                Node::Struct(s) => {
                    interfaces
                        .get_mut(&base)
                        .unwrap()
                        .push_str(&emit_struct(s.as_ref()));
                }
                Node::Interface(i) => {
                    let mut interface_content = String::new();

                    interface_content
                        .push_str("import com.qualcomm.qti.qms.api.mink.IMinkObject;\n");
                    interface_content
                        .push_str("import com.qualcomm.qti.qms.api.mink.JMinkObject;\n");
                    interface_content.push_str("import com.qualcomm.qti.qms.api.mink.MinkProxy;\n");
                    interface_content.push_str("import java.nio.ByteBuffer;\n");
                    interface_content.push_str("import java.nio.ByteOrder;\n\n");
                    interface_content.push_str(&emit_interface(i, input_name));
                    let name = format!("{}.java", i.ident);
                    if name == base.to_str().unwrap() {
                        interfaces
                            .get_mut(&base)
                            .unwrap()
                            .push_str(&interface_content);
                    } else {
                        interfaces.insert(name.into(), format!("{prologue}{interface_content}"));
                    }
                }
                _ => (),
            }
        }
        interfaces.get_mut(&base).unwrap().push_str(
            r#"
}"#,
        );

        interfaces
            .into_iter()
            .filter(|(_, content)| content != prologue)
            .collect()
    }
}
