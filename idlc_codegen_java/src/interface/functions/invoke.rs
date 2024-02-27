use crate::interface::mink_primitives::{
    BI, BO, BUNDLE_IN, BUNDLE_OUT, BYTE_BUFFER, BYTE_ORDER, IMINK_OBJECT, OI, OO, OP_ID,
};
use crate::types::{capitalize_first_letter, change_primitive, get_struct_pair};

use super::serialization::TransportBuffer;

#[derive(Debug, Default, Clone)]
pub struct Invoke {
    pre: Vec<String>,
    post: Vec<String>,

    bi_idx: usize,
    bo_idx: usize,
    bo_sz_idx: usize,
    io_idx: usize,
    oo_idx: usize,
}

impl Invoke {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();

        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);
        me
    }

    pub fn pre(&self) -> String {
        self.pre.concat()
    }

    pub fn post(&self) -> String {
        self.post.concat()
    }
}

impl Invoke {
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

impl idlc_codegen::functions::ParameterVisitor for Invoke {
    fn visit_input_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let bi_idx = self.bi_idx();
        let ty = change_primitive(ty);
        let capitalized_ty = capitalize_first_letter(ty);
        let to_buffer = if !capitalized_ty.is_empty() {
            format!("as{capitalized_ty}Buffer().")
        } else {
            "".to_string()
        };

        self.pre.push(format!(
            r#"{ty}[] {ident} = {BYTE_BUFFER}.wrap({BI}[{bi_idx}]).order({BYTE_ORDER}).{to_buffer}array();
                    "#
        ));
    }

