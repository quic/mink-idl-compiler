use crate::{ident::EscapedIdent, types::change_primitive};

#[derive(Debug, Clone)]
pub struct TransportBuffer {
    pub definition: String,
    pub size: usize,
}

#[derive(Debug)]
pub struct PackedPrimitives(idlc_codegen::serialization::PackedPrimitives);

impl From<idlc_codegen::serialization::PackedPrimitives> for PackedPrimitives {
    fn from(value: idlc_codegen::serialization::PackedPrimitives) -> Self {
        Self(value)
    }
}

impl PackedPrimitives {
    #[inline]
    pub fn new(f: &idlc_mir::Function) -> Self {
        Self(idlc_codegen::serialization::PackedPrimitives::new(f))
    }

    pub fn bi_definition(&self) -> Option<TransportBuffer> {
        self.generate_struct(self.0.input_types(), "BI", self.0.packed_input_size())
    }

    pub fn bi_idents(&self) -> impl Iterator<Item = String> + '_ {
        self.0
            .inputs_by_idents()
            .map(|(ident, _)| EscapedIdent::new(ident).to_string())
    }

    pub fn bo_definition(&self) -> Option<TransportBuffer> {
        self.generate_struct(self.0.output_types(), "BO", self.0.packed_output_size())
    }

    pub fn bo_idents(&self) -> impl Iterator<Item = String> + '_ {
        self.0
            .outputs_by_idents()
            .map(|(ident, _)| EscapedIdent::new(ident).to_string())
    }

    #[inline]
    fn generate_struct(
        &self,
        types: impl Iterator<Item = idlc_mir::Primitive>,
        ident: &'static str,
        size: usize,
    ) -> Option<TransportBuffer> {
        if size == 0 {
            return None;
        }

        let fields = super::signature::iter_to_string(types.map(|ty| change_primitive(&ty)));
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
