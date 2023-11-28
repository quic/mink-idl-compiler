use idlc_mir::{Ident, Primitive, StructInner};

#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub input_buffers: u8,
    pub input_objects: u8,
    pub output_objects: u8,
    pub output_buffers: u8,

    has_bundled_input: bool,
    has_bundled_output: bool,
}

impl super::functions::ParameterVisitor for Counter {
    #[inline]
    fn visit_input_primitive_buffer(&mut self, _: &Ident, _: &Primitive) {
        self.input_buffers += 1;
    }

    #[inline]
    fn visit_input_struct_buffer(&mut self, _: &Ident, _: &StructInner) {
        self.input_buffers += 1;
    }

    #[inline]
    fn visit_input_primitive(&mut self, _: &Ident, _: &Primitive) {
        self.has_bundled_input = true;
    }

    #[inline]
    fn visit_input_small_struct(&mut self, _: &Ident, _: &StructInner) {
        self.has_bundled_input = true;
    }

    #[inline]
    fn visit_input_big_struct(&mut self, _: &Ident, _: &StructInner) {
        self.input_buffers += 1;
    }

    #[inline]
    fn visit_input_object(&mut self, _: &Ident, _: Option<&str>) {
        self.input_objects += 1;
    }

    #[inline]
    fn visit_output_primitive_buffer(&mut self, _: &Ident, _: &Primitive) {
        self.output_buffers += 1;
    }

    #[inline]
    fn visit_output_struct_buffer(&mut self, _: &Ident, _: &StructInner) {
        self.output_buffers += 1;
    }

    #[inline]
    fn visit_output_primitive(&mut self, _: &Ident, _: &Primitive) {
        self.has_bundled_output = true;
    }
    #[inline]
    fn visit_output_small_struct(&mut self, _: &Ident, _: &StructInner) {
        self.has_bundled_output = true;
    }

    #[inline]
    fn visit_output_big_struct(&mut self, _: &Ident, _: &StructInner) {
        self.output_buffers += 1;
    }

    #[inline]
    fn visit_output_object(&mut self, _: &Ident, _: Option<&str>) {
        self.output_objects += 1;
    }
}

impl Counter {
    #[must_use]
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        super::functions::visit_params(function, &mut me);

        me.input_buffers += me.has_bundled_input.then_some(1).unwrap_or_default();
        me.output_buffers += me.has_bundled_output.then_some(1).unwrap_or_default();

        me
    }

    #[inline]
    #[must_use]
    pub const fn total(&self) -> u32 {
        (self.input_buffers + self.input_objects + self.output_buffers + self.output_objects) as u32
    }
}
