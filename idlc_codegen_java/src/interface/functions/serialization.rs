use idlc_codegen::serialization::Type;

use crate::interface::mink_primitives::{BUNDLE_IN, BUNDLE_OUT};

use crate::types::{capitalize_first_letter, change_primitive, get_struct_pair};

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
            BUNDLE_IN,
            self.0.packed_input_size(),
            is_invoke,
        )
    }

    pub fn pre_bo_assignments(&self) -> String {
        let mut assignments = String::new();
        self.0.outputs_by_idents().for_each(|(ident, ty)| match ty {
            &Type::Primitive(p) => {
                let ty = change_primitive(p);
                assignments += &format!(
                    r#"{ty}[] {ident} = new {ty}[1];
                    "#
                );
            }
            Type::SmallStruct(s) => {
                let ty = s.ident.to_string();
                assignments += &format!(
                    r#"{ty}[] {ident} = new {ty}[1];
                    "#
                );
            }
        });
        assignments
    }

    pub fn bo_definition(&self, is_invoke: bool) -> Option<TransportBuffer> {
        Self::generate_struct(
            self.0.outputs_by_idents(),
            BUNDLE_OUT,
            self.0.packed_output_size(),
            is_invoke,
        )
    }

    #[inline]
    fn generate_struct(
        pairs: impl Iterator<Item = (&'a idlc_mir::Ident, &'a Type)>,
        in_out: &'static str,
        size: usize,
        is_invoke: bool,
    ) -> Option<TransportBuffer> {
        if size == 0 {
            return None;
        }

        let mut fields = String::new();
        for (ident, ty) in pairs {
            match ty {
                &Type::Primitive(p) => {
                    let ty = change_primitive(p);
                    let capitalized_ty = capitalize_first_letter(ty);
                    if is_invoke {
                        if in_out == BUNDLE_IN {
                            fields.push_str(&format!(
                                r#"{ty} {ident}={in_out}.get{capitalized_ty}();
                    "#
                            ));
                        } else {
                            fields.push_str(&format!(
                                r#"{in_out}.put{capitalized_ty}({ident}[0]);
                    "#
                            ));
                        }
                    } else if in_out == BUNDLE_IN {
                        fields.push_str(&format!(
                            r#"{in_out}.put{capitalized_ty}({ident}_val);
                "#
                        ));
                    } else {
                        fields.push_str(&format!(
                            r#"if ({ident}_ptr != null) {{
                    {ident}_ptr[0] = {in_out}.get{capitalized_ty}();
                }}
                "#
                        ));
                    }
                }
                Type::SmallStruct(s) => {
                    let mut field_idents = Vec::new();
                    let ty = s.ident.to_string();
                    if is_invoke && in_out == BUNDLE_IN {
                        fields.push_str(&format!(
                            r#"{ty} {ident} = new {ty}();
                    "#,
                        ));
                    }
                    get_struct_pair(s, &mut field_idents, "".to_string());
                    for (field_ident, field_ty) in field_idents {
                        let mut capitalized_ty = String::new();
                        if let idlc_mir::Type::Primitive(p) = field_ty {
                            capitalized_ty = capitalize_first_letter(change_primitive(p));
                        }
                        if is_invoke {
                            if in_out == BUNDLE_IN {
                                fields.push_str(&format!(
                                    r#"{ident}.{field_ident}={BUNDLE_IN}.get{capitalized_ty}();
                    "#,
                                ));
                            } else {
                                fields.push_str(&format!(
                                    r#"{BUNDLE_OUT}.put{capitalized_ty}({ident}[0].{field_ident});
                    "#,
                                ));
                            }
                        } else if in_out == BUNDLE_IN {
                            fields.push_str(&format!(
                                r#"{in_out}.put{capitalized_ty}({ident}_val.{field_ident});
            "#,
                            ));
                        } else {
                            fields.push_str(&format!(
                                r#"if ({ident}_ptr != null) {{
                    {ident}_ptr[0].{field_ident} = {in_out}.get{capitalized_ty}();
                }}
                "#,
                            ));
                        }
                    }
                }
            };
        }
        let definition = fields;
        Some(TransportBuffer { definition, size })
    }
}
