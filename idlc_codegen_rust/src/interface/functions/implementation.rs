use idlc_mir::Ident;

use crate::interface::mink_primitives::{OK, PACK_COUNTS};
use crate::interface::variable_names::invoke::{ARGS, BI_STRUCT, BO_STRUCT};
use crate::types::change_primitive;

use crate::interface::mink_primitives::ARG;

use crate::ident::EscapedIdent;
use crate::types::namespaced_struct;

use super::serialization::TransportBuffer;
const INPUT_BUFFER: &str = crate::interface::mink_primitives::namespace!("BufIn");
const OUTPUT_BUFFER: &str = crate::interface::mink_primitives::namespace!("BufOut");

#[derive(Debug, Clone, Default)]
pub struct Implementation {
    initializations: Vec<String>,
    post_call: Vec<String>,
    args: Vec<String>,
}

impl Implementation {
    pub fn new(function: &idlc_mir::Function) -> Self {
        let mut me = Self::default();

        idlc_codegen::functions::visit_params_with_bundling(function, &mut me);

        me
    }

    pub fn args(&self) -> String {
        self.args.join(",")
    }

    pub fn initializations(&self) -> String {
        self.initializations.concat()
    }

    pub fn post_call_assignments(&self) -> String {
        self.post_call.concat()
    }
}

impl Implementation {
    fn generate_input_buffer(&mut self, ident: &Ident) {
        let ident = EscapedIdent::new(ident);
        self.args.push(format!(
            r#"{ARG} {{
                bi: {INPUT_BUFFER} {{
                    ptr: {ident}.as_ptr().cast(),
                    size: std::mem::size_of_val({ident})
                }}
            }}"#
        ));
    }

    fn generate_output_buffer(&mut self, ident: &Ident, ty: &str) {
        let ident = EscapedIdent::new(ident);
        self.post_call.push(format!(
            "*{ident}_lenout = unsafe {{ {ARGS}[{idx}].b.size }} / std::mem::size_of::<{ty}>();",
            idx = self.args.len()
        ));
        self.args.push(format!(
            r#"{ARG} {{
                b: {OUTPUT_BUFFER} {{
                    ptr: {ident}.as_mut_ptr().cast(),
                    size: std::mem::size_of_val({ident})
                }}
            }}"#
        ));
    }
}

impl idlc_codegen::functions::ParameterVisitor for Implementation {
    fn visit_input_primitive_buffer(&mut self, ident: &Ident, _: &idlc_mir::Primitive) {
        self.generate_input_buffer(ident);
    }

    fn visit_input_struct_buffer(&mut self, ident: &Ident, _: &idlc_mir::StructInner) {
        self.generate_input_buffer(ident);
    }

