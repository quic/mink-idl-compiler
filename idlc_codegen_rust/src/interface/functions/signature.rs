// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::{Ident, Primitive, StructInner};

use crate::{
    ident::EscapedIdent,
    types::{change_primitive, namespaced_struct},
};

#[derive(Debug, Clone, Default)]
pub struct Signature {
    inputs: Vec<(String, String)>,
    outputs: Vec<(String, String)>,
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

pub fn idents_to_struct_path(objects: &[&Ident]) -> String {
    objects
        .iter()
        .fold(String::new(), |acc, x| acc + "." + x.as_ref())
}

impl Signature {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        idlc_codegen::functions::visit_params(function, &mut me);

        me
    }

    #[inline]
    pub fn param_idents(&self) -> impl Iterator<Item = &str> {
        self.inputs.iter().map(|(ident, _)| ident.as_str())
    }

    pub fn params(&self) -> impl Iterator<Item = String> + '_ {
        self.inputs
            .iter()
            .map(|(ident, ty)| format!("{ident}: {ty}"))
    }

    #[inline]
    pub fn return_types(&self) -> impl Iterator<Item = &str> {
        self.outputs.iter().map(|(_, ty)| ty.as_str())
    }

    #[inline]
    pub fn return_idents(&self) -> impl Iterator<Item = &str> {
        self.outputs.iter().map(|(ident, _)| ident.as_str())
    }

    #[inline]
    fn push_inputs(&mut self, ident: &Ident, ty: impl Into<String>) {
        let ident = EscapedIdent::new(ident);
        self.inputs.push((ident.to_string(), ty.into()));
    }

    #[inline]
    fn push_outputs(&mut self, ident: &Ident, ty: impl Into<String>) {
        let ident = EscapedIdent::new(ident);
        self.outputs.push((ident.to_string(), ty.into()));
    }
}

impl idlc_codegen::functions::ParameterVisitor for Signature {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: Primitive) {
        self.push_inputs(ident, format!("&[{}]", change_primitive(ty)));
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &StructInner) {
        self.push_inputs(ident, format!("&[{}]", namespaced_struct(ty)));
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: Primitive) {
        self.push_inputs(ident, change_primitive(ty));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.push_inputs(ident, format!("&{}", namespaced_struct(ty)));
    }
    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.push_inputs(ident, format!("&{}", namespaced_struct(ty)));
    }

    fn visit_input_object(&mut self, ident: &Ident, ty: Option<&str>) {
        use crate::interface::mink_primitives::{INTERFACES_BASE, OBJECT};
        use std::borrow::Cow;

        let ty = ty.map_or(Cow::Borrowed(OBJECT), |ty| {
            Cow::Owned(format!("{INTERFACES_BASE}::{}::{ty}", ty.to_lowercase()))
        });

        self.push_inputs(ident, format!("Option<&{ty}>"));
    }

    fn visit_input_object_array(&mut self, ident: &Ident, ty: Option<&str>, cnt: idlc_mir::Count) {
        use crate::interface::mink_primitives::{INTERFACES_BASE, OBJECT};
        use std::borrow::Cow;

        let ty = ty.map_or(Cow::Borrowed(OBJECT), |ty| {
            Cow::Owned(format!("{INTERFACES_BASE}::{}::{ty}", ty.to_lowercase()))
        });

        self.push_inputs(ident, format!("&[Option<{ty}>; {cnt}]"));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: Primitive) {
        self.push_inputs(ident, format!("&mut [{}]", change_primitive(ty)));
        self.push_inputs(
            &idlc_mir::Ident::new_without_span(format!("{ident}_lenout")),
            "&mut usize",
        );
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &StructInner) {
        self.push_inputs(ident, format!("&mut [{}]", namespaced_struct(ty)));
        self.push_inputs(
            &idlc_mir::Ident::new_without_span(format!("{ident}_lenout")),
            "&mut usize",
        );
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: Primitive) {
        self.push_outputs(ident, change_primitive(ty));
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.push_outputs(ident, namespaced_struct(ty));
    }
    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.push_outputs(ident, namespaced_struct(ty));
    }

    fn visit_output_object(&mut self, ident: &Ident, ty: Option<&str>) {
        use crate::interface::mink_primitives::{INTERFACES_BASE, OBJECT};
        use std::borrow::Cow;

        let ty = ty.map_or(Cow::Borrowed(OBJECT), |ty| {
            Cow::Owned(format!("{INTERFACES_BASE}::{}::{ty}", ty.to_lowercase()))
        });
        self.push_outputs(ident, format!("Option<{ty}>"));
    }

    fn visit_output_object_array(&mut self, ident: &Ident, ty: Option<&str>, cnt: idlc_mir::Count) {
        use crate::interface::mink_primitives::{INTERFACES_BASE, OBJECT};
        use std::borrow::Cow;

        let ty = ty.map_or(Cow::Borrowed(OBJECT), |ty| {
            Cow::Owned(format!("{INTERFACES_BASE}::{}::{ty}", ty.to_lowercase()))
        });
        self.push_outputs(ident, format!("[Option<{ty}>; {cnt}]"));
    }
}
