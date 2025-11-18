// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::{Count, Ident, Primitive, StructInner};

#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub input_buffers: u8,
    pub input_objects: u8,
    pub output_objects: u8,
    pub output_buffers: u8,
    pub total_bundled_input: u8,
    pub total_bundled_output: u8,

    has_bundled_input: bool,
    has_bundled_output: bool,
}

impl super::functions::ParameterVisitor for Counter {
    #[inline]
    fn visit_input_primitive_buffer(&mut self, _: &Ident, _: Primitive) {
        self.input_buffers += 1;
    }

    #[inline]
    fn visit_input_struct_buffer(&mut self, _: &Ident, _: &StructInner) {
        self.input_buffers += 1;
    }

    #[inline]
    fn visit_input_primitive(&mut self, _: &Ident, _: Primitive) {
        self.has_bundled_input = true;
        self.total_bundled_input += 1;
    }

    #[inline]
    fn visit_input_small_struct(&mut self, _: &Ident, _: &StructInner) {
        self.has_bundled_input = true;
        self.total_bundled_input += 1;
    }

    #[inline]
    fn visit_input_big_struct(&mut self, _: &Ident, s: &StructInner) {
        self.input_buffers += 1;
        self.input_objects += u8::try_from(s.objects().len()).unwrap();
    }

    #[inline]
    fn visit_input_object(&mut self, _: &Ident, _: Option<&str>) {
        self.input_objects += 1;
    }

    #[inline]
    fn visit_input_object_array(&mut self, _: &Ident, _: Option<&str>, cnt: Count) {
        self.input_objects += u8::try_from(cnt.get()).unwrap();
    }

    #[inline]
    fn visit_output_primitive_buffer(&mut self, _: &Ident, _: Primitive) {
        self.output_buffers += 1;
    }

    #[inline]
    fn visit_output_struct_buffer(&mut self, _: &Ident, _: &StructInner) {
        self.output_buffers += 1;
    }

    #[inline]
    fn visit_output_primitive(&mut self, _: &Ident, _: Primitive) {
        self.has_bundled_output = true;
        self.total_bundled_output += 1;
    }
    #[inline]
    fn visit_output_small_struct(&mut self, _: &Ident, _: &StructInner) {
        self.has_bundled_output = true;
        self.total_bundled_output += 1;
    }

    #[inline]
    fn visit_output_big_struct(&mut self, _: &Ident, s: &StructInner) {
        self.output_buffers += 1;
        self.output_objects += u8::try_from(s.objects().len()).unwrap();
    }

    #[inline]
    fn visit_output_object(&mut self, _: &Ident, _: Option<&str>) {
        self.output_objects += 1;
    }

    #[inline]
    fn visit_output_object_array(&mut self, _: &Ident, _: Option<&str>, cnt: Count) {
        self.output_objects += u8::try_from(cnt.get()).unwrap();
    }
}

impl Counter {
    #[must_use]
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        super::functions::visit_params(function, &mut me);

        me.input_buffers += if me.has_bundled_input { 1 } else { Default::default() };
        me.output_buffers += if me.has_bundled_output { 1 } else { Default::default() };

        me
    }

    #[inline]
    #[must_use]
    pub const fn total(&self) -> u32 {
        (self.input_buffers + self.input_objects + self.output_buffers + self.output_objects) as u32
    }
}
