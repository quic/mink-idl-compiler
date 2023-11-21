use idlc_mir::{Function, Ident, Param, ParamTypeIn, ParamTypeOut, Primitive, Struct, Type};

#[allow(unused)]
pub trait ParameterVisitor {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_input_primitive(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_input_struct(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_input_object(&mut self, ident: &Ident, ty: Option<&str>) {}

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_output_primitive(&mut self, ident: &Ident, ty: &Primitive) {}
    fn visit_output_struct(&mut self, ident: &Ident, ty: &Struct) {}
    fn visit_output_object(&mut self, ident: &Ident, ty: Option<&str>) {}
}

pub fn visit_params<V: ParameterVisitor>(function: &Function, visitor: &mut V) {
    for params in &function.params {
        match params {
            Param::In { r#type, ident } => match r#type {
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
            Param::Out { r#type, ident } => match r#type {
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
        }
    }
}
