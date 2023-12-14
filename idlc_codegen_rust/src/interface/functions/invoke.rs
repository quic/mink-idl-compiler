use crate::interface::mink_primitives::GENERIC_ERROR;
use crate::{
    interface::variable_names::invoke::{ARGS, BI_STRUCT, BO_STRUCT},
    types::change_primitive,
};

use crate::{ident::EscapedIdent, types::namespaced_struct};

use super::serialization::TransportBuffer;

#[derive(Debug, Default, Clone)]
pub struct Invoke {
    pre: Vec<String>,
    post: Vec<String>,

    idx: usize,
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
    fn idx(&mut self) -> usize {
        let idx = self.idx;
        self.idx += 1;

        idx
    }

    fn generate_for_input_buffer(&mut self, ident: EscapedIdent, ty: &str) {
        let idx = self.idx();
        self.pre.push(format!(
        r#"let {ident} = if {ARGS}[{idx}].bi.size == 0 {{
            &[]
        }} else {{
            std::slice::from_raw_parts({ARGS}[{idx}].bi.ptr.cast::<{ty}>(), {ARGS}[{idx}].bi.size / std::mem::size_of::<{ty}>())
        }};"#));
    }

    fn generate_for_output_buffer(&mut self, ident: EscapedIdent, ty: &str) {
        let idx = self.idx();
        self.pre.push(format!(
            r#"let {ident}_orig = {ARGS}[{idx}].b.size;
            let {ident}_lenout = &mut *std::ptr::addr_of_mut!({ARGS}[{idx}].b.size);"#
        ));
        self.pre.push(format!(
        r#"let {ident} = if {ident}_orig == 0 {{
            &mut []
        }} else {{
            std::slice::from_raw_parts_mut({ARGS}[{idx}].b.ptr.cast::<{ty}>(), {ident}_orig / std::mem::size_of::<{ty}>())
        }};"#));
        self.post.push(format!(
            r#"
            *{ident}_lenout = {ident}_lenout.saturating_mul(std::mem::size_of::<{ty}>());
            assert!(*{ident}_lenout <= {ident}_orig);
            "#
        ));
    }
}

