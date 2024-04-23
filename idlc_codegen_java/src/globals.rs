// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::{Const, StructInner};

use crate::types::change_primitive;

pub fn emit_struct(r#struct: &StructInner) -> String {
    let mut contents = String::new();
    let struct_ident = &r#struct.ident;

    for field in &r#struct.fields {
        let ident = &field.ident;
        let count = field.val.1.get();
        let ty = match &field.val.0 {
            &idlc_mir::Type::Primitive(primitive) => change_primitive(primitive).to_string(),
            idlc_mir::Type::Struct(s) => s.as_ref().ident.to_string(),
            idlc_mir::Type::Interface(_) => {
                unimplemented!("Java codegen doesn't support objects in struct")
            }
            _ => unreachable!(),
        };
        contents.push_str(&if count == 1 {
            format!(
                r#"public {ty} {ident};
        "#
            )
        } else {
            format!(
                r#"public {ty}[] {ident}=new {ty}[{count}];
        "#
            )
        });
    }

    format!(
        r#"
    class {struct_ident} {{
        {contents}
    }}
    "#
    )
}

pub fn emit_const(r#const: &Const) -> String {
    let ident = r#const.ident.to_string(); // Const ident should be uppercase, but leave it for now for backward compatibility.
    let ty = change_primitive(r#const.r#type);
    let value = &r#const.value;

    format!(
        r#"{ty} {ident} = {value};
    "#
    )
}
