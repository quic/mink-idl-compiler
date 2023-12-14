pub const fn change_primitive(primitive: idlc_mir::Primitive) -> &'static str {
    match primitive {
        idlc_mir::Primitive::Uint8 => "u8",
        idlc_mir::Primitive::Uint16 => "u16",
        idlc_mir::Primitive::Uint32 => "u32",
        idlc_mir::Primitive::Uint64 => "u64",
        idlc_mir::Primitive::Int8 => "i8",
        idlc_mir::Primitive::Int16 => "i16",
        idlc_mir::Primitive::Int32 => "i32",
        idlc_mir::Primitive::Int64 => "i64",
        idlc_mir::Primitive::Float32 => "f32",
        idlc_mir::Primitive::Float64 => "f64",
    }
}

pub fn namespaced_struct(r#struct: &idlc_mir::StructInner) -> String {
    use crate::interface::mink_primitives::INTERFACES_BASE;
    let namespace = r#struct
        .origin
        .as_ref()
        .map(|origin| {
            let ident = origin
                .file_stem()
                .expect("Expected IDL file")
                .to_str()
                .unwrap()
                .to_lowercase();
            format!("{INTERFACES_BASE}::{ident}::")
        })
        .unwrap_or_default();
    let ident = super::ident::EscapedIdent::new(&r#struct.ident);
    format!("{namespace}{ident}")
}
