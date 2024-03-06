use idlc_mir::{Ident, Primitive, StructInner};

use crate::interface::variable_names::invoke::CONST;
use crate::types::change_primitive;

#[derive(Debug, Clone, Default)]
pub struct Signature {
    inputs: Vec<(String, String)>,
    outputs: Vec<(String, String)>,
    bundled_inputs: Vec<String>,
    bundled_outputs: Vec<String>,
    input_obj_arg: Vec<String>,

    total_bundled_input: u8,
    total_bundled_output: u8,
    is_no_typed_objects: bool,
}

pub fn iter_to_string(iter: impl Iterator<Item = impl AsRef<str>>) -> String {
    let mut acc = String::new();
    for item in iter {
        acc.push(',');
        acc.push(' ');
        acc += item.as_ref();
    }

    acc
}

pub fn addition_iter_to_string(iter: impl Iterator<Item = impl AsRef<str>>) -> String {
    let mut acc = String::new();
    for item in iter {
        acc.push('+');
        acc += item.as_ref();
    }

    acc
}

impl Signature {
    pub fn new(
        function: &idlc_mir::Function,
        counts: &idlc_codegen::counts::Counter,
        is_no_typed_objects: bool,
    ) -> Self {
        let mut me = Self {
            inputs: vec![],
            outputs: vec![],
            bundled_inputs: vec![],
            bundled_outputs: vec![],
            input_obj_arg: vec![],
            total_bundled_input: counts.total_bundled_input,
            total_bundled_output: counts.total_bundled_output,
            is_no_typed_objects,
        };

        let packed_primitives = idlc_codegen::serialization::PackedPrimitives::new(function);
        let packed_input = packed_primitives.input_idents();
        let packed_output = packed_primitives.output_idents();
        me.bundled_inputs = packed_input.map(|ident| ident.to_string()).collect();
        me.bundled_outputs = packed_output.map(|ident| ident.to_string()).collect();

        idlc_codegen::functions::visit_params(function, &mut me);

        me
    }

    pub fn input_obj_buffer(&self) -> impl Iterator<Item = &String> + '_ {
        self.input_obj_arg.iter()
    }

    pub fn params(&self) -> impl Iterator<Item = String> + '_ {
        self.inputs
            .iter()
            .map(|(ident, ty)| format!("{ty} {ident}"))
    }

    #[inline]
    pub fn return_idents(&self) -> impl Iterator<Item = &str> {
        self.outputs.iter().map(|(ident, _)| ident.as_str())
    }
}

