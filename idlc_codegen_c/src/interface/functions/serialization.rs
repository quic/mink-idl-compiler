// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::borrow::Cow;

use idlc_codegen::serialization::Type;

use crate::interface::variable_names::invoke::{BI, BI_NAME, BO, BO_NAME, INDENT};

use crate::types::change_primitive;

#[derive(Debug, Clone)]
pub struct TransportBuffer {
    pub definition: Vec<String>,
    pub size: usize,
    pub initialization: String,
}

#[derive(Debug)]
pub struct PackedPrimitives<'a>(&'a idlc_codegen::serialization::PackedPrimitives);

impl<'a> PackedPrimitives<'a> {
    #[inline]
    pub const fn new(packer: &'a idlc_codegen::serialization::PackedPrimitives) -> Self {
        Self(packer)
    }

    pub fn bi_definition(&self) -> Option<TransportBuffer> {
        Self::generate_struct(self.0.inputs_by_idents(), self.0.packed_input_size(), BI)
    }

    pub fn bi_assignments(&self) -> Vec<String> {
        let mut assignments = vec![];
        self.0.inputs_by_idents().for_each(|(ident, ty)| match ty {
            Type::Primitive(_) => {
                assignments.push(format!("{BI_NAME}.m_{ident} = {ident}_val;"));
            }
            Type::SmallStruct(_s) => {
                assignments.push(format!("{BI_NAME}.m_{ident} = *{ident}_ptr;"));
            }
        });
        assignments
    }

    pub fn bi_embedded(&self) -> Vec<String> {
        let mut assignments = vec![];
        self.0.inputs_by_idents().for_each(|(ident, ty)| match ty {
            Type::Primitive(_) => {}
            Type::SmallStruct(s) => {
                if s.contains_interfaces() {
                    let ty = s.ident.to_string();
                    assignments.push(format!("{ty} {ident}_cpy = *{ident}_ptr;"));
                }
            }
        });
        assignments
    }

    pub fn bo_definition(&self) -> Option<TransportBuffer> {
        Self::generate_struct(self.0.outputs_by_idents(), self.0.packed_output_size(), BO)
    }

    pub fn post_bo_assignments(&self) -> Vec<String> {
        let mut assignments = vec![];
        self.0.outputs_by_idents().for_each(|(ident, ty)| match ty {
            Type::Primitive(_) => {
                assignments.push(format!("*{ident}_ptr = {BO_NAME}.m_{ident};"));
            }
            Type::SmallStruct(_) => {
                assignments.push(format!("*{ident}_ptr = {BO_NAME}.m_{ident};"));
            }
        });

        assignments
    }

    pub fn bo_embedded(&self) -> Vec<(idlc_mir::Ident, &idlc_mir::Ident, Option<&str>)> {
        let mut embedded_objs: Vec<(idlc_mir::Ident, &idlc_mir::Ident, Option<&str>)> = vec![];
        self.0.outputs_by_idents().for_each(|(ident, ty)| match ty {
            Type::Primitive(_) => {}
            Type::SmallStruct(s) => {
                if s.contains_interfaces() {
                    for field in &s.fields {
                        if let idlc_mir::Type::Interface(i) = &field.val.0 {
                            embedded_objs.push((ident.clone(), &field.ident, i.as_deref()));
                        }
                    }
                }
            }
        });
        embedded_objs
    }

    // Note this function is called recursively
    pub fn struct_init(s: &idlc_mir::StructInner, formatted: &mut String) {
        formatted.push('{');
        let mut raw_elements = vec![];
        for field in &s.fields {
            let (ty, size) = &field.val;
            let count = size.get();
            match ty {
                idlc_mir::Type::Primitive(_) => {
                    if count > 1 {
                        raw_elements.push("{0}".to_string());
                    } else {
                        raw_elements.push("0".to_string());
                    }
                }
                idlc_mir::Type::Struct(s) => {
                    let mut inner_struct = String::new();
                    if count > 1 {
                        inner_struct.push('{');
                    }
                    Self::struct_init(s.as_ref(), &mut inner_struct);
                    if count > 1 {
                        inner_struct.push('}');
                    }
                    raw_elements.push(inner_struct);
                }
                idlc_mir::Type::Interface(_) => raw_elements.push("Object_NULL".to_string()),
                _ => unreachable!(),
            }
        }
        formatted.push_str(&raw_elements.join(", "));
        formatted.push('}');
    }

    #[inline]
    fn generate_struct(
        pairs: impl Iterator<Item = (&'a idlc_mir::Ident, &'a Type)>,
        size: usize,
        in_out: &str,
    ) -> Option<TransportBuffer> {
        if size == 0 {
            return None;
        }

        let mut definition = vec![format!("struct {in_out} {{")];
        let mut init: Vec<String> = vec![];
        for (ident, ty) in pairs {
            let ty = match ty {
                &Type::Primitive(p) => {
                    init.push("0".to_string());
                    Cow::Borrowed(change_primitive(p))
                }
                Type::SmallStruct(s) => {
                    let mut inner_struct = String::new();
                    Self::struct_init(s, &mut inner_struct);
                    init.push(inner_struct);
                    Cow::Owned(s.ident.to_string())
                }
            };
            definition.push(format!("{INDENT}{} m_{};", ty, ident));
        }
        definition.push("}".to_string());
        let initialization = init.join(", ");
        Some(TransportBuffer {
            definition,
            size,
            initialization,
        })
    }
}
