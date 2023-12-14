use idlc_codegen_c::interface::variable_names::invoke::{ARGS, OBJECTBUF, OBJECTBUFIN, OP};
use idlc_mir::Ident;

use super::serialization::TransportBuffer;

#[derive(Debug, Clone, Default)]
pub struct Implementation(idlc_codegen_c::interface::functions::implementation::Implementation);

impl Implementation {
    pub fn new(function: &idlc_mir::Function, _counts: &idlc_codegen::counts::Counter) -> Self {
        let mut me = Self::default();

        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);

        me
    }
}

impl idlc_codegen::functions::ParameterVisitor for Implementation {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_input_primitive_buffer(ident, ty);
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_input_struct_buffer(ident, ty);
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_input_primitive(ident, ty);
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bi_definition(false) else {
            unreachable!()
        };
        let idx = self.0.idx();

        self.0.initializations.push(format!(
            r#"{definition} i;
    {0}"#,
            packer.bi_assignments(),
        ));
        self.0.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ &i, {size} }};
    "#
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.0.idx();
        let name = format!("&{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.0.initializations.push(format!(
            r#"{ARGS}[{idx}].bi = ({OBJECTBUFIN}) {{ {name}, sizeof({ty}) }};
    "#
        ));
    }

    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        let idx = self.0.idx();
        let name = format!("{}_val", ident);
        self.0.initializations.push(format!(
            r#"{ARGS}[{idx}].o = {name}.get();
    "#
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_output_primitive_buffer(ident, ty);
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_output_struct_buffer(ident, ty);
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_output_primitive(ident, ty);
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bo_definition(false) else {
            unreachable!()
        };
        let idx = self.0.idx();

        self.0.initializations.push(format!(
            r#"{definition} o;
    "#
        ));
        self.0.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ &o, {size} }};
    "#
        ));
        self.0.post_call.push(packer.post_bo_assignments());
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.0.idx();
        let name = format!("&{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.0.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ {name}, sizeof({ty}) }};
    "#
        ));
    }

    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_object(&mut self, ident: &Ident, _: Option<&str>) {
        let idx = self.0.idx();
        let name = format!("{}_val", ident);
        self.0.post_call.push(format!(
            r#"{name}.consume({ARGS}[{idx}].o);
    "#
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    counts: &idlc_codegen::counts::Counter,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;
    let total = counts.total();

    let mut params =
        idlc_codegen_c::interface::functions::signature::iter_to_string(signature.params());
    if !params.is_empty() {
        params.remove(0);
    }

    let implementation = Implementation::new(function, counts);
    let mut initializations = implementation.0.initializations();
    let mut post_call_assignments = implementation.0.post_call_assignments();
    initializations = initializations.replace('\n', "\n    ");
    post_call_assignments = post_call_assignments.replace('\n', "\n    ");

    let returns = if total > 0 {
        format!(
            "invoke({OP}_{ident}, a, ObjectCounts_pack({0}, {1}, {2}, {3}));",
            counts.input_buffers,
            counts.output_buffers,
            counts.input_objects,
            counts.output_objects
        )
    } else {
        format!("invoke({OP}_{ident}, 0, 0);")
    };

    format!(
        r#"
    {documentation}
    virtual int32_t {ident}({params}) {{
        ObjectArg {ARGS}[{total}]={{{{{{0,0}}}}}};
        {initializations}
        int32_t result = {returns}
        {post_call_assignments}

        return result;
    }}
    "#
    )
}
