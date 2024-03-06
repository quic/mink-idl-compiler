use std::borrow::Cow;

use idlc_codegen::serialization::Type;

use crate::interface::variable_names::invoke::{BI, BO};

use crate::types::change_primitive;

#[derive(Debug, Clone)]
pub struct TransportBuffer {
    pub definition: String,
    pub size: usize,
}

#[derive(Debug)]
pub struct PackedPrimitives<'a>(&'a idlc_codegen::serialization::PackedPrimitives);

impl<'a> PackedPrimitives<'a> {
    #[inline]
    pub const fn new(packer: &'a idlc_codegen::serialization::PackedPrimitives) -> Self {
        Self(packer)
    }

    pub fn bi_definition(&self, is_invoke: bool) -> Option<TransportBuffer> {
        Self::generate_struct(
            self.0.inputs_by_idents(),
            self.0.packed_input_size(),
            BI,
            is_invoke,
        )
    }

    pub fn bi_assignments(&self) -> String {
        let mut assignments = String::new();
        self.0.inputs_by_idents().for_each(|(ident, ty)| match ty {
            Type::Primitive(_) => {
                assignments += &format!(
                    r#"i.m_{ident} = {ident}_val;
    "#
                );
            }
            Type::SmallStruct(_s) => {
                assignments += &format!(
                    r#"i.m_{ident} = *{ident}_ptr;
    "#
                );
            }
        });

        assignments
    }

    pub fn bi_embedded(&self) -> String {
        let mut assignments = String::new();
        self.0.inputs_by_idents().for_each(|(ident, ty)| match ty {
            Type::Primitive(_) => {}
            Type::SmallStruct(s) => {
                if s.contains_interfaces() {
                    let ty = s.ident.to_string();

                    assignments += &format!(
                        r#"{ty} {ident}_cpy = *{ident}_ptr;
    "#
                    );
                }
            }
        });
        assignments
    }

    pub fn bo_definition(&self, is_invoke: bool) -> Option<TransportBuffer> {
        Self::generate_struct(
            self.0.outputs_by_idents(),
            self.0.packed_output_size(),
            BO,
            is_invoke,
        )
    }

    pub fn post_bo_assignments(&self) -> String {
        let mut assignments = String::new();
        self.0.outputs_by_idents().for_each(|(ident, ty)| match ty {
            Type::Primitive(_) => {
                assignments += &format!(
                    r"*{ident}_ptr = o.m_{ident};
    "
                );
            }
            Type::SmallStruct(_) => {
                assignments += &format!(
                    r"*{ident}_ptr = o.m_{ident};
    "
                );
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

    #[inline]
    fn generate_struct(
        pairs: impl Iterator<Item = (&'a idlc_mir::Ident, &'a Type)>,
        size: usize,
        in_out: &str,
        is_invoke: bool,
    ) -> Option<TransportBuffer> {
        if size == 0 {
            return None;
        }

        let mut fields = String::new();
        for (ident, ty) in pairs {
            let ty = match ty {
                &Type::Primitive(p) => Cow::Borrowed(change_primitive(p)),
                Type::SmallStruct(s) => Cow::Owned(s.ident.to_string()),
            };
            if is_invoke {
                fields.push_str(&format!(
                    r#"
                    {} m_{}; \"#,
                    ty, ident
                ));
            } else {
                fields.push_str(&format!(
                    r#"
        {} m_{};"#,
                    ty, ident
                ));
            }
        }

        let definition = if is_invoke {
            format!(
                r#"struct {in_out} {{ \{fields}
                }}"#
            )
        } else {
            format!(
                r#"struct {in_out} {{{fields}
    }}"#
            )
        };
        Some(TransportBuffer { definition, size })
    }
}
