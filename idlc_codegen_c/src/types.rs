pub const fn change_const_primitive(primitive: &idlc_mir::Primitive) -> &'static str {
    match primitive {
        idlc_mir::Primitive::Uint8 => "UINT8_C",
        idlc_mir::Primitive::Uint16 => "UINT16_C",
        idlc_mir::Primitive::Uint32 => "UINT32_C",
        idlc_mir::Primitive::Uint64 => "UINT64_C",
        idlc_mir::Primitive::Int8 => "INT8_C",
        idlc_mir::Primitive::Int16 => "INT16_C",
        idlc_mir::Primitive::Int32 => "INT32_C",
        idlc_mir::Primitive::Int64 => "INT64_C",
        idlc_mir::Primitive::Float32 => "FLOAT",
        idlc_mir::Primitive::Float64 => "DOUBLE",
    }
}

pub const fn change_primitive(primitive: &idlc_mir::Primitive) -> &'static str {
    match primitive {
        idlc_mir::Primitive::Uint8 => "uint8_t",
        idlc_mir::Primitive::Uint16 => "uint16_t",
        idlc_mir::Primitive::Uint32 => "uint32_t",
        idlc_mir::Primitive::Uint64 => "uint64_t",
        idlc_mir::Primitive::Int8 => "int8_t",
        idlc_mir::Primitive::Int16 => "int16_t",
        idlc_mir::Primitive::Int32 => "int32_t",
        idlc_mir::Primitive::Int64 => "int64_t",
        idlc_mir::Primitive::Float32 => "float",
        idlc_mir::Primitive::Float64 => "double",
    }
}

// get the string type of type name
// pub fn change_type(r#type: &idlc_mir::Type, is_cpp: bool) -> &'static str {
//     match r#type {
//         idlc_mir::Type::Primitive(primitive) => change_primitive(primitive),
//         idlc_mir::Type::Struct(custom) => &custom.r#struct.ident,
//         idlc_mir::Type::Interface(iface) => {
//             if is_cpp {
//                 iface
//             } else {
//                 "Object"
//             }
//         }
//     }
// }
