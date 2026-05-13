// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_codegen_c::interface::variable_names::invoke::{
    ARGS, BI_NAME, BO_NAME, INDENT, OBJECTBUF, OBJECTBUFIN, OP_PREFIX,
};
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
            self.0
                .args
                .push(format!("{INDENT}{{.o = ({ident}_ref)[{i}].get() }},"));
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
        let Some(TransportBuffer {
            definition, size, ..
        }) = packer.bi_definition()
        else {
            unreachable!()
        };
        let _idx = self.0.idx();
        self.0.initializations.extend(definition);
        self.0
            .initializations
            .last_mut()
            .unwrap()
            .push_str(&format!(" {BI_NAME};"));
        self.0.initializations.extend(packer.bi_embedded());
        self.0.initializations.extend(packer.bi_assignments());
        self.0.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ &{BI_NAME}, {size} }} }},"
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.0.idx();
        let name = format!("{}_ref", ident);
        let ty_ident = ty.ident.to_string();
        if ty.contains_interfaces() {
            self.0
                .initializations
                .push(format!("{ty_ident} {ident}_cpy = {name};"));
            self.0.args.push(format!(
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
                self.0
                    .pre_call
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
            self.0.args.push(format!(
                "{INDENT}{{.bi = ({OBJECTBUFIN}) {{ &{name}, sizeof({ty_ident}) }} }},"
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
            self.0.args.push(format!("{INDENT}{{.o = {name} }},"));
        } else {
            self.0.args.push(format!("{INDENT}{{.o = {name}.get() }},"));
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
            self.0.args.push(format!("{INDENT}{{.o = Object_NULL }},"));
        }

        self.0
            .post_call
            .push(format!("for(size_t arg_idx=0;arg_idx<{cnt};arg_idx++)"));
        self.0.post_call.push(format!(
            "{INDENT}({ident}_ref)[arg_idx].consume({ARGS}[{idx}+arg_idx].o);"
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
        let Some(TransportBuffer {
            definition,
            size,
            initialization,
        }) = packer.bo_definition()
        else {
            unreachable!()
        };
        let _idx = self.0.idx();

        self.0.initializations.extend(definition);
        self.0
            .initializations
            .last_mut()
            .unwrap()
            .push_str(&format!(" {BO_NAME} = {{{initialization}}};"));
        self.0.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{ &{BO_NAME}, {size} }} }},"
        ));
        self.0.post_call.extend(packer.post_bo_assignments());
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let _idx = self.0.idx();
        let name = format!("{}_ref", ident);
        let ty_ident = ty.ident.to_string();
        self.0.args.push(format!(
            "{INDENT}{{.b = ({OBJECTBUF}) {{  &{name}, sizeof({ty_ident}) }} }},"
        ));

        let objects = ty.objects();
        for object in objects {
            let path = object
                .0
                .iter()
                .map(|ident| ident.to_string())
                .collect::<Vec<String>>()
                .join(".");
            self.0
                .initializations
                .push(format!("{name}.{path} = Object_NULL;"));
            self.visit_output_object(
                &idlc_mir::Ident {
                    ident: format!("({name}.{})", path),
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
        if ident.contains('.') || ident.contains("->") {
            self.0.post_call.push(format!("{ident} = {ARGS}[{idx}].o;"));
        } else {
            self.0
                .post_call
                .push(format!("{ident}.consume({ARGS}[{idx}].o);"));
        }
        self.0.args.push(format!("{INDENT}{{.o = Object_NULL }},"));
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
        "nullptr, 0".to_string()
    };

    let mut body = Vec::new();
    body.extend(implementation.0.initializations());
    if total > 0 {
        body.push(format!("ObjectArg {ARGS}[] = {{"));
        body.extend(implementation.0.args());
        body.push("};".to_string());
    }
    body.extend(implementation.0.pre_call_assignments());
    body.push(format!(
        "int32_t result = invoke({OP_PREFIX}_{ident}, {arguments});"
    ));
    body.push("if (Object_OK != result) { return result; }".to_string());
    body.extend(implementation.0.post_call_assignments());
    body.push("return result;".to_string());
    let formatted_body = idlc_codegen::join_with_prefix(&body, INDENT, 2, "\n");

    format!(
        r#"
{documentation}
{INDENT}virtual int32_t {ident}({params}) {{
{formatted_body}
{INDENT}}}
"#
    )
}
