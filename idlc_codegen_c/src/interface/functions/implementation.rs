// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

use heck::ToSnakeCase;

use idlc_mir::Ident;

use crate::interface::variable_names::invoke::{
    ARGS, BI_NAME, BO_NAME, INDENT, OBJECTBUF, OBJECTBUFIN, OP_PREFIX,
};

use crate::types::change_primitive;

use super::serialization::TransportBuffer;

#[derive(Debug, Clone, Default)]
pub struct Implementation {
    pub args: Vec<String>,
    pub initializations: Vec<String>,
    pub pre_call: Vec<String>,
    pub post_call: Vec<String>,

    idx: usize,
}

impl Implementation {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self {
            args: vec![],
            initializations: vec![],
            pre_call: vec![],
            post_call: vec![],
            idx: 0,
        };

        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);

        me
    }

    pub fn args(&self) -> Vec<String> {
        self.args.clone()
    }

    pub fn initializations(&self) -> Vec<String> {
        self.initializations.clone()
    }

    pub fn pre_call_assignments(&self) -> Vec<String> {
        self.pre_call.clone()
    }

    pub fn post_call_assignments(&self) -> Vec<String> {
        self.post_call.clone()
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
            "{INDENT}{{.bi = ({OBJECTBUFIN}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"
        ));
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();

        self.args.push(format!(
            "{INDENT}{{.bi = ({OBJECTBUFIN}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"
        ));
    }

    fn visit_input_object_array(&mut self, ident: &Ident, _ty: Option<&str>, cnt: idlc_mir::Count) {
        for i in 0..cnt.into() {
            let _idx = self.idx();
            self.args
                .push(format!("{INDENT}{{.o = (*{ident}_ptr)[{i}] }},"));
        }
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let _idx = self.idx();
        let name = format!("{}_val", ident);
        let ty = change_primitive(ty);
        self.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ &{name}, sizeof({ty}) }} }},"
        ));
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer {
            definition, size, ..
        }) = packer.bi_definition()
        else {
            unreachable!()
        };
        let _idx = self.idx();
        self.initializations.extend(definition);
        self.initializations
            .last_mut()
            .unwrap()
            .push_str(&format!(" {BI_NAME};"));
        self.initializations.extend(packer.bi_embedded());
        self.initializations.extend(packer.bi_assignments());
        self.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ &{BI_NAME}, {size} }} }},"
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty_ident = ty.ident.to_string();
        if ty.contains_interfaces() {
            self.initializations
                .push(format!("{ty_ident} {ident}_cpy = *{name};"));
            self.args.push(format!(
                "{INDENT}{{.bi = ({OBJECTBUFIN}) {{ &{ident}_cpy, sizeof({ty_ident}) }} }},"
            ));
            let objects = ty.objects();
            for object in objects {
                let path = object
                    .0
                    .iter()
                    .map(|ident| ident.to_string())
                    .collect::<Vec<String>>()
                    .join(".");
                self.pre_call
                    .push(format!("{ident}_cpy.{path} = Object_NULL;"));
                self.visit_input_object(
                    &idlc_mir::Ident {
                        ident: format!("{ident}_cpy.{}", path),
                        span: object.0.last().unwrap().span,
                    },
                    object.1,
                );
            }
        } else {
            self.args.push(format!(
                "{INDENT}{{.bi = ({OBJECTBUFIN}) {{ {name}, sizeof({ty_ident}) }} }},"
            ));
        }
    }

    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        let _idx = self.idx();
        self.args.push(format!("{INDENT}{{.o = {ident} }},"));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);
        self.post_call.push(format!(
            "*{ident}_lenout = {ARGS}[{idx}].b.size / sizeof({ty});"
        ));
        self.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"
        ));
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.post_call.push(format!(
            "*{ident}_lenout = {ARGS}[{idx}].b.size / sizeof({ty});"
        ));
        self.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ {name}, {ident}_len * sizeof({ty}) }} }},"
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
            self.args.push(format!("{INDENT}{{.o = Object_NULL}},"));
            self.post_call
                .push(format!("(*{ident}_ptr)[{i}] = {ARGS}[{idx}].o;"));
        }
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty = change_primitive(ty);
        self.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ {name}, sizeof({ty}) }} }},"
        ));
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer {
            definition,
            size,
            initialization,
        }) = packer.bo_definition()
        else {
            unreachable!()
        };
        let _idx = self.idx();

        self.initializations.extend(definition);
        self.initializations
            .last_mut()
            .unwrap()
            .push_str(&format!(" {BO_NAME} = {{{initialization}}};"));
        self.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ &{BO_NAME}, {size} }} }},"
        ));
        self.post_call.extend(packer.post_bo_assignments());
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.idx();
        let name = format!("{}_ptr", ident);
        let ty_ident = ty.ident.to_string();
        self.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{  {name}, sizeof({ty_ident}) }} }},"
        ));
        let objects = ty.objects();
        for object in objects {
            let path = object
                .0
                .iter()
                .map(|ident| ident.to_string())
                .collect::<Vec<String>>()
                .join(".");
            self.initializations
                .push(format!("{name}->{path} = Object_NULL;"));
            self.visit_output_object(
                &idlc_mir::Ident {
                    ident: format!("{name}->{}", path),
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
        let idx = self.idx();
        let mut name = format!("{ident}");
        if !ident.contains("->") {
            name = format!("*{name}");
        }
        self.post_call.push(format!("{name} = {ARGS}[{idx}].o;"));
        self.args.push(format!("{INDENT}{{.o = Object_NULL }},"));
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
    let ident = &function.ident.to_snake_case();
    let total = counts.total();

    let params = signature.params();

    let implementation = Implementation::new(function);

    let arguments = if total > 0 {
        format!(
            "{ARGS}, ObjectCounts_pack({0}, {1}, {2}, {3})",
            counts.input_buffers,
            counts.output_buffers,
            counts.input_objects,
            counts.output_objects
        )
    } else {
        "NULL, 0".to_string()
    };

    let mut body = Vec::new();
    body.extend(implementation.initializations());
    if total > 0 {
        body.push(format!("ObjectArg {ARGS}[] = {{"));
        body.extend(implementation.args());
        body.push("};".to_string());
    }
    body.extend(implementation.pre_call_assignments());
    body.push(format!(
        "int32_t result = Object_invoke(self, {iface_ident}_{OP_PREFIX}_{ident}, {arguments});"
    ));
    body.extend(implementation.post_call_assignments());
    body.push("return result;".to_string());
    let formatted_body = idlc_codegen::join_with_prefix(&body, INDENT, 1, "\n");

    format!(
        r#"
{documentation}
static inline int32_t {current_iface_ident}_{ident}(Object self{params})
{{
{formatted_body}
}}
"#
    )
}
