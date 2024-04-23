// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::Ident;

use crate::interface::mink_primitives::{
    BI, BO, BUNDLE_IN, BUNDLE_OUT, BYTE_BUFFER, BYTE_ORDER, OI, OO, OP_ID,
};
use crate::types::{capitalize_first_letter, change_primitive, get_struct_pair};

use super::serialization::TransportBuffer;

#[derive(Debug, Clone, Default)]
pub struct Implementation {
    initializations: Vec<String>,
    post_call: Vec<String>,
    obj_arr_in: bool,
    obj_arr_out: bool,

    bi_idx: usize,
    bo_idx: usize,
    bo_sz_idx: usize,
    io_idx: usize,
    oo_idx: usize,
}

impl Implementation {
    #[inline]
    fn bi_idx(&mut self) -> usize {
        let bi_idx = self.bi_idx;
        self.bi_idx += 1;

        bi_idx
    }

    #[inline]
    fn bo_idx(&mut self) -> usize {
        let bo_idx = self.bo_idx;
        self.bo_idx += 1;

        bo_idx
    }

    #[inline]
    fn bo_sz_idx(&mut self) -> usize {
        let bo_sz_idx = self.bo_sz_idx;
        self.bo_sz_idx += 1;

        bo_sz_idx
    }

    #[inline]
    fn io_idx(&mut self) -> usize {
        let io_idx = self.io_idx;
        self.io_idx += 1;

        io_idx
    }

    #[inline]
    fn oo_idx(&mut self) -> usize {
        let oo_idx = self.oo_idx;
        self.oo_idx += 1;

        oo_idx
    }
}

