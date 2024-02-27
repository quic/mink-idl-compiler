use convert_case::Casing;

pub const fn change_primitive(primitive: idlc_mir::Primitive) -> &'static str {
    match primitive {
        idlc_mir::Primitive::Uint8 | idlc_mir::Primitive::Int8 => "byte",
        idlc_mir::Primitive::Uint16 => "char",
        idlc_mir::Primitive::Int16 => "char", // need to be changed to 'short' in the future for the java size correctness, but leave it for now for backward compatibility.
        idlc_mir::Primitive::Uint32 | idlc_mir::Primitive::Int32 => "int",
        idlc_mir::Primitive::Uint64 | idlc_mir::Primitive::Int64 => "long",
        idlc_mir::Primitive::Float32 => "float",
        idlc_mir::Primitive::Float64 => "double",
    }
}

pub fn capitalize_first_letter(s: &str) -> String {
    if s == "byte" {
        return "".to_string();
    }
    s.to_case(convert_case::Case::UpperCamel)
}

pub fn get_struct_pair(
    r#struct: &idlc_mir::StructInner,
    result: &mut Vec<(String, idlc_mir::Type)>,
    mut parent: String,
) {
    if !parent.is_empty() {
        parent.push('.');
    }
    for field in &r#struct.fields {
        match &field.val.0 {
            idlc_mir::Type::Primitive(_) => {
                result.push((format!("{parent}{}", field.ident), field.val.0.clone()))
            }
            idlc_mir::Type::Struct(idlc_mir::Struct::Big(s) | idlc_mir::Struct::Small(s)) => {
                let new_parent = format!("{parent}{}", field.ident);
                get_struct_pair(s, result, new_parent);
            }
            idlc_mir::Type::Interface(_) => {
                result.push((format!("{parent}{}", field.ident), field.val.0.clone()))
            }
            _ => unreachable!(),
        }
    }
}
