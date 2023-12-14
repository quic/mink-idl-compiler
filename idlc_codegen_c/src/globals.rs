use idlc_mir::{Const, StructInner};

use crate::types::{change_const_primitive, change_primitive};

pub fn emit_include(include: &std::path::Path) -> String {
    let inc_name = include.display().to_string().replace(".idl", "");
    format!("#include \"{}.h\"\n", inc_name)
}

pub fn emit_struct(r#struct: &StructInner) -> String {
    let mut result = String::new();
    result.push_str("typedef struct {\n");

    for field in &r#struct.fields {
        let ident = &field.ident;
        let count = field.val.1.get();
        let ty = match &field.val.0 {
            &idlc_mir::Type::Primitive(primitive) => change_primitive(primitive).to_string(),
            idlc_mir::Type::Struct(s) => s.as_ref().ident.to_string(),
            idlc_mir::Type::Interface(_) => "Object".to_string(),
        };
        result.push_str(&if count == 1 {
            format!("  {ty} {ident};\n")
        } else {
            format!("  {ty} {ident}[{count}];\n")
        });
    }
    result.push_str(&format!("}} {};\n\n", r#struct.ident));
    result
}

pub fn emit_const(r#const: &Const) -> String {
    let ident = r#const.ident.to_uppercase();
    let ty = change_const_primitive(r#const.r#type);
    let value = &r#const.value;

    format!("#define {ident} {ty}({value})\n\n")
}