impl idlc_codegen::functions::ParameterVisitor for Signature {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: Primitive) {
        let name = format!("*{}_ptr", ident);
        let ty = format!("{CONST} {}", change_primitive(ty));
        self.inputs.push((name, ty));
        self.inputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_ptr", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_len", ident), "size_t".to_string()));
    }

    fn visit_input_untyped_buffer(&mut self, ident: &Ident) {
        let ty = "void".to_string();
        let name = format!("*{}_ptr", ident);
        let ty = format!("{CONST} {}", ty);
        self.inputs.push((name, ty));
        self.inputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_ptr", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_len", ident), "size_t".to_string()));
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &StructInner) {
        let name = format!("*{}_ptr", ident);
        let ty = format!("{CONST} {}", ty.ident);
        self.inputs.push((name, ty));
        self.inputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_ptr", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_len", ident), "size_t".to_string()));
    }

    fn visit_input_object_array(&mut self, ident: &Ident, ty: Option<&str>, cnt: idlc_mir::Count) {
        let name = format!("(*{}_ptr)[{cnt}]", ident);
        let ty = if self.is_no_typed_objects {
            "Object".to_string()
        } else {
            format!("{CONST} {}", ty.unwrap_or("Object"))
        };
        self.inputs.push((name, ty));
        self.input_obj_arg.push(format!("{}_len", ident));
        self.outputs
            .push((format!("&{}", ident), "size_t".to_string()));
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: Primitive) {
        self.inputs
            .push((format!("{}_val", ident), change_primitive(ty).to_string()));
        if self.total_bundled_input > 1
            && self.bundled_inputs.contains(&ident.to_string())
            && self.bundled_inputs.len() > 1
        {
            self.outputs
                .push((format!("i->m_{}", ident), change_primitive(ty).to_string()));
        } else {
            self.outputs
                .push((format!("*{}_ptr", ident), change_primitive(ty).to_string()));
        }
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.inputs
            .push((format!("*{}_ptr", ident), format!("{CONST} {}", ty.ident)));
        if self.total_bundled_input > 1
            && self.bundled_inputs.contains(&ident.to_string())
            && self.bundled_inputs.len() > 1
        {
            self.outputs
                .push((format!("&i->m_{}", ident), format!("{CONST} {}", ty.ident)));
        } else if ty.contains_interfaces() {
            self.outputs
                .push((format!("&{}_ptr", ident), format!("{CONST} {}", ty.ident)));
        } else {
            self.outputs
                .push((format!("{}_ptr", ident), format!("{CONST} {}", ty.ident)));
        }
    }
    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.inputs
            .push((format!("*{}_ptr", ident), format!("{CONST} {}", ty.ident)));
        if self.total_bundled_input > 1
            && self.bundled_inputs.contains(&ident.to_string())
            && self.bundled_inputs.len() > 1
        {
            self.outputs
                .push((format!("&i->m_{}", ident), format!("{CONST} {}", ty.ident)));
        } else if ty.contains_interfaces() {
            self.outputs
                .push((format!("&{}_ptr", ident), format!("{CONST} {}", ty.ident)));
        } else {
            self.outputs
                .push((format!("{}_ptr", ident), format!("{CONST} {}", ty.ident)));
        }
    }

    fn visit_input_object(&mut self, ident: &Ident, ty: Option<&str>) {
        let ty = if self.is_no_typed_objects {
            "Object"
        } else {
            ty.unwrap_or("Object")
        };
        self.inputs.push((format!("{ident}"), ty.to_string()));
        self.outputs
            .push((format!("*{}_ptr", ident), ty.to_string()));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: Primitive) {
        let name = format!("*{}_ptr", ident);
        let ty = change_primitive(ty).to_string();
        self.inputs.push((name, ty));
        self.inputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.inputs
            .push((format!("*{}_lenout", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_ptr", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.outputs
            .push((format!("&{}_len", ident), "size_t".to_string()));
    }

    fn visit_output_untyped_buffer(&mut self, ident: &Ident) {
        let ty = "void".to_string();
        let name = format!("*{}_ptr", ident);
        self.inputs.push((name, ty.to_string()));
        self.inputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.inputs
            .push((format!("*{}_lenout", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_ptr", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.outputs
            .push((format!("&{}_len", ident), "size_t".to_string()));
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &StructInner) {
        let name = format!("*{}_ptr", ident);
        let ty = ty.ident.to_string();
        self.inputs.push((name, ty));
        self.inputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.inputs
            .push((format!("*{}_lenout", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_ptr", ident), "size_t".to_string()));
        self.outputs
            .push((format!("{}_len", ident), "size_t".to_string()));
        self.outputs
            .push((format!("&{}_len", ident), "size_t".to_string()));
    }

    fn visit_output_object_array(&mut self, ident: &Ident, ty: Option<&str>, cnt: idlc_mir::Count) {
        let name = format!("(*{}_ptr)[{cnt}]", ident);
        let ty = if self.is_no_typed_objects {
            "Object".to_string()
        } else {
            ty.unwrap_or("Object").to_string()
        };
        self.inputs.push((name, ty));
        self.outputs
            .push((format!("&{}", ident), "size_t".to_string()));
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: Primitive) {
        self.inputs
            .push((format!("*{}_ptr", ident), change_primitive(ty).to_string()));
        if self.total_bundled_output > 1
            && self.bundled_outputs.contains(&ident.to_string())
            && self.bundled_outputs.len() > 1
        {
            self.outputs
                .push((format!("&o->m_{}", ident), change_primitive(ty).to_string()));
        } else {
            self.outputs
                .push((format!("{}_ptr", ident), change_primitive(ty).to_string()));
        }
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.inputs
            .push((format!("*{}_ptr", ident), ty.ident.to_string()));
        if self.total_bundled_output > 1
            && self.bundled_outputs.contains(&ident.to_string())
            && self.bundled_outputs.len() > 1
        {
            self.outputs
                .push((format!("&o->m_{}", ident), ty.ident.to_string()));
        } else {
            self.outputs
                .push((format!("{}_ptr", ident), ty.ident.to_string()));
        }
    }
    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &StructInner) {
        self.inputs
            .push((format!("*{}_ptr", ident), ty.ident.to_string()));
        if self.total_bundled_output > 1
            && self.bundled_outputs.contains(&ident.to_string())
            && self.bundled_outputs.len() > 1
        {
            self.outputs
                .push((format!("&o->m_{}", ident), ty.ident.to_string()));
        } else {
            self.outputs
                .push((format!("{}_ptr", ident), ty.ident.to_string()));
        }
    }

    fn visit_output_object(&mut self, ident: &Ident, ty: Option<&str>) {
        let ty = if self.is_no_typed_objects {
            "Object"
        } else {
            ty.unwrap_or("Object")
        };
        self.inputs.push((format!("*{}", ident), ty.to_string()));
        self.outputs.push((format!("{}", ident), ty.to_string()));
    }
}