impl idlc_codegen::functions::ParameterVisitor for Invoke {
    fn visit_input_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        self.generate_for_input_buffer(EscapedIdent::new(ident), change_primitive(ty));
    }

    fn visit_input_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.generate_for_input_buffer(EscapedIdent::new(ident), &namespaced_struct(ty));
    }

    fn visit_input_primitive(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        let ty: &str = change_primitive(ty);
        let ident = EscapedIdent::new(ident);
        let idx = self.idx();
        self.pre.push(format!(
            r#"if {ARGS}[{idx}].bi.size != std::mem::size_of::<{ty}>() {{
            return std::mem::transmute({GENERIC_ERROR}::INVALID);
        }}"#
        ));

        self.pre.push(format!(
            "let {ident} = *{ARGS}[{idx}].bi.ptr.cast::<{ty}>();"
        ));
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bi_definition() else {
            unreachable!()
        };
        let idx = self.idx();
        let idents = super::signature::iter_to_string(packer.bi_definition_idents());
        self.pre.push(definition);
        self.pre.push(format!(
            r#"
            if {ARGS}[{idx}].bi.size != {size} {{
                return std::mem::transmute({GENERIC_ERROR}::INVALID);
            }}
            "#,
        ));
        self.pre.push(format!(
            "let {BI_STRUCT}({idents}) = std::ptr::read(args[0].bi.ptr.cast::<{BI_STRUCT}>());"
        ));
        self.pre.push(packer.post_bi_assignments());
    }

    fn visit_input_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let ty: &str = &namespaced_struct(ty);
        let ident = EscapedIdent::new(ident);
        let idx = self.idx();
        self.pre.push(format!(
            r#"if {ARGS}[{idx}].bi.size != std::mem::size_of::<{ty}>() {{
            return std::mem::transmute({GENERIC_ERROR}::INVALID);
        }}"#
        ));

        self.pre.push(format!(
            "let {ident} = &*{ARGS}[{idx}].bi.ptr.cast::<{ty}>();"
        ));
    }
    fn visit_input_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &idlc_mir::Ident, _: Option<&str>) {
        let idx = self.idx();
        let ident = EscapedIdent::new(ident);
        self.pre.push(format!(
            "let {ident} = std::mem::transmute({ARGS}[{idx}].o.as_ref());"
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        self.generate_for_output_buffer(EscapedIdent::new(ident), change_primitive(ty));
    }

    fn visit_output_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.generate_for_output_buffer(EscapedIdent::new(ident), &namespaced_struct(ty));
    }

    fn visit_output_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let ident = EscapedIdent::new(ident);
        let ty = &namespaced_struct(ty);
        let idx = self.idx();
        self.pre.push(format!(
            r#"if {ARGS}[{idx}].b.size != std::mem::size_of::<{ty}>() {{
            return std::mem::transmute({GENERIC_ERROR}::SIZE_OUT);
        }}"#
        ));
        self.post
            .push(format!("*{ARGS}[{idx}].b.ptr.cast::<{ty}>() = {ident};\n"));
    }
    fn visit_output_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_primitive(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        let ty = change_primitive(ty);
        let ident = EscapedIdent::new(ident);
        let idx = self.idx();
        self.pre.push(format!(
            r#"if {ARGS}[{idx}].b.size != std::mem::size_of::<{ty}>() {{
            return std::mem::transmute({GENERIC_ERROR}::SIZE_OUT);
        }}"#
        ));
        self.post
            .push(format!("*{ARGS}[{idx}].b.ptr.cast::<{ty}>() = {ident};\n"));
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bo_definition() else {
            unreachable!()
        };

        let idx = self.idx();
        self.pre.push(definition);
        self.pre.push(format!(
            r#"
            if {ARGS}[{idx}].b.size != {size} {{
                return std::mem::transmute({GENERIC_ERROR}::INVALID);
            }}
            "#,
        ));
        let idents = super::signature::iter_to_string(packer.bo_idents());
        self.post.push(format!(
            r#"
            std::ptr::write({ARGS}[{idx}].b.ptr.cast::<{BO_STRUCT}>(), {BO_STRUCT}({idents}));
            "#
        ));
    }

    fn visit_output_object(&mut self, ident: &idlc_mir::Ident, _: Option<&str>) {
        let idx = self.idx();
        let ident = EscapedIdent::new(ident);
        self.post.push(format!(
            "{ARGS}[{idx}].o = std::mem::ManuallyDrop::new(std::mem::transmute({ident}));\n"
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    signature: &super::signature::Signature,
    counts: idlc_codegen::counts::Counter,
) -> String {
    use crate::interface::mink_primitives::{ERROR_STRUCT, OK, PACK_COUNTS};
    use crate::interface::variable_names::invoke::{CONTEXT, COUNTS};

    let ident = &function.ident;
    let op_id = function.id;
    let n_args = counts.total();
    let counts = (
        counts.input_buffers,
        counts.output_buffers,
        counts.input_objects,
        counts.output_objects,
    );
    let params = Invoke::new(function);
    let pre = params.pre();
    let post = params.post();

    let params = super::signature::iter_to_string(signature.param_idents());
    let returns = super::signature::iter_to_string(signature.return_idents());

    format!(
        r#"
    {op_id} => {{
        if {COUNTS} != {PACK_COUNTS}{counts:?} {{
            return std::mem::transmute({GENERIC_ERROR}::GENERIC)
        }}
        let args = std::slice::from_raw_parts_mut({ARGS}, {n_args});

        {pre}

        match (*{CONTEXT}).inner.r#{ident}({params}) {{
            Ok(({returns})) => {{
                {post}
                {OK}
            }},
            Err(e) => {{
                i32::from({ERROR_STRUCT}::from(e))
            }}
        }}

    }}
    "#
    )
}
