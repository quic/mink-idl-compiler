use idlc_mir::Ident;

use crate::interface::variable_names::invoke::{ARGS, OBJECTBUF, OBJECTBUFIN, OP};

use crate::types::change_primitive;

use super::serialization::TransportBuffer;

#[derive(Debug, Clone, Default)]
pub struct Implementation {
    initializations: Vec<String>,
    post_call: Vec<String>,

    pub obj_arr_in: String,
    pub obj_arr_out: String,
    pub obj_arr_num: u32,

    idx: usize,
}

impl Implementation {
    pub fn new(function: &idlc_mir::Function, counts: &idlc_codegen::counts::Counter) -> Self {
        let mut me = Self {
            initializations: vec![],
            post_call: vec![],
            obj_arr_in: String::new(),
            obj_arr_out: String::new(),
            obj_arr_num: counts.total(),
            idx: 0,
        };

        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);

        me
    }

    pub fn initializations(&self) -> String {
        self.initializations.concat()
    }

    pub fn post_call_assignments(&self) -> String {
        self.post_call.concat()
    }
}

impl Implementation {
    #[inline]
    fn idx(&mut self) -> usize {
        let idx = self.idx;
        self.idx += 1;

        idx
    }
}

impl idlc_codegen::functions::ParameterVisitor for Implementation {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].bi = ({OBJECTBUFIN}) {{ {name}, {ident}_len * sizeof({ty}) }};
    "#
        ));
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].bi = ({OBJECTBUFIN}) {{ {name}, {ident}_len * sizeof({ty}) }};
    "#
        ));
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let name = format!("{}_val", ident);
        let ty = change_primitive(ty);
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ &{name}, sizeof({ty}) }};
    "#
        ));
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bi_definition(false) else {
            unreachable!()
        };
        let idx = self.idx();

        self.initializations.push(format!(
            r#"{definition} i = {{0}};
    {0}"#,
            packer.bi_assignments(),
        ));
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ &i, {size} }};
    "#
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].bi = ({OBJECTBUFIN}) {{ {name}, sizeof({ty}) }};
    "#
        ));
    }

    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        let idx = self.idx();
        let name = format!("{}_val", ident);
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].o = {name};
    "#
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ {name}, {ident}_len * sizeof({ty}) }};
    "#
        ));
        self.post_call.push(format!(
            r#"*{ident}_lenout = {ARGS}[{idx}].b.size / sizeof({ty});
    "#
        ));
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ {name}, {ident}_len * sizeof({ty}) }};
    "#
        ));
        self.post_call.push(format!(
            r#"*{ident}_lenout = {ARGS}[{idx}].b.size / sizeof({ty});
    "#
        ));
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ {name}, sizeof({ty}) }};
    "#
        ));
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bo_definition(false) else {
            unreachable!()
        };
        let idx = self.idx();

        self.initializations.push(format!(
            r#"{definition} o = {{0}};
    "#
        ));
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ &o, {size} }};
    "#
        ));
        self.post_call.push(packer.post_bo_assignments());
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.initializations.push(format!(
            r#"{ARGS}[{idx}].b = ({OBJECTBUF}) {{ {name}, sizeof({ty}) }};
    "#
        ));
    }

    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_object(&mut self, ident: &Ident, _: Option<&str>) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        self.post_call.push(format!(
            r#"*{name} = {ARGS}[{idx}].o;
    "#
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    current_iface_ident: &str,
    iface_ident: &str,
    documentation: &str,
    counts: &idlc_codegen::counts::Counter,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;
    let total = counts.total();

    let params = super::signature::iter_to_string(signature.params());

    let implementation = Implementation::new(function, counts);
    let initializations = implementation.initializations();
    let post_call_assignments = implementation.post_call_assignments();

    let returns = if total > 0 {
        format!("Object_invoke(self, {iface_ident}_{OP}_{ident}, a, ObjectCounts_pack({0}, {1}, {2}, {3}));",
            counts.input_buffers,
            counts.output_buffers,
            counts.input_objects,
            counts.output_objects)
    } else {
        format!("Object_invoke(self, {iface_ident}_{OP}_{ident}, 0, 0);")
    };

    format!(
        r#"
static inline int32_t
{documentation}
{current_iface_ident}_{ident}(Object self{params})
{{
    ObjectArg {ARGS}[{total}]={{{{{{0,0}}}}}};
    {initializations}
    int32_t result = {returns}
    {post_call_assignments}

    return result;
}}
"#
    )
}
