use idlc_codegen_c::interface::variable_names::invoke::{ARGS, OP};

use idlc_mir::Ident;

#[derive(Debug, Default, Clone)]
pub struct Invoke(idlc_codegen_c::interface::functions::invoke::Invoke);

impl Invoke {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();
        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);

        me
    }
}

impl idlc_codegen::functions::ParameterVisitor for Invoke {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_input_primitive_buffer(ident, ty);
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_input_struct_buffer(ident, ty);
    }

    fn visit_input_object_buffer(
        &mut self,
        ident: &idlc_mir::Ident,
        ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let ty = ty.unwrap_or("ProxyBase").to_string();
        let mut obj_args = String::new();
        for _ in 0..cnt.into() {
            let idx = self.0.idx();
            obj_args.push_str(&format!(r"a[{idx}].o,"));
        }
        self.0.pre.push(format!(
            r#" \
                const union {ident} {{
                    ~{ident}() {{}}
                    {ty} inner[{cnt}];
                }} {ident} = {{.inner= {{ {obj_args} }}}};"#,
        ));
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_input_primitive(ident, ty);
    }

    fn visit_input_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        self.0.visit_input_bundled(packed_primitives);
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_input_big_struct(ident, ty);
    }

    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, ty: Option<&str>) {
        let idx = self.0.idx();
        let ty = ty.unwrap_or("ProxyBase").to_string();
        self.0.pre.push(format!(
            r#" \
                {ty} p_{ident}({ARGS}[{idx}].o);"#
        ));
        self.0.post.push(format!(
            r#" \
            p_{ident}.extract();"#
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: idlc_mir::Primitive) {
        self.0.visit_output_primitive_buffer(ident, ty);
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_output_struct_buffer(ident, ty);
    }

    fn visit_output_object_buffer(
        &mut self,
        ident: &idlc_mir::Ident,
        ty: Option<&str>,
        cnt: idlc_mir::Count,
    ) {
        let ty = ty.unwrap_or("ProxyBase").to_string();
        let idx = self.0.idx();
        self.0.pre.push(format!(
            r#" \
                {ty} p_{ident}[{cnt}];"#
        ));
        self.0.post.push(format!(
            r#" \
            for(size_t arg_idx=0;arg_idx<{cnt};arg_idx++) \
                {ARGS}[{idx}+arg_idx].o=p_{ident}[arg_idx].extract();"#,
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
        self.0.visit_output_bundled(packed_primitives);
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.0.visit_output_big_struct(ident, ty);
    }

    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_object(&mut self, ident: &Ident, ty: Option<&str>) {
        let idx = self.0.idx();
        let ty = ty.unwrap_or("ProxyBase").to_string();
        self.0.pre.push(format!(
            r#" \
                {ty} p_{ident};"#
        ));
        self.0.post.push(format!(
            r#" \
            {ARGS}[{idx}].o = p_{ident}.extract();"#
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    signature: &super::signature::Signature,
    counts: &idlc_codegen::counts::Counter,
) -> String {
    let ident = &function.ident;

    let invoke = Invoke::new(function);
    let mut len_intialize = invoke.0.len_intialize();
    let mut pre = invoke.0.pre();
    let mut post = invoke.0.post();
    let mut args = invoke.0.args();

    len_intialize = len_intialize.replace(" \\", "");
    args = args.replace(" \\", "");
    pre = pre.replace(" \\", "");
    post = post.replace(" \\\n", "\n    ");

    let mut return_idents =
        idlc_codegen_c::interface::functions::signature::iter_to_string(signature.return_idents());
    if !return_idents.is_empty() {
        return_idents.remove(0);
    }

    let counts = format!(
        "({0}, {1}, {2}, {3})",
        counts.input_buffers, counts.output_buffers, counts.input_objects, counts.output_objects,
    );

    format!(
        r#"
            case {OP}_{ident}: {{
                {len_intialize}
                if (k != ObjectCounts_pack{counts}{args}) {{
                    break;
                }}
                {pre}
                int32_t r = {ident}({return_idents});
                {post}
                return r;
            }} "#
    )
}
