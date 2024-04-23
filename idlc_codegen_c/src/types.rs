// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

pub const fn change_const_primitive(primitive: idlc_mir::Primitive) -> &'static str {
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

pub const fn change_primitive(primitive: idlc_mir::Primitive) -> &'static str {
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
