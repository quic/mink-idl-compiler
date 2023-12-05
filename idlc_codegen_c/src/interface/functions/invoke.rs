use crate::{
    interface::variable_names::invoke::{ARGS, CONST, OP},
    types::change_primitive,
};

use super::serialization::TransportBuffer;

#[derive(Debug, Default, Clone)]
pub struct Invoke {
    args: Vec<String>,
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

    pub fn args(&self) -> String {
        let mut acc = String::new();
        for arg in &self.args {
            acc.push_str(
                r" || \
                    ",
            );
            acc += arg.as_ref();
        }
        acc
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
}

impl idlc_codegen::functions::ParameterVisitor for Invoke {
    fn visit_input_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let ty: &str = change_primitive(ty);
        let name = format!("*{}_ptr", ident);
        self.pre.push(format!(
            r#" \
                {CONST} {ty} {name} = ({CONST} {ty}*){ARGS}[{idx}].b.ptr; \
                size_t {ident}_len = {ARGS}[{idx}].b.size / sizeof({ty});"#
        ));
    }

    fn visit_input_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let ty: &str = ty.ident.as_ref();
        let name = format!("*{}_ptr", ident);
        self.pre.push(format!(
            r#" \
                {CONST} {ty} {name} = ({CONST} {ty}*){ARGS}[{idx}].b.ptr; \
                size_t {ident}_len = {ARGS}[{idx}].b.size / sizeof({ty});"#
        ));
    }

    fn visit_input_primitive(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let ty: &str = change_primitive(ty);
        let name = format!("*{}_ptr", ident);
        self.args
            .push(format!("{ARGS}[{idx}].b.size != sizeof({ty})"));
        self.pre.push(format!(
            r#" \
                {CONST} {ty} {name} = ({CONST} {ty}*){ARGS}[{idx}].b.ptr;"#
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
        let idx = self.idx();
        self.args.push(format!("{ARGS}[{idx}].b.size != {size}"));
        self.pre.push(format!(
            r#" \
                {CONST} {definition} *i = {ARGS}[{idx}].b.ptr;"#
        ))
    }

    fn visit_input_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("*{}_ptr", ident);
        let sz = ty.size();
        let ty = ty.ident.to_string();
        self.args.push(format!("{ARGS}[{idx}].b.size != {sz}"));
        self.pre.push(format!(
            r#" \
                {CONST} {ty} {name} = ({CONST} {ty}*){ARGS}[{idx}].b.ptr;"#
        ));
    }
    fn visit_input_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &idlc_mir::Ident, ty: Option<&str>) {
        let idx = self.idx();
        let ty = ty.unwrap_or("Object").to_string();
        self.pre.push(format!(
            r#" \
                {ty} {ident}_ptr = ({ty}){ARGS}[{idx}].o;"#
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let ty: &str = change_primitive(ty);
        let name = format!("*{}_ptr", ident);
        self.pre.push(format!(
            r#" \
                {ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr; \
                size_t {ident}_len = {ARGS}[{idx}].b.size / sizeof({ty});"#
        ));
        self.post.push(format!(
            r#"\
            {ARGS}[{idx}].b.size = {ident}_len * sizeof({ty});"#
        ));
    }

    fn visit_output_struct_buffer(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let ty: &str = ty.ident.as_ref();
        let name = format!("*{}_ptr", ident);
        self.pre.push(format!(
            r#" \
                {ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr; \
                size_t {ident}_len = {ARGS}[{idx}].b.size / sizeof({ty});"#
        ));
        self.post.push(format!(
            r#" \
            {ARGS}[{idx}].b.size = {ident}_len * sizeof({ty});"#
        ));
    }

    fn visit_output_big_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        let idx = self.idx();
        let name = format!("*{}_ptr", ident);
        let sz = ty.size();
        let ty = ty.ident.to_string();
        self.args.push(format!("{ARGS}[{idx}].b.size != {sz}"));
        self.pre.push(format!(
            r#" \
                {ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr;"#
        ));
    }
    fn visit_output_small_struct(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_primitive(&mut self, ident: &idlc_mir::Ident, ty: &idlc_mir::Primitive) {
        let idx = self.idx();
        let ty: &str = change_primitive(ty);
        let name = format!("*{}_ptr", ident);
        self.args
            .push(format!("{ARGS}[{idx}].b.size != sizeof({ty})"));
        self.pre.push(format!(
            r#" \
                {ty} {name} = ({ty}*){ARGS}[{idx}].b.ptr;"#
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
        let idx = self.idx();
        self.args.push(format!("{ARGS}[{idx}].b.size != {size}"));
        self.pre.push(format!(
            r#" \
                {definition} *o = {ARGS}[{idx}].b.ptr;"#
        ))
    }

    fn visit_output_object(&mut self, ident: &idlc_mir::Ident, ty: Option<&str>) {
        let idx = self.idx();
        let ty = ty.unwrap_or("Object").to_string();
        self.pre.push(format!(
            r#" \
                {ty} {ident}_ptr = ({ty}){ARGS}[{idx}].o;"#
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    iface_ident: &str,
    documentation: &str,
    signature: &super::signature::Signature,
    counts: &idlc_codegen::counts::Counter,
) -> String {
    let ident = &function.ident;
    let invoke = Invoke::new(function);
    let pre = invoke.pre();
    let post = invoke.post();
    let args = invoke.args();

    let return_idents = super::signature::iter_to_string(signature.return_idents());

    let counts = (
        counts.input_buffers,
        counts.output_buffers,
        counts.input_objects,
        counts.output_objects,
    );

    format!(
        r#" \
            {documentation} \
            case {iface_ident}_{OP}_{ident}: {{ \
                if (k != ObjectCounts_pack{counts:?}{args}) {{ \
                    break; \
                }} \
                {pre} \
                int32_t r = prefix##{ident}(me{return_idents}); \
                {post} \
                return r; \
            }} "#
    )
}