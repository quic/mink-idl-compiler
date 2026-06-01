// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_codegen_c::interface::variable_names::invoke::{ARGS, COUNTS, INDENT, OP_PREFIX};

use idlc_mir::Ident;

#[derive(Debug, Default, Clone)]
pub struct Invoke(idlc_codegen_c::interface::functions::invoke::Invoke);

impl Invoke {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);

        me
    }
}

impl idlc_codegen::functions::ParameterVisitor for Invoke {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_input_primitive_buffer(ident, ty);
    }

    fn visit_input_untyped_buffer(&mut self, ident: &Ident) {
        self.0.visit_input_untyped_buffer(ident);
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_input_struct_buffer(ident, ty);
    }

    fn visit_input_object_array(
        &mut self,
        ident: &idlc_mir::Ident,
        ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let ty = ty.unwrap_or("ProxyBase").to_string();
        let mut obj_args = String::new();
        for _ in 0..cnt.into() {
            let idx = self.0.idx();
            obj_args.push_str(&format!(r"{ARGS}[{idx}].o,"));
        }
        self.0.pre.push(format!("const union {ident} {{"));
        self.0.pre.push(format!("{INDENT}~{ident}() {{}}"));
        self.0.pre.push(format!("{INDENT}{ty} inner[{cnt}];"));
        self.0
            .pre
            .push(format!("}} {ident} = {{.inner= {{ {obj_args} }}}};"));
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_input_primitive(ident, ty);
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        self.0.visit_input_bundled(packed_primitives);
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_input_big_struct(ident, ty);
    }

    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, ty: Option<&str>) {
        let idx = self.0.idx();
        let ty = ty.unwrap_or("ProxyBase").to_string();
        self.0.pre.push(format!("{ty} p_{ident}({ARGS}[{idx}].o);"));
        self.0.post.push(format!("p_{ident}.extract();"));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_output_primitive_buffer(ident, ty);
    }

    fn visit_output_untyped_buffer(&mut self, ident: &Ident) {
        self.0.visit_output_untyped_buffer(ident);
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_output_struct_buffer(ident, ty);
    }

    fn visit_output_object_array(
        &mut self,
        ident: &idlc_mir::Ident,
        ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let ty = ty.unwrap_or("ProxyBase").to_string();
        let mut objs = String::new();
        for i in 0..cnt.into() {
            if i != 0 {
                objs.push_str(", ");
            }
            let idx = self.0.idx();
            objs.push_str("Object_NULL");
            self.0
                .post
                .push(format!("{ARGS}[{idx}].o = p_{ident}[{i}].extract();"));
        }
        self.0
            .pre
            .push(format!("{ty} p_{ident}[{cnt}] = {{ {objs} }};"));
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_output_primitive(ident, ty);
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        self.0.visit_output_bundled(packed_primitives);
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        // self.0.visit_output_big_struct(ident, ty);
        let idx = self.0.idx();
        let name = format!("{}_ptr", ident);
        let sz = ty.size();
        let ty_ident = ty.ident.to_string();
        self.0.args.push(format!("{ARGS}[{idx}].b.size != {sz}"));
        if ty.contains_interfaces() {
            let name = format!("{}_ptr", ident);
            self.0.pre.push(format!(
                "{ty_ident} *{name} = &(*({ty_ident}*){ARGS}[{idx}].b.ptr);"
            ));

            let objects = ty.objects();
            for object in objects {
                let path = object
                    .0
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join("->");
                self.0.visit_output_object(
                    &idlc_mir::Ident {
                        ident: format!("{name}->{}", path),
                        span: object.0.last().unwrap().span,
                    },
                    object.1,
                );
            }
        } else {
            self.0.pre.push(format!(
                "{ty_ident} *{name} = ({ty_ident}*){ARGS}[{idx}].b.ptr;"
            ));
        }
    }

    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_object(&mut self, ident: &Ident, ty: Option<&str>) {
        let idx = self.0.idx();
        let ty = ty.unwrap_or("ProxyBase").to_string();
        self.0.pre.push(format!("{ty} p_{ident};"));
        self.0
            .post
            .push(format!("{ARGS}[{idx}].o = p_{ident}.extract();"));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    signature: &super::signature::Signature,
    counts: &idlc_codegen::counts::Counter,
    fn_ident: &str,
) -> String {
    let ident = &function.ident;
    let invoke = Invoke::new(function);
    let return_idents = signature.return_idents();

    let call = format!("int32_t r = {fn_ident}({return_idents});");

    let counts = format!(
        "{0}, {1}, {2}, {3}",
        counts.input_buffers, counts.output_buffers, counts.input_objects, counts.output_objects,
    );

    let mut body = vec![];
    body.push(format!(
        "if ({COUNTS} != ObjectCounts_pack({counts}){}){{",
        invoke.0.args()
    ));
    body.push(format!("{INDENT}break;"));
    body.push("}".to_string());
    body.extend(invoke.0.pre());
    body.push(call);
    body.extend(invoke.0.post());
    body.push("return r;".to_string());
    let formatted_body = idlc_codegen::join_with_prefix(&body, INDENT, 4, "\n");

    format!(
        r#"
{INDENT}{INDENT}{INDENT}case {OP_PREFIX}_{ident}: {{
{formatted_body}
{INDENT}{INDENT}{INDENT}}}"#
    )
}
