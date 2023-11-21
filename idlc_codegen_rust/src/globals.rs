use idlc_mir::{Const, Struct};

use crate::types::change_primitive;

pub fn emit_struct(r#struct: &Struct) -> String {
    let mut result = String::new();
    result.push_str("#[repr(C)]\n");
    result.push_str("#[derive(Debug, Clone, Copy, PartialEq, Hash)]\n");
    result.push_str(&format!("pub struct {} {{\n", r#struct.ident));

    for field in &r#struct.fields {
        let ident = &field.ident;
        let count = field.val.1.get();
        let ty = match &field.val.0 {
            idlc_mir::Type::Primitive(primitive) => change_primitive(primitive).to_string(),
            idlc_mir::Type::Struct(s) => crate::types::namespaced_struct(s),
            idlc_mir::Type::Interface(_) => {
                unimplemented!("Rust codegen doesn't support objects in struct")
            }
        };
        result.push_str(&if count == 1 {
            format!("pub {ident}: {ty},")
        } else {
            format!("pub {ident}: [{ty}; {count}],")
        });
    }
    result.push('}');
    result
}

pub fn emit_const(r#const: &Const) -> String {
    let ident = r#const.ident.to_uppercase();
    let ty = change_primitive(&r#const.r#type);
    let value = &r#const.value;

    format!("pub const {ident}: {ty} = {value};")
}
