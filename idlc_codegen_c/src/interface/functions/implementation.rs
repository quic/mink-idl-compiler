use idlc_mir::Ident;

use crate::interface::variable_names::invoke::{ARGS, OBJECTBUF, OBJECTBUFIN, OP};

use crate::types::change_primitive;

use super::serialization::TransportBuffer;

#[derive(Debug, Clone, Default)]
pub struct Implementation {
    pub args: Vec<String>,
    pub initializations: Vec<String>,
    pub post_call: Vec<String>,

    idx: usize,
}

impl Implementation {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self {
            args: vec![],
            initializations: vec![],
            post_call: vec![],
            idx: 0,
        };

        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);

        me
    }

    pub fn args(&self) -> String {
        self.args.concat()
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
    pub fn idx(&mut self) -> usize {
        let idx = self.idx;
        self.idx += 1;

        idx
    }
}

impl idlc_codegen::functions::ParameterVisitor for Implementation {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);

        self.args.push(format!(
            r#"
        {{.bi = ({OBJECTBUFIN}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"#
        ));
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();

        self.args.push(format!(
            r#"
        {{.bi = ({OBJECTBUFIN}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"#
        ));
    }

    fn visit_input_object_array(&mut self, ident: &Ident, _ty: Option<&str>, cnt: idlc_mir::Count) {
        for i in 0..cnt.into() {
            let _idx = self.idx();
            self.args.push(format!(
                r#"
        {{.o = (*{ident}_ptr)[{i}] }},"#
            ));
        }
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let _idx = self.idx();
        let name = format!("{}_val", ident);
        let ty = change_primitive(ty);
        self.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{ &{name}, sizeof({ty}) }} }},"#
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
        let _idx = self.idx();

        self.initializations.push(format!(
            r#"{definition} i;
    {0}"#,
            packer.bi_assignments(),
        ));
        self.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{ &i, {size} }} }},"#
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.args.push(format!(
            r#"
        {{.bi = ({OBJECTBUFIN}) {{ {name}, sizeof({ty}) }} }},"#
        ));
    }

    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        let _idx = self.idx();
        let name = format!("{}_val", ident);
        self.args.push(format!(
            r#"
        {{.o = {name} }},"#
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);
        self.post_call.push(format!(
            r#"*{ident}_lenout = {ARGS}[{idx}].b.size / sizeof({ty});
    "#
        ));
        self.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"#
        ));
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.post_call.push(format!(
            r#"*{ident}_lenout = {ARGS}[{idx}].b.size / sizeof({ty});
    "#
        ));
        self.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"#
        ));
    }

    fn visit_output_object_array(
        &mut self,
        ident: &Ident,
        _ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        for i in 0..cnt.into() {
            let idx = self.idx();
            self.args.push(
                r#"
        {.o = (Object) { NULL, NULL } },"#
                    .to_string(),
            );
            self.post_call.push(format!(
                r#"(*{ident}_ptr)[{i}] = a[{idx}].o;
    "#,
            ));
        }
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);
        self.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{ {name}, sizeof({ty}) }} }},"#
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
        let _idx = self.idx();

        self.initializations.push(format!(
            r#"{definition} o;
    "#
        ));
        self.post_call.push(packer.post_bo_assignments());
        self.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{  &o, {size} }} }},"#
        ));
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{  {name}, sizeof({ty}) }} }},"#
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
        self.args.push(
            r#"
        {.o = (Object) { NULL, NULL } },"#
                .to_string(),
        );
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
    let mut object_args = String::new();
    // let mut returns = String::new();

    let params = super::signature::iter_to_string(signature.params());

    let implementation = Implementation::new(function);
    let initializations = implementation.initializations();
    let args = implementation.args();
    let post_call_assignments = implementation.post_call_assignments();

    if total > 0 {
        object_args = format!(
            r#"ObjectArg a[] = {{{args}
    }};
        "#
        );
    }

    let returns = if total > 0 {
        format!("Object_invoke(self, {iface_ident}_{OP}_{ident}, a, ObjectCounts_pack({0}, {1}, {2}, {3}));",
        counts.input_buffers,
        counts.output_buffers,
        counts.input_objects,
        counts.output_objects)
    } else {
        return format!(
            r#"
{documentation}
static inline int32_t {current_iface_ident}_{ident}(Object self{params})
{{
    return Object_invoke(self, {iface_ident}_{OP}_{ident}, 0, 0);;
}}
"#
        );
    };

    format!(
        r#"
{documentation}
static inline int32_t {current_iface_ident}_{ident}(Object self{params})
{{
    {0}{1}int32_t result = {returns}
    {2}
    return result;
}}
"#,
        if !initializations.is_empty() {
            format!("{initializations}\n    ")
        } else {
            "".to_string()
        },
        if !object_args.is_empty() {
            format!("{object_args}\n    ")
        } else {
            "".to_string()
        },
        if !post_call_assignments.is_empty() {
            format!("{post_call_assignments}    ")
        } else {
            "".to_string()
        }
    )
}