impl Implementation {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();

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

impl idlc_codegen::functions::ParameterVisitor for Implementation {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let bi_idx = self.bi_idx();
        let name = format!("{}_val", ident);
        let sz = ty.size();
        let ty = capitalize_first_letter(change_primitive(ty));
        let to_buffer = if !ty.is_empty() {
            format!("as{ty}Buffer().")
        } else {
            "".to_string()
        };
        self.initializations.push(format!(
            r#"if({name} != null) {{
                {BYTE_BUFFER} buffer_{name} = {BYTE_BUFFER}.allocate({name}.length*{sz}).order({BYTE_ORDER});
                buffer_{name}.{to_buffer}put({name});
                {BI}[{bi_idx}] = buffer_{name}.array();
            }}
            "#
        ));
    }

    fn visit_input_untyped_buffer(&mut self, ident: &idlc_mir::Ident) {
        let bi_idx = self.bi_idx();
        let name = format!("{}_val", ident);
        self.initializations.push(format!(
            r#"bi[{bi_idx}] = {name};
            "#
        ));
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let bi_idx = self.bi_idx();
        let name = format!("{}_val", ident);
        let sz = ty.size();
        let mut field_idents = Vec::new();
        let mut buffer_in = String::new();

        get_struct_pair(ty, &mut field_idents, "".to_string());
        for (field_ident, field_ty) in field_idents {
            let mut capitalized_ty = String::new();
            if let idlc_mir::Type::Primitive(p) = field_ty {
                capitalized_ty = capitalize_first_letter(change_primitive(p));
            }
            buffer_in.push_str(&format!(
                r#"buffer_{name}.put{capitalized_ty}(i.{field_ident});
                    "#
            ));
        }

        self.initializations.push(format!(
            r#"if({name} != null) {{
                {BYTE_BUFFER} buffer_{name} = {BYTE_BUFFER}.allocate({name}.length*{sz}).order({BYTE_ORDER});
                for ({0} i : {name}) {{
                    {buffer_in}
                }}
                {BI}[{bi_idx}] = buffer_{name}.array();
            }}
            "#,
            ty.ident
        ));
    }

    fn visit_input_object_array(
        &mut self,
        ident: &Ident,
        _ty: Option<&str>,
        _cnt: idlc_mir::Count,
    ) {
        let _io_idx = self.io_idx();
        let name = format!("{}_val", ident);
        self.obj_arr_in = true;
        self.initializations.push(format!(
            r#" for (int i=0; i<{name}.length; i++) {{
                {OI}[i] = {name}[i];
            }}
            "#,
        ));
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let bi_idx = self.bi_idx();
        let name = format!("{}_val", ident);
        let sz = ty.size();
        let ty = capitalize_first_letter(change_primitive(ty));
        self.initializations.push(format!(
            r#"{BI}[{bi_idx}] = {BYTE_BUFFER}.allocate({sz}).order({BYTE_ORDER}).put{ty}({name}).array();
            "#
        ));
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let bi_idx = self.bi_idx();
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bi_definition(false) else {
            unreachable!()
        };
        self.initializations.push(format!(
            r#"{BYTE_BUFFER} {BUNDLE_IN} = {BYTE_BUFFER}.allocate({size}).order({BYTE_ORDER});
            {definition}
            "#,
        ));
        self.initializations.push(format!(
            r#"{BI}[{bi_idx}] = {BUNDLE_IN}.array();
            "#,
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let bi_idx = self.bi_idx();
        let name = format!("{}_val", ident);
        let sz = ty.size();
        let mut fields = String::new();
        let mut field_idents = Vec::new();
        get_struct_pair(ty, &mut field_idents, "".to_string());
        for (field_ident, field_ty) in field_idents {
            let mut capitalized_ty = String::new();
            if let idlc_mir::Type::Primitive(p) = field_ty {
                capitalized_ty = capitalize_first_letter(change_primitive(p));
            }
            fields.push_str(&format!(
                r#"buffer_{name}.put{capitalized_ty}({name}.{field_ident});
            "#
            ));
        }

        self.initializations.push(format!(
            r#"{BYTE_BUFFER} buffer_{name} = {BYTE_BUFFER}.allocate({sz}).order({BYTE_ORDER});
            {fields}
            {BI}[{bi_idx}] = buffer_{name}.array();
            "#
        ));
    }
    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        let io_idx = self.io_idx();
        let name = format!("{}_val", ident);
        self.initializations.push(format!(
            r#"{OI}[{io_idx}] = {name};
            "#,
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let name = format!("{}_ptr", ident);
        let sz = ty.size();
        let ty = change_primitive(ty);
        let capitalized_ty = capitalize_first_letter(ty);
        let to_buffer = if !capitalized_ty.is_empty() {
            format!("as{capitalized_ty}Buffer().")
        } else {
            "".to_string()
        };
        self.initializations.push(format!(
            r#"boSizes[{bo_sz_idx}] = {ident}_len*{sz};
            "#,
        ));
        self.post_call.push(format!(
            r#"if({name} != null) {{
                int {ident}_lenout = Math.min({BO}[{bo_idx}].length/{sz}, {ident}_len);
                {name}[0] = new {ty}[{ident}_lenout];
                {BYTE_BUFFER}.wrap({BO}[{bo_idx}]).order({BYTE_ORDER}).{to_buffer}get({name}[0], 0, {ident}_lenout);
            }}
            "#,
        ));
    }

    fn visit_output_untyped_buffer(&mut self, ident: &idlc_mir::Ident) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let name = format!("{}_ptr", ident);
        self.initializations.push(format!(
            r#"boSizes[{bo_sz_idx}] = {ident}_len;
            "#,
        ));
        self.post_call.push(format!(
            r#"if({name} != null) {{
                {name}[0] = bo[{bo_idx}];
            }}
            "#,
        ));
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let name = format!("{}_ptr", ident);
        let sz = ty.size();

        let mut field_idents = Vec::new();
        let mut buffer_in = String::new();

        get_struct_pair(ty, &mut field_idents, "".to_string());
        let ty = ty.ident.to_string();
        for (field_ident, field_ty) in field_idents {
            let mut capitalized_ty = String::new();
            if let idlc_mir::Type::Primitive(p) = field_ty {
                capitalized_ty = capitalize_first_letter(change_primitive(p));
            }
            buffer_in.push_str(&format!(
                r#"{name}[0][i].{field_ident} = buffer_{name}.get{capitalized_ty}();
                    "#
            ));
        }

        self.initializations.push(format!(
            r#"boSizes[{bo_sz_idx}] = {ident}_len*{sz};
            "#,
        ));
        self.post_call.push(format!(
            r#"if({name} != null) {{
                int {ident}_lenout = Math.min({BO}[{bo_idx}].length/{sz}, {ident}_len);
                {name}[0] = new {ty}[{ident}_lenout];
                {BYTE_BUFFER} buffer_{name} = {BYTE_BUFFER}.wrap({BO}[{bo_idx}]).order({BYTE_ORDER});
                for (int i=0;i<{ident}_len;i++) {{
                    {buffer_in}
                }}
            }}
            "#,
        ));
    }

    fn visit_output_object_array(
        &mut self,
        ident: &Ident,
        _ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let name = format!("{}_ptr", ident);
        self.obj_arr_out = true;
        self.post_call.push(format!(
            r#"
            for (int i=0;i<{cnt};i++) {{
                {name}[0][i] = {OO}[i];
            }}
            "#,
        ));
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let name = format!("{}_ptr", ident);
        let sz = ty.size();
        let ty = capitalize_first_letter(change_primitive(ty));
        self.initializations.push(format!(
            r#"boSizes[{bo_sz_idx}] = {sz};
            "#,
        ));
        self.post_call.push(format!(
            r#"{BYTE_BUFFER} {BUNDLE_OUT};
            {BUNDLE_OUT} = {BYTE_BUFFER}.wrap({BO}[{bo_idx}]).order({BYTE_ORDER});
            if({name} != null) {{
                {name}[0] = {BUNDLE_OUT}.get{ty}();
            }}
            "#,
        ));
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bo_definition(false) else {
            unreachable!()
        };
        self.initializations.push(format!(
            r#"boSizes[{bo_sz_idx}] = {size};
            "#,
        ));
        self.post_call.push(format!(
            r#"{BYTE_BUFFER} {BUNDLE_OUT} = {BYTE_BUFFER}.wrap({BO}[{bo_idx}]).order({BYTE_ORDER});
            {definition}
            "#,
        ));
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let name = format!("{}_ptr", ident);
        let sz = ty.size();
        let mut fields = String::new();
        let mut field_idents = Vec::new();
        get_struct_pair(ty, &mut field_idents, "".to_string());
        for (field_ident, field_ty) in field_idents {
            let mut capitalized_ty = String::new();
            if let idlc_mir::Type::Primitive(p) = field_ty {
                capitalized_ty = capitalize_first_letter(change_primitive(p));
            }
            fields.push_str(&format!(
                r#"{name}[0].{field_ident} = {BUNDLE_OUT}.get{capitalized_ty}();
                "#
            ));
        }
        self.initializations.push(format!(
            r#"boSizes[{bo_sz_idx}] = {sz};
            "#,
        ));
        self.post_call.push(format!(
            r#"{BYTE_BUFFER} {BUNDLE_OUT} = {BYTE_BUFFER}.wrap({BO}[{bo_idx}]).order({BYTE_ORDER});
            if ({name} != null) {{
                {fields}
            }}
            "#,
        ));
    }
    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_object(&mut self, ident: &Ident, _: Option<&str>) {
        let oo_idx = self.oo_idx();
        let name = format!("{}_ptr", ident);
        self.post_call.push(format!(
            r#"if({name} != null) {{
                {name}[0] = {OO}[{oo_idx}];
            }}
            "#,
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    iface_ident: &str,
    documentation: &str,
    counts: &idlc_codegen::counts::Counter,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;

    let implementation = Implementation::new(function);
    let initializations = implementation.initializations();
    let post_call_assignments = implementation.post_call_assignments();
    let documentation = documentation.to_string().replace('\n', "\n        ");

    let params = super::signature::iter_to_string(signature.params());

    let mut inputs = String::new();

    let bi = if counts.input_buffers > 0 {
        inputs.push_str(&format!(
            r#"byte[][] {BI} = new byte[{}][];
            "#,
            counts.input_buffers
        ));
        BI.to_string()
    } else {
        "null".to_string()
    };
    let mut bo_sizes = "null".to_string();
    let bo = if counts.output_buffers > 0 {
        inputs.push_str(&format!(
            r#"byte[][] bo = new byte[{}][];
            int[] boSizes = new int[{}];
            "#,
            counts.output_buffers, counts.output_buffers
        ));
        bo_sizes = "boSizes".to_string();
        "bo".to_string()
    } else {
        "null".to_string()
    };
    let oi = if counts.input_objects > 0 {
        inputs.push_str(&format!(
            r#"IMinkObject[] oi = new IMinkObject[{}];
            "#,
            counts.input_objects
        ));
        "oi".to_string()
    } else if implementation.obj_arr_in {
        "oi".to_string()
    } else {
        "null".to_string()
    };
    let oo = if counts.output_objects > 0 {
        inputs.push_str(&format!(
            r#"IMinkObject[] oo = new IMinkObject[{}];
            "#,
            counts.output_objects
        ));
        "oo".to_string()
    } else if implementation.obj_arr_out {
        "oo".to_string()
    } else {
        "null".to_string()
    };

    format!(
        r#"
        {documentation}
        @Override
        public void {ident}({params}) throws IMinkObject.InvokeException {{
            {inputs}
            {initializations}
            minkObject.invoke({iface_ident}_{OP_ID}_{ident}, {bi}, {bo_sizes}, {bo}, {oi}, {oo});
            {post_call_assignments}
        }}"#
    )
}
