// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use crate::{
    interface::variable_names::invoke::{ARGS, BI, BO, CONST, COUNTS, INDENT, OP_PREFIX},
    types::change_primitive,
};

use super::serialization::TransportBuffer;

#[derive(Debug, Default, Clone)]
pub struct Invoke {
    pub args: Vec<String>,
    pub pre: Vec<String>,
    pub post: Vec<String>,

    is_no_typed_objects: bool,

    idx: usize,
}

impl Invoke {
    pub fn new(function: &idlc_mir::Function, is_no_typed_objects: bool) -> Self {
        let mut me = Self {
            args: vec![],
            pre: vec![],
            post: vec![],
            is_no_typed_objects,
            idx: 0,
        };

        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);
        me
    }

    pub fn args(&self) -> String {
        let mut acc = String::new();
        for arg in &self.args {
            acc.push_str(" || ");
            acc += arg.as_ref();
        }
        acc
    }

    pub fn pre(&self) -> Vec<String> {
        self.pre.clone()
    }

    pub fn post(&self) -> Vec<String> {
        self.post.clone()
    }
}

impl Invoke {
    #[inline]
    pub fn idx(&mut self) -> usize {
        let idx = self.idx;
        self.idx += 1;

        idx
    }
}

impl idlc_codegen::functions::ParameterVisitor for Invoke {
    fn visit_input_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let idx = self.idx();
        let sz = ty.size();
        let ty: &str = change_primitive(ty);
        self.pre.push(format!(
            "{CONST} {ty} *{ident}_ptr = ({CONST} {ty}*){ARGS}[{idx}].b.ptr;"
        ));
        self.pre
            .push(format!("size_t {ident}_len = {ARGS}[{idx}].b.size / {sz};"));
    }

    fn visit_input_untyped_buffer(&mut self, ident: &idlc_mir::Ident) {
        let idx = self.idx();
        let ty = "void".to_string();
        let name = format!("*{}_ptr", ident);
        self.pre.push(format!(
            "{CONST} {ty} {name} = ({CONST} {ty}*){ARGS}[{idx}].b.ptr;"
        ));
        self.pre
            .push(format!("size_t {ident}_len = {ARGS}[{idx}].b.size;"));
    }

    fn visit_input_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let ty: &str = ty.ident.as_ref();
        let name = format!("*{}_ptr", ident);
        self.pre.push(format!(
            "{CONST} {ty} {name} = ({CONST} {ty}*){ARGS}[{idx}].b.ptr;"
        ));
        self.pre.push(format!(
            "size_t {ident}_len = {ARGS}[{idx}].b.size / sizeof({ty});"
        ));
    }

    fn visit_input_object_array(
        &mut self,
        ident: &idlc_mir::Ident,
        ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let ty = if self.is_no_typed_objects {
            "Object".to_string()
        } else {
            ty.unwrap_or("Object").to_string()
        };
        let mut objs = String::new();
        for _ in 0..cnt.into() {
            let idx = self.idx();
            objs.push_str(&format!(r"{ARGS}[{idx}].o,"));
        }
        self.pre
            .push(format!("const {ty} {ident}[{cnt}] = {{ {objs} }};"));
    }

    fn visit_input_primitive(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let idx = self.idx();
        let sz = ty.size();
        let ty: &str = change_primitive(ty);
        let name = format!("*{}_ptr", ident);
        self.args.push(format!("{ARGS}[{idx}].b.size != {sz}"));
        self.pre.push(format!(
            "{CONST} {ty} {name} = ({CONST} {ty}*){ARGS}[{idx}].b.ptr;"
        ));
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer {
            mut definition,
            size,
            ..
        }) = packer.bi_definition(// true
        )
        else {
            unreachable!()
        };
        let idx = self.idx();
        self.args.push(format!("{ARGS}[{idx}].b.size != {size}"));
        self.pre.push(format!("{CONST} {}", definition.remove(0)));
        self.pre.extend(definition);
        self.pre
            .last_mut()
            .unwrap()
            .push_str(&format!(" *i = (const struct {BI}*){ARGS}[{idx}].b.ptr;"));
    }

    fn visit_input_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let sz = ty.size();
        let ty_ident = ty.ident.to_string();
        self.args.push(format!("{ARGS}[{idx}].b.size != {sz}"));
        if ty.contains_interfaces() {
            self.pre.push(format!(
                "{ty_ident} {name} = *({CONST} {ty_ident}*){ARGS}[{idx}].b.ptr;"
            ));
            let objects = ty.objects();
            for object in objects {
                let path = object
                    .0
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(".");
                let idx = self.idx();
                let field_ident = format!("{name}.{}", path);
                self.pre.push(format!("{field_ident} = {ARGS}[{idx}].o;"));
            }
        } else {
            self.pre.push(format!(
                "{CONST} {ty_ident} *{name} = ({CONST} {ty_ident}*){ARGS}[{idx}].b.ptr;"
            ));
        }
    }
    fn visit_input_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &idlc_mir::Ident, ty: Option<&str>) {
        let idx = self.idx();
        let ty = if self.is_no_typed_objects {
            "Object".to_string()
        } else {
            ty.unwrap_or("Object").to_string()
        };

        self.pre
            .push(format!("{ty} *{ident}_ptr = &{ARGS}[{idx}].o;"));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let idx = self.idx();
        let sz = ty.size();
        let ty: &str = change_primitive(ty);
        let name = format!("*{}_ptr", ident);
        self.pre
            .push(format!("{ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr;"));
        self.pre
            .push(format!("size_t {ident}_len = {ARGS}[{idx}].b.size / {sz};"));
        self.post
            .push(format!("{ARGS}[{idx}].b.size = {ident}_len * {sz};"));
    }

    fn visit_output_untyped_buffer(&mut self, ident: &idlc_mir::Ident) {
        let idx = self.idx();
        let ty = "void".to_string();
        let name = format!("*{}_ptr", ident);
        self.pre
            .push(format!("{ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr;"));
        self.pre
            .push(format!("size_t {ident}_len = {ARGS}[{idx}].b.size;"));
        self.post
            .push(format!("{ARGS}[{idx}].b.size = {ident}_len;"));
    }

    fn visit_output_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let ty: &str = ty.ident.as_ref();
        let name = format!("*{}_ptr", ident);
        self.pre
            .push(format!("{ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr;"));
        self.pre.push(format!(
            "size_t {ident}_len = {ARGS}[{idx}].b.size / sizeof({ty});"
        ));
        self.post.push(format!(
            "{ARGS}[{idx}].b.size = {ident}_len * sizeof({ty});"
        ));
    }

    fn visit_output_object_array(
        &mut self,
        ident: &idlc_mir::Ident,
        ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let name = format!("{}", ident);
        let ty = if self.is_no_typed_objects {
            "Object".to_string()
        } else {
            ty.unwrap_or("Object").to_string()
        };
        let mut objs = String::new();
        let mut obj_assign = String::new();
        for i in 0..cnt.into() {
            let idx = self.idx();
            objs.push_str("Object_NULL, ");
            obj_assign.push_str(&format!(
                r#"{ARGS}[{idx}].o = {name}[{i}]; \
                "#
            ))
        }

        self.pre.push(format!("{ty} {name}[{cnt}] = {{ {objs} }};"));
        self.post.push(obj_assign);
    }

    fn visit_output_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let sz = ty.size();
        let ty_ident = ty.ident.to_string();
        self.args.push(format!("{ARGS}[{idx}].b.size != {sz}"));
        if ty.contains_interfaces() {
            self.pre.push(format!(
                "{ty_ident} *{name} = &(*({ty_ident}*){ARGS}[{idx}].b.ptr);"
            ));
            let objects = ty.objects();
            for object in objects {
                let path = object
                    .0
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(".");
                self.visit_output_object(
                    &idlc_mir::Ident {
                        ident: format!("{name}->{}", path),
                        span: object.0.last().unwrap().span,
                    },
                    object.1,
                );
            }
        } else {
            self.pre.push(format!(
                "{ty_ident} *{name} = ({ty_ident}*){ARGS}[{idx}].b.ptr;"
            ));
        }
    }

    fn visit_output_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_primitive(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let idx = self.idx();
        let sz = ty.size();
        let ty: &str = change_primitive(ty);
        let name = format!("*{}_ptr", ident);
        self.args.push(format!("{ARGS}[{idx}].b.size != {sz}"));
        self.pre
            .push(format!("{ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr;"));
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer {
            definition, size, ..
        }) = packer.bo_definition(// true
        )
        else {
            unreachable!()
        };
        let idx = self.idx();
        self.args.push(format!("{ARGS}[{idx}].b.size != {size}"));
        self.pre.extend(definition);
        self.pre
            .last_mut()
            .unwrap()
            .push_str(&format!(" *o = (struct {BO}*){ARGS}[{idx}].b.ptr;"));
    }

    fn visit_output_object(&mut self, ident: &idlc_mir::Ident, ty: Option<&str>) {
        let idx = self.idx();
        let mut name = format!("{ident}");
        let ty = if self.is_no_typed_objects {
            "Object".to_string()
        } else {
            ty.unwrap_or("Object").to_string()
        };
        if !ident.contains("->") && !ident.contains('.') {
            name = format!("*{name}");
            self.pre.push(format!("{ty} {name} = &{ARGS}[{idx}].o;"));
        } else {
            self.pre.push(format!("{name} = {ARGS}[{idx}].o;"));
            self.post.push(format!("{ARGS}[{idx}].o = {name};"));
            self.post.push(format!("{name} = Object_NULL;"));
        }
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    iface_ident: &str,
    signature: &super::signature::Signature,
    counts: &idlc_codegen::counts::Counter,
    is_no_typed_objects: bool,
) -> String {
    let ident = &function.ident;

    let invoke = Invoke::new(function, is_no_typed_objects);

    let return_idents = super::signature::iter_to_string(signature.return_idents());
    let call = format!(r"int32_t r = prefix##{ident}(me{return_idents});");

    let counts = format!(
        "{0}, {1}, {2}, {3}",
        counts.input_buffers, counts.output_buffers, counts.input_objects, counts.output_objects,
    );

    let mut body = vec![];
    body.push(format!(
        "if ({COUNTS} != ObjectCounts_pack({counts}){}) {{",
        invoke.args()
    ));
    body.push(format!("{INDENT}break;"));
    body.push("}".to_string());
    body.extend(invoke.pre());
    body.push(call);
    body.extend(invoke.post());
    body.push("return r;".to_string());
    let formatted_body = idlc_codegen::join_with_prefix(&body, INDENT, 4, " \\\n");

    format!(
        r#" \
{INDENT}{INDENT}{INDENT}case {iface_ident}_{OP_PREFIX}_{ident}: {{ \
{formatted_body} \
{INDENT}{INDENT}{INDENT}}}"#
    )
}
