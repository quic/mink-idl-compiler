// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::{Ident, Primitive, StructInner};

use crate::types::change_primitive;

#[derive(Debug, Clone, Default)]
pub struct Signature {
    inputs: Vec<(String, String)>,
    returns: Vec<(String, String)>,
}

pub fn iter_to_string(iter: impl Iterator<Item = impl AsRef<str>>) -> String {
    let mut acc = String::new();
    for item in iter {
        acc += item.as_ref();
        acc.push(',');
    }

    if !acc.is_empty() {
        // Remove leading commas, this can make single element values a tuple.
        acc.truncate(acc.len() - 1);
    }

    acc
}

impl Signature {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        idlc_codegen::functions::visit_params(function, &mut me);

        me
    }

    pub fn params(&self) -> impl Iterator<Item = String> + '_ {
        self.inputs
            .iter()
            .map(|(ident, ty)| format!("{ty} {ident}"))
    }

    #[inline]
    pub fn return_idents(&self) -> impl Iterator<Item = &str> {
        self.returns.iter().map(|(ident, _)| ident.as_str())
    }
}

impl idlc_codegen::functions::ParameterVisitor for Signature {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: Primitive) {
        let ty = format!("{}[]", change_primitive(ty));
        self.inputs.push((format!("{}_val", ident), ty.to_string()));
        self.returns.push((ident.to_string(), ty));
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &StructInner) {
        let new_ty = format!("{}[]", ty.ident);
        self.inputs.push((format!("{}_val", ident), new_ty));
        self.returns.push((ident.to_string(), ty.ident.to_string()));
    }

    fn visit_input_object_array(
        &mut self,
        ident: &Ident,
        _ty: Option<&str>,
        _cnt: idlc_mir::Count,
    ) {
        use crate::interface::mink_primitives::IMINK_OBJECT;
        let ty = format!("{IMINK_OBJECT}[]");

        self.inputs.push((format!("{}_val", ident), ty.to_string()));
        self.returns
            .push((format!("{}_val", ident), ty.to_string()));
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: Primitive) {
        self.inputs
            .push((format!("{}_val", ident), change_primitive(ty).to_string()));
        self.returns
            .push((ident.to_string(), change_primitive(ty).to_string()));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.inputs
            .push((format!("{}_val", ident), ty.ident.to_string()));
        self.returns.push((ident.to_string(), ty.ident.to_string()));
    }
    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.inputs
            .push((format!("{}_val", ident), ty.ident.to_string()));
        self.returns.push((ident.to_string(), ty.ident.to_string()));
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        use crate::interface::mink_primitives::IMINK_OBJECT;

        self.inputs
            .push((format!("{}_val", ident), IMINK_OBJECT.to_string()));
        self.returns
            .push((ident.to_string(), IMINK_OBJECT.to_string()));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: Primitive) {
        let ty = format!("{}[][]", change_primitive(ty));
        self.inputs.push((format!("{}_ptr", ident), ty.to_string()));
        self.inputs
            .push((format!("{}_len", ident), "int".to_string()));
        self.returns.push((ident.to_string(), ty.to_string()));
        self.returns.push((format!("{}_len", ident), ty));
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &StructInner) {
        let new_ty = format!("{}[][]", ty.ident);
        self.inputs.push((format!("{}_ptr", ident), new_ty));
        self.inputs
            .push((format!("{}_len", ident), "int".to_string()));
        self.returns.push((ident.to_string(), ty.ident.to_string()));
        self.returns
            .push((format!("{}_len", ident), ty.ident.to_string()));
    }

    fn visit_output_object_array(&mut self, ident: &Ident, _: Option<&str>, _cnt: idlc_mir::Count) {
        use crate::interface::mink_primitives::IMINK_OBJECT;

        let ty = format!("{IMINK_OBJECT}[][]");
        self.inputs.push((format!("{}_ptr", ident), ty));
        self.inputs
            .push((format!("{}_len", ident), "int".to_string()));
        self.returns
            .push((ident.to_string(), IMINK_OBJECT.to_string()));
        self.returns
            .push((format!("{}_len", ident), "int".to_string()));
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: Primitive) {
        let ty = format!("{}[]", change_primitive(ty));
        self.inputs.push((format!("{}_ptr", ident), ty.to_string()));
        self.returns.push((ident.to_string(), ty));
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &StructInner) {
        let new_ty = format!("{}[]", ty.ident);
        self.inputs.push((format!("{}_ptr", ident), new_ty));
        self.returns.push((ident.to_string(), ty.ident.to_string()));
    }
    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &StructInner) {
        let new_ty = format!("{}[]", ty.ident);
        self.inputs.push((format!("{}_ptr", ident), new_ty));
        self.returns.push((ident.to_string(), ty.ident.to_string()));
    }

    fn visit_output_object(&mut self, ident: &Ident, _: Option<&str>) {
        use crate::interface::mink_primitives::IMINK_OBJECT;

        let ty = format!("{IMINK_OBJECT}[]");
        self.inputs.push((format!("{}_ptr", ident), ty));
        self.returns
            .push((ident.to_string(), IMINK_OBJECT.to_string()));
    }
}
