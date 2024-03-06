use idlc_mir::{Const, StructInner};

use crate::types::change_primitive;

pub fn emit_struct(r#struct: &StructInner) -> String {
    let mut inner = String::new();
    let derives = ["Debug", "Clone", "PartialEq", "Copy"];
    let mut contains_interface = false;

    for field in &r#struct.fields {
        let ident = &field.ident;
        let count = field.val.1.get();
        let ty = match &field.val.0 {
            &idlc_mir::Type::Primitive(primitive) => change_primitive(primitive).to_string(),
            idlc_mir::Type::Struct(s) => {
                contains_interface |= s.as_ref().contains_interfaces();
                crate::types::namespaced_struct(s.as_ref())
            }
            idlc_mir::Type::Interface(ty) => {
                use crate::interface::mink_primitives::{INTERFACES_BASE, OBJECT};
                use std::borrow::Cow;
                contains_interface = true;

                let ty = ty.as_ref().map_or(Cow::Borrowed(OBJECT), |ty| {
                    Cow::Owned(format!("{INTERFACES_BASE}::{}::{ty}", ty.to_lowercase()))
                });

                format!("Option<{ty}>")
            }
            _ => unreachable!(),
        };

        inner.push_str(&if count == 1 {
            format!("pub {ident}: {ty},\n")
        } else {
            format!("pub {ident}: [{ty}; {count}],\n")
        });
    }
    format!(
        r#"
#[repr(C)]
#[derive({derives})]
pub struct {ident} {{
    {inner}
}}
"#,
        derives = derives[..derives.len() - contains_interface as usize].join(","),
        ident = r#struct.ident,
    )
}

pub fn emit_const(r#const: &Const) -> String {
    let ident = r#const.ident.to_uppercase();
    let ty = change_primitive(r#const.r#type);
    let value = &r#const.value;

    format!("pub const {ident}: {ty} = {value};\n")
}
