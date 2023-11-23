use idlc_mir::{Function, Ident, ParamTypeIn, ParamTypeOut, Primitive, Struct, Type};

use crate::serialization::PackedPrimitives;

#[allow(unused)]
pub trait ParameterVisitor {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_input_primitive(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_input_bundled(&mut self, packed_primitives: &PackedPrimitives) {}
    fn visit_input_struct(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_input_object(&mut self, ident: &Ident, ty: Option<&str>) {}

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_output_primitive(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_output_bundled(&mut self, packed_primitives: &PackedPrimitives) {}
    fn visit_output_struct(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_output_object(&mut self, ident: &Ident, ty: Option<&str>) {}
}

#[derive(Debug)]
enum Param<'a> {
    InputBundledPrimitives(&'a PackedPrimitives),
    OutputBundledPrimitives(&'a PackedPrimitives),
    Params(&'a idlc_mir::Param),
}

impl<'a> Param<'a> {
    fn visit<V: ParameterVisitor>(params: impl Iterator<Item = Param<'a>>, visitor: &mut V) {
        for param in params.into_iter() {
            match param {
                Param::InputBundledPrimitives(b) => visitor.visit_input_bundled(b),
                Param::OutputBundledPrimitives(b) => visitor.visit_output_bundled(b),
                Param::Params(p) => match p {
                    idlc_mir::Param::In { r#type, ident } => match r#type {
                        ParamTypeIn::Array(t) => match t {
                            Type::Primitive(p) => visitor.visit_input_primitive_buffer(ident, p),
                            Type::Interface(_) => todo!(),
                            Type::Struct(s) => visitor.visit_input_struct_buffer(ident, s),
                        },
                        ParamTypeIn::Value(t) => match t {
                            Type::Primitive(p) => visitor.visit_input_primitive(ident, p),
                            Type::Interface(i) => visitor.visit_input_object(ident, i.as_deref()),
                            Type::Struct(s) => visitor.visit_input_struct(ident, s),
                        },
                    },
                    idlc_mir::Param::Out { r#type, ident } => match r#type {
                        ParamTypeOut::Array(t) => match t {
                            Type::Primitive(p) => visitor.visit_output_primitive_buffer(ident, p),
                            Type::Interface(_) => todo!(),
                            Type::Struct(s) => visitor.visit_output_struct_buffer(ident, s),
                        },
                        ParamTypeOut::Reference(t) => match t {
                            Type::Primitive(p) => visitor.visit_output_primitive(ident, p),
                            Type::Interface(i) => visitor.visit_output_object(ident, i.as_deref()),
                            Type::Struct(s) => visitor.visit_output_struct(ident, s),
                        },
                    },
                },
            }
        }
    }

    fn new(params: &'a [idlc_mir::Param], packed_primitives: &'a PackedPrimitives) -> Vec<Self> {
        let mut out = Vec::new();
        if packed_primitives.n_inputs() > 1 {
            out.push(Self::InputBundledPrimitives(packed_primitives));
        }
        out.extend(
            params
                .iter()
                .filter(|&x| {
                    !(packed_primitives.n_inputs() > 1 && x.is_input() && x.is_primitive_value())
                })
                .filter(|x| {
                    !(packed_primitives.n_outputs() > 1 && x.is_output() && x.is_primitive_value())
                })
                .map(Param::Params),
        );

        if packed_primitives.n_outputs() > 1 {
            let me = idlc_mir::Param::Out {
                r#type: idlc_mir::ParamTypeOut::Array(Type::Primitive(Primitive::Uint8)),
                ident: idlc_mir::Ident::new_without_span(String::new()),
            };
            let idx = out.iter().position(|x| {
                if let Param::Params(p) = *x {
                    p >= &me
                } else {
                    false
                }
            });

            out.insert(
                idx.unwrap_or(out.len()),
                Param::OutputBundledPrimitives(packed_primitives),
            );
        }

        out
    }
}

#[inline]
pub fn visit_params<V: ParameterVisitor>(function: &Function, visitor: &mut V) {
    Param::visit(function.params.iter().map(Param::Params), visitor)
}

#[inline]
pub fn visit_params_sorted<V: ParameterVisitor>(function: &Function, visitor: &mut V) {
    let mut params = function.params.clone();
    params.sort();
    let packed_primitives = super::serialization::PackedPrimitives::new(function);
    let with_bundling = Param::new(&params, &packed_primitives);
    Param::visit(with_bundling.into_iter(), visitor)
}