    fn visit_input_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let bi_idx = self.bi_idx();
        let mut fields = String::new();
        let mut field_idents = Vec::new();
        get_struct_pair(ty, &mut field_idents, "".to_string());
        for (field_ident, field_ty) in field_idents {
            let mut capitalized_ty = String::new();
            if let idlc_mir::Type::Primitive(p) = field_ty {
                capitalized_ty = capitalize_first_letter(change_primitive(p));
            }
            fields.push_str(&format!(
                r#"{ident}[0].{field_ident}={BUNDLE_IN}{bi_idx}.get{capitalized_ty}();
                    "#
            ));
        }
        let ty = ty.ident.to_string();
        self.pre.push(format!(
            r#"{ty}[] {ident} = new {ty}[1];
                    {BYTE_BUFFER} {BUNDLE_IN}{bi_idx} = {BYTE_BUFFER}.wrap({BI}[{bi_idx}]).order({BYTE_ORDER});
                    {fields}
                    "#
        ));
    }

    fn visit_input_object_array(
        &mut self,
        ident: &idlc_mir::Ident,
        _ty: Option<&str>,
        _cnt: idlc_mir::Count,
    ) {
        self.pre.push(format!(
            r#"IMinkObject[] {ident}_val = new IMinkObject[{OI}.length];
                    for (int i=0;i<{OI}.length;i++) {{
                        {ident}_val[i] = {OI}[i];
                    }}
                    "#
        ));
    }

    fn visit_input_primitive(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let bi_idx = self.bi_idx();
        let sz = ty.size();
        let ty = change_primitive(ty);
        let capitalized_ty = capitalize_first_letter(ty);

        self.pre.push(format!(
            r#"if ({BI}[{bi_idx}].length != {sz}) {{
                        break;
                    }}
                    {ty} {ident} = {BYTE_BUFFER}.wrap({BI}[{bi_idx}]).order({BYTE_ORDER}).get{capitalized_ty}();
                    "#
        ));
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bi_definition(true) else {
            unreachable!()
        };
        let bi_idx = self.bi_idx();
        self.pre.push(format!(
            r#"if ({BI}[{bi_idx}].length != {size}) {{
                        break;
                    }}
                    {BYTE_BUFFER} {BUNDLE_IN} = {BYTE_BUFFER}.wrap({BI}[{bi_idx}]).order({BYTE_ORDER});
                    {definition}
                    "#,
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let bi_idx = self.bi_idx();
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
                r#"{ident}.{field_ident}={BUNDLE_IN}{bi_idx}.get{capitalized_ty}();
                    "#
            ));
        }
        let ty = ty.ident.to_string();
        self.pre.push(format!(
            r#"if ({BI}[{bi_idx}].length != {sz}) {{
                        break;
                    }}
                    {ty} {ident} = new {ty}();
                    {BYTE_BUFFER} {BUNDLE_IN}{bi_idx} = {BYTE_BUFFER}.wrap({BI}[{bi_idx}]).order({BYTE_ORDER});
                    {fields}
                    "#
        ));
    }
    fn visit_input_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &idlc_mir::Ident, _: Option<&str>) {
        let io_idx = self.io_idx();
        let ty = IMINK_OBJECT.to_string();

        self.pre.push(format!(
            r#"{ty} {ident};
                    {ident} = {OI}[{io_idx}];
                    "#
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let sz = ty.size();
        let ty = change_primitive(ty);
        let capitalized_ty = capitalize_first_letter(ty);
        let to_buffer = if !capitalized_ty.is_empty() {
            format!("as{capitalized_ty}Buffer().")
        } else {
            "".to_string()
        };

        self.pre.push(format!(
            r#"{ty}[][] {ident} = new {ty}[1][];
                    int {ident}_len = boSizes[{bo_sz_idx}];
                    "#,
        ));
        self.post.push(format!(
            r#"{BYTE_BUFFER} buffer_{ident} = {BYTE_BUFFER}.allocate({ident}.length*{sz}).order({BYTE_ORDER});
                    buffer_{ident}.{to_buffer}put({ident}[0]);
                    {BO}[{bo_idx}] = buffer_{ident}.array();
                    "#,
        ));
    }

    fn visit_output_untyped_buffer(&mut self, ident: &idlc_mir::Ident) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        self.pre.push(format!(
            r#"byte[][] {ident} = new byte[1][];
                    int {ident}_len = boSizes[{bo_sz_idx}];
                    "#,
        ));
        self.post.push(format!(
            r#"{BO}[{bo_idx}]={ident}[0];
                    "#,
        ));
    }

    fn visit_output_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
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
                r#"buffer_{ident}.put{capitalized_ty}(i.{field_ident});
                    "#
            ));
        }

        let ty = ty.ident.to_string();

        self.pre.push(format!(
            r#"{ty}[][] {ident} = new {ty}[1][];
                    int {ident}_len = boSizes[{bo_sz_idx}];
                    "#,
        ));

        self.post.push(format!(
            r#"{BYTE_BUFFER} buffer_{ident} = {BYTE_BUFFER}.allocate({ident}.length*{sz}).order({BYTE_ORDER});
                    for ({ty} i : {ident}[0]) {{
                        {buffer_in}
                    }}
                    {BO}[{bo_idx}] = buffer_{ident}.array();
                    "#,
        ));
    }

    fn visit_output_object_array(
        &mut self,
        ident: &idlc_mir::Ident,
        _ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let ty = IMINK_OBJECT.to_string();
        self.pre.push(format!(
            r#"{ty}[][] {ident} = new {ty}[1][{OO}.length];
                    int {ident}_len = oo.length;
                    "#,
        ));
        self.post.push(format!(
            r#"for (int i=0;i<{cnt};i++) {{
                        oo[i] = {ident}[0][i];
                    }}
                    "#
        ));
    }

    fn visit_output_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
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
                r#"buffer_{ident}.put{capitalized_ty}({ident}[0].{field_ident});
                    "#
            ));
        }
        let ty = ty.ident.to_string();

        self.pre.push(format!(
            r#"if (boSizes[{bo_sz_idx}] != {sz}) {{
                        break;
                    }}
                    {ty}[] {ident} = new {ty}[1];
                    "#,
        ));
        self.post.push(format!(
            r#"{BYTE_BUFFER} buffer_{ident} = {BYTE_BUFFER}.allocate({sz}).order({BYTE_ORDER});
                    {fields}
                    {BO}[{bo_idx}] = buffer_{ident}.array();
                    "#,
        ));
    }
    fn visit_output_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_primitive(&mut self, ident: &idlc_mir::Ident, ty: idlc_mir::Primitive) {
        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let sz = ty.size();
        let ty = change_primitive(ty);
        let capitalized_ty = capitalize_first_letter(ty);

        self.pre.push(format!(
            r#"if (boSizes[{bo_sz_idx}] != {sz}) {{
                        break;
                    }}
                    {ty}[] {ident} = new {ty}[1];
                    "#,
        ));
        self.post.push(format!(
            r#"{BO}[{bo_idx}] = {BYTE_BUFFER}.allocate({sz}).order({BYTE_ORDER}).put{capitalized_ty}({ident}[0]).array();
                    "#,
        ));
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bo_definition(true) else {
            unreachable!()
        };

        let bo_idx = self.bo_idx();
        let bo_sz_idx = self.bo_sz_idx();
        let bo_assignments = packer.pre_bo_assignments();

        self.pre.push(format!(
            r#"if (boSizes[{bo_sz_idx}] != {size}) {{
                        break;
                    }}
                    {bo_assignments}
                    "#,
        ));
        self.post.push(format!(
            r#"{BYTE_BUFFER} {BUNDLE_OUT} = {BYTE_BUFFER}.allocate({size}).order({BYTE_ORDER});
                    {definition}
                    {BO}[{bo_idx}] = {BUNDLE_OUT}.array();
                    "#
        ));
    }

    fn visit_output_object(&mut self, ident: &idlc_mir::Ident, _: Option<&str>) {
        let oo_idx = self.oo_idx();
        let ty = IMINK_OBJECT.to_string();
        self.pre.push(format!(
            r#"{ty}[] {ident} = new {ty}[1];
                    "#,
        ));
        self.post.push(format!(
            r#"{OO}[{oo_idx}] = {ident}[0];
                    "#
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    current_iface_ident: &str,
    iface_ident: &str,
    signature: &super::signature::Signature,
) -> String {
    let ident = &function.ident;

    let invokes = Invoke::new(function);
    let pre = invokes.pre();
    let post = invokes.post();

    let returns = super::signature::iter_to_string(signature.return_idents());

    format!(
        r#"case {iface_ident}_{OP_ID}_{ident}: {{
                    {pre}
                    (({current_iface_ident})mObj).{ident}({returns});
                    {post}
                    return;
                }}
                "#
    )
}
