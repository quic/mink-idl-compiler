use idlc_mir::{Ident, Primitive, Struct};

#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub input_buffers: u8,
    pub input_objects: u8,
    pub output_objects: u8,
    pub output_buffers: u8,

    has_primitive_input: bool,
    has_primitive_output: bool,
}

impl super::functions::ParameterVisitor for Counter {
    #[inline]
    fn visit_input_primitive_buffer(&mut self, _: &Ident, _: &Primitive) {
        self.input_buffers += 1;
    }

    #[inline]
    fn visit_input_struct_buffer(&mut self, _: &Ident, _: &Struct) {
        self.input_buffers += 1;
    }

    #[inline]
    fn visit_input_primitive(&mut self, _: &Ident, _: &Primitive) {
        self.has_primitive_input = true;
    }

    #[inline]
    fn visit_input_struct(&mut self, _: &Ident, _: &Struct) {
        self.input_buffers += 1
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
    fn visit_output_struct_buffer(&mut self, _: &Ident, _: &Struct) {
        self.output_buffers += 1;
    }

    #[inline]
    fn visit_output_primitive(&mut self, _: &Ident, _: &Primitive) {
        self.has_primitive_output = true;
    }

    #[inline]
    fn visit_output_struct(&mut self, _: &Ident, _: &Struct) {
        self.output_buffers += 1;
    }

    #[inline]
    fn visit_output_object(&mut self, _: &Ident, _: Option<&str>) {
        self.output_objects += 1;
    }
}

impl Counter {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        super::functions::visit_params(function, &mut me);

        me.input_buffers += me.has_primitive_input.then_some(1).unwrap_or_default();
        me.output_buffers += me.has_primitive_output.then_some(1).unwrap_or_default();

        me
    }

    #[inline]
    pub const fn total(&self) -> u32 {
        (self.input_buffers + self.input_objects + self.output_buffers + self.output_objects) as u32
    }
}
