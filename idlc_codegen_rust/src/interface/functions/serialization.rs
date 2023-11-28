use std::borrow::Cow;

use idlc_codegen::serialization::Type;

use crate::{
    ident::EscapedIdent,
    types::{change_primitive, namespaced_struct},
};

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

    pub fn bi_definition(&self) -> Option<TransportBuffer> {
        self.generate_struct(self.0.input_types(), "BI", self.0.packed_input_size())
    }

    pub fn bi_definition_idents(&self) -> impl ExactSizeIterator<Item = String> + '_ {
        self.0
            .inputs_by_idents()
            .map(|(ident, _)| EscapedIdent::new(ident).to_string())
    }

    pub fn bi_assignment_idents(&self) -> impl ExactSizeIterator<Item = String> + '_ {
        self.0.inputs_by_idents().map(|(ident, ty)| {
            let ident = EscapedIdent::new(ident);
            match ty {
                Type::Primitive(_) => ident.to_string(),
                Type::SmallStruct(_) => format!("*{ident}"),
            }
        })
    }

    pub fn post_bi_assignments(&self) -> String {
        let mut assignments = String::new();
        self.0.inputs_by_idents().for_each(|(ident, ty)| {
            if matches!(ty, Type::SmallStruct(_)) {
                let ident = EscapedIdent::new(ident);
                assignments += &format!(r"let {ident} = &{ident};");
            }
        });

        assignments
    }

    pub fn bo_definition(&self) -> Option<TransportBuffer> {
        self.generate_struct(self.0.output_types(), "BO", self.0.packed_output_size())
    }

    pub fn bo_idents(&self) -> impl ExactSizeIterator<Item = String> + '_ {
        self.0
            .outputs_by_idents()
            .map(|(ident, _)| EscapedIdent::new(ident).to_string())
    }

    #[inline]
    fn generate_struct(
        &self,
        types: impl Iterator<Item = &'a Type>,
        ident: &'static str,
        size: usize,
    ) -> Option<TransportBuffer> {
        if size == 0 {
            return None;
        }

        let fields = super::signature::iter_to_string(types.map(|ty| match ty {
            Type::Primitive(p) => Cow::Borrowed(change_primitive(p)),
            Type::SmallStruct(s) => Cow::Owned(namespaced_struct(s)),
        }));
        let definition = format!(
            r#"
        #[repr(C, packed)]
        #[derive(Clone, Copy)]
        struct {ident}({fields});
        "#
        );
        Some(TransportBuffer { definition, size })
    }
}
