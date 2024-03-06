use idlc_codegen_c::interface::variable_names::invoke::{ARGS, OBJECTBUF, OBJECTBUFIN, OP};
use idlc_mir::Ident;

use super::serialization::TransportBuffer;

#[derive(Debug, Clone, Default)]
pub struct Implementation(idlc_codegen_c::interface::functions::implementation::Implementation);

impl Implementation {
    pub fn new(function: &idlc_mir::Function) -> Self {
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

    fn visit_input_object_array(&mut self, ident: &Ident, _ty: Option<&str>, cnt: idlc_mir::Count) {
        for i in 0..cnt.into() {
            let _idx = self.0.idx();
            self.0.args.push(format!(
                r#"
        {{.o = ({ident}_ref)[{i}].get() }},"#
            ));
        }
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
        let _idx = self.0.idx();
        let bi_embedded = packer.bi_embedded();
        self.0.initializations.push(format!(
            r#"{definition} i;
    {0}{1}"#,
            bi_embedded,
            packer.bi_assignments(),
        ));
        self.0.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{ &i, {size} }} }},"#
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.0.idx();
        let name = format!("{}_ref", ident);
        let ty_ident = ty.ident.to_string();
        if ty.contains_interfaces() {
            self.0.initializations.push(format!(
                r#"{ty_ident} {ident}_cpy = {name};
    "#
            ));
            self.0.args.push(format!(
                r#"{{.bi = ({OBJECTBUFIN}) {{ &{ident}_cpy, sizeof({ty_ident}) }} }},
        "#
            ));

            let objects = ty.objects();
            for object in objects {
                let path = object
                    .0
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(".");
                self.0.pre_call.push(format!(
                    r#"{ident}_cpy.{path} = Object_NULL;
    "#
                ));
                self.visit_input_object(
                    &idlc_mir::Ident {
                        ident: format!("{ident}_cpy.{}", path),
                        span: object.0.last().unwrap().span,
                    },
                    object.1,
                );
            }
        } else {
            self.0.args.push(format!(
                r#"
        {{.bi = ({OBJECTBUFIN}) {{ &{name}, sizeof({ty_ident}) }} }},"#
            ));
        }
    }

    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        let _idx = self.0.idx();
        let name = format!("{}", ident);
        if name.contains('.') {
            self.0.args.push(format!(
                r#"
        {{.o = {name} }},"#
            ));
        } else {
            self.0.args.push(format!(
                r#"
        {{.o = {name}.get() }},"#
            ));
        }
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_output_primitive_buffer(ident, ty);
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_output_struct_buffer(ident, ty);
    }

    fn visit_output_object_array(
        &mut self,
        ident: &Ident,
        _ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let idx = self.0.idx();
        for _ in 0..cnt.into() {
            self.0.args.push(
                r#"
        {.o = (Object) { NULL, NULL } },"#
                    .to_string(),
            );
        }

        self.0.post_call.push(format!(
            r#"for(size_t arg_idx=0;arg_idx<{cnt};arg_idx++)
        ({ident}_ref)[arg_idx].consume(a[{idx}+arg_idx].o);
    "#,
        ));

        for _ in 1..cnt.into() {
            let _idx = self.0.idx();
        }
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
        let _idx = self.0.idx();

        self.0.initializations.push(format!(
            r#"{definition} o;
    "#
        ));
        self.0.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{  &o, {size} }} }},"#
        ));
        self.0.post_call.push(packer.post_bo_assignments());
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.0.idx();
        let name = format!("{}_ptr", ident);
        let ty_ident = ty.ident.to_string();
        self.0.args.push(format!(
            r#"
        {{.b = ({OBJECTBUF}) {{  {name}, sizeof({ty_ident}) }} }},"#
        ));

        let objects = ty.objects();
        for object in objects {
            let path = object
                .0
                .iter()
                .map(|ident| ident.to_string())
                .collect::<Vec<String>>()
                .join(".");
            self.0.initializations.push(format!(
                r#"{name}->{path} = Object_NULL;
    "#
            ));
            self.visit_output_object(
                &idlc_mir::Ident {
                    ident: format!("({name}->{})", path),
                    span: object.0.last().unwrap().span,
                },
                object.1,
            )
        }
    }

    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_object(&mut self, ident: &Ident, _: Option<&str>) {
        let idx = self.0.idx();
        self.0.args.push(
            r#"
        {.o = (Object) { NULL, NULL } },"#
                .to_string(),
        );
        if ident.contains('.') || ident.contains("->") {
            self.0.post_call.push(format!(
                r#"{ident} = {ARGS}[{idx}].o;
    "#
            ));
        } else {
            self.0.post_call.push(format!(
                r#"{ident}.consume({ARGS}[{idx}].o);
    "#
            ));
        }
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
    let mut object_args = String::new();

    let mut params =
        idlc_codegen_c::interface::functions::signature::iter_to_string(signature.params());
    if !params.is_empty() {
        params.remove(0);
    }

    let implementation = Implementation::new(function);
    let mut initializations = implementation.0.initializations();
    let mut args = implementation.0.args();
    let mut pre_call_assignments = implementation.0.pre_call_assignments();
    let mut post_call_assignments = implementation.0.post_call_assignments();
    initializations = initializations.replace('\n', "\n    ");
    args = args.replace('\n', "\n    ");
    pre_call_assignments = pre_call_assignments.replace('\n', "\n    ");
    post_call_assignments = post_call_assignments.replace('\n', "\n    ");

    if total > 0 {
        object_args = format!(
            r#"ObjectArg a[] = {{{args}
        }};
            "#
        );
    }

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
        {0}{1}{2}int32_t result = {returns}
        if (Object_OK != result) {{ return result; }}
        {post_call_assignments}
        return result;
    }}
    "#,
        if !initializations.is_empty() {
            format!("{initializations}\n        ")
        } else {
            "".to_string()
        },
        if !object_args.is_empty() {
            format!("{object_args}\n        ")
        } else {
            "".to_string()
        },
        if !pre_call_assignments.is_empty() {
            format!("{pre_call_assignments}\n        ")
        } else {
            "".to_string()
        },
    )
}