    fn visit_input_primitive(&mut self, ident: &Ident, ty: &idlc_mir::Primitive) {
        let ty: &str = change_primitive(ty);
        let ident = EscapedIdent::new(ident);
        self.args.push(format!(
            r#"{ARG} {{
                bi: {INPUT_BUFFER} {{
                    ptr: std::ptr::addr_of!({ident}).cast(),
                    size: std::mem::size_of::<{ty}>(),
                }}
            }}"#
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
        let idents = super::signature::iter_to_string(packer.bi_assignment_idents());
        self.initializations.push(format!(
            r#"
            {definition}
            let mut bi = {BI_STRUCT}({idents});
            "#
        ));
        self.args.push(format!(
            r#"{ARG} {{
                bi: {INPUT_BUFFER} {{
                    ptr: std::ptr::addr_of!(bi).cast(),
                    size: {size},
                }}
            }}
            "#
        ));
    }

    fn visit_input_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let ty: &str = &namespaced_struct(ty);
        let ident = EscapedIdent::new(ident);
        self.args.push(format!(
            r#"{ARG} {{
                bi: {INPUT_BUFFER} {{
                    ptr: ({ident} as *const {ty}).cast(),
                    size: std::mem::size_of::<{ty}>(),
                }}
            }}"#
        ));
    }
    fn visit_input_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_input_big_struct(ident, ty);
    }

    fn visit_input_object(&mut self, ident: &Ident, _: Option<&str>) {
        let ident = EscapedIdent::new(ident);
        self.args.push(format!(
            r#"{ARG} {{
                o: std::mem::ManuallyDrop::new({ident}.map(|o| unsafe {{ std::mem::transmute_copy(o) }} ))
            }}"#
        ));
    }

    fn visit_output_primitive_buffer(&mut self, ident: &Ident, ty: &idlc_mir::Primitive) {
        self.generate_output_buffer(ident, change_primitive(ty));
    }

    fn visit_output_struct_buffer(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.generate_output_buffer(ident, &namespaced_struct(ty));
    }

    fn visit_output_primitive(&mut self, ident: &Ident, ty: &idlc_mir::Primitive) {
        let ty: &str = change_primitive(ty);
        let ident = EscapedIdent::new(ident);
        self.initializations.push(format!(
            "let mut {ident} = std::mem::MaybeUninit::<{ty}>::uninit();\n"
        ));
        self.post_call
            .push(format!("let {ident} = unsafe {{ {ident}.assume_init() }};"));
        self.args.push(format!(
            r#"{ARG} {{
                bi: {INPUT_BUFFER} {{
                    ptr: std::ptr::addr_of_mut!({ident}).cast(),
                    size: std::mem::size_of::<{ty}>(),
                }}
            }}"#
        ));
    }

    fn visit_output_bundled(
        &mut self,
        packed_primitives: &idlc_codegen::serialization::PackedPrimitives,
    ) {
        let packer = super::serialization::PackedPrimitives::new(packed_primitives);
        let Some(TransportBuffer { definition, size }) = packer.bo_definition() else {
            unreachable!()
        };
        let idents = super::signature::iter_to_string(packer.bo_idents());
        self.initializations.push(format!(
            r#"
                {definition}
                let mut bo = std::mem::MaybeUninit::<{BO_STRUCT}>::uninit();
                "#
        ));
        self.post_call.push(format!(
            "let {BO_STRUCT}({idents}) = unsafe {{ bo.assume_init() }};"
        ));
        self.args.push(format!(
            r#"{ARG} {{
                b: {OUTPUT_BUFFER} {{
                    ptr: std::ptr::addr_of_mut!(bo).cast(),
                    size: {size},
                }}
            }}"#
        ));
    }

    fn visit_output_big_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        let ty: &str = &namespaced_struct(ty);
        let ident = EscapedIdent::new(ident);
        self.initializations.push(format!(
            "let mut {ident} = std::mem::MaybeUninit::<{ty}>::uninit();\n"
        ));
        self.post_call
            .push(format!("let {ident} = unsafe {{ {ident}.assume_init() }};"));
        self.args.push(format!(
            r#"{ARG} {{
                b: {OUTPUT_BUFFER} {{
                    ptr: std::ptr::addr_of_mut!({ident}).cast(),
                    size: std::mem::size_of::<{ty}>(),
                }}
            }}"#
        ));
    }
    fn visit_output_small_struct(&mut self, ident: &Ident, ty: &idlc_mir::StructInner) {
        self.visit_output_big_struct(ident, ty);
    }

    fn visit_output_object(&mut self, ident: &Ident, _: Option<&str>) {
        let idx = self.args.len();
        let ident = EscapedIdent::new(ident);
        self.post_call.push(format!(
            "let {ident} = unsafe {{ std::mem::transmute(std::mem::ManuallyDrop::take(&mut {ARGS}[{idx}].o)) }};"
        ));
        self.args.push(format!(
            r#"{ARG} {{
                o: std::mem::ManuallyDrop::new(None)
            }}"#
        ));
    }
}

pub fn emit(
    function: &idlc_mir::Function,
    documentation: &str,
    counts: idlc_codegen::counts::Counter,
    signature: &super::signature::Signature,
) -> String {
    let id = function.id;
    let ident = &function.ident;

    let implementation = Implementation::new(function);
    let initializations = implementation.initializations();
    let post_call_assignments = implementation.post_call_assignments();
    let args = implementation.args();

    let return_idents = super::signature::iter_to_string(signature.return_idents());
    let returns_types = super::signature::iter_to_string(signature.return_types());
    let params = super::signature::iter_to_string(signature.params());

    let counts = (
        counts.input_buffers,
        counts.output_buffers,
        counts.input_objects,
        counts.output_objects,
    );

    format!(
        r#"
        #[inline]
        {documentation}
        pub fn r#{ident}(&self, {params}) -> Result<({returns_types}), Error> {{
            {initializations}
            let mut {ARGS} = [{args}];

            match unsafe {{ self.0.invoke({id}, {ARGS}.as_mut_ptr(), {PACK_COUNTS}{counts:?}) }} {{
                {OK} => {{
                    {post_call_assignments}
                    Ok(({return_idents}))
                }},
                err => Err(unsafe {{ std::mem::transmute(err) }})
            }}
        }}
    "#
    )
}
