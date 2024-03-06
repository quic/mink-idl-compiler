use idlc_mir::{Interface, InterfaceNode};

pub mod functions;
pub mod variable_names;

use crate::types::change_const_primitive;

pub fn emit_interface_impl(interface: &Interface, is_no_typed_objects: bool) -> String {
    let ident = interface.ident.to_string();

    let mut constants = String::new();
    let mut errors = String::new();
    let mut op_codes = String::new();
    let mut implementations = String::new();

    // need to have all of the base-class functions, error-codes and const values
    interface.iter().skip(1).for_each(|iface| {
        iface.nodes.iter().for_each(|node| match node {
            InterfaceNode::Const(c) => {
                constants.push_str(&format!(
                    "#define {}_{} {}({})\n",
                    ident,
                    c.ident,
                    change_const_primitive(c.r#type),
                    c.value
                ));
            }
            InterfaceNode::Error(e) => {
                errors.push_str(&format!(
                    "#define {}_{} INT32_C({})\n",
                    ident, e.ident, e.value
                ));
            }
            InterfaceNode::Function(f) => {
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature =
                    functions::signature::Signature::new(f, &counts, is_no_typed_objects);
                let documentation = idlc_codegen::documentation::Documentation::new(
                    f,
                    idlc_codegen::documentation::DocumentationStyle::C,
                );

                implementations.push_str(&functions::implementation::emit(
                    f,
                    &ident,
                    &iface.ident,
                    &documentation,
                    &counts,
                    &signature,
                ));
            }
        })
    });

    for node in &interface.nodes {
        match node {
            InterfaceNode::Const(c) => {
                constants.push_str(&format!(
                    "#define {}_{} {}({})\n",
                    ident,
                    c.ident,
                    change_const_primitive(c.r#type),
                    c.value
                ));
            }
            InterfaceNode::Error(e) => {
                errors.push_str(&format!(
                    "#define {}_{} INT32_C({})\n",
                    ident, e.ident, e.value
                ));
            }
            InterfaceNode::Function(f) => {
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature =
                    functions::signature::Signature::new(f, &counts, is_no_typed_objects);
                let documentation = idlc_codegen::documentation::Documentation::new(
                    f,
                    idlc_codegen::documentation::DocumentationStyle::C,
                );

                op_codes.push_str(&format!("#define {}_OP_{} {}\n", ident, f.ident, f.id));
                implementations.push_str(&functions::implementation::emit(
                    f,
                    &ident,
                    &ident,
                    &documentation,
                    &counts,
                    &signature,
                ));
            }
        }
    }
    let object_defined = if is_no_typed_objects {
        "".to_string()
    } else {
        format!("typedef Object {ident};")
    };

    format!(
        r#"
{object_defined}
{constants}
{errors}
{op_codes}
static inline int32_t
{ident}_release(Object self)
{{
    return Object_invoke(self, Object_OP_release, 0, 0);
}}

static inline int32_t
{ident}_retain(Object self)
{{
    return Object_invoke(self, Object_OP_retain, 0, 0);
}}
{implementations}
"#
    )
}

pub fn emit_interface_invoke(interface: &Interface, is_no_typed_objects: bool) -> String {
    let ident = interface.ident.to_string();

    let mut invokes = String::new();

    // need to have all of the base-class functions
    interface.iter().skip(1).for_each(|iface| {
        iface.nodes.iter().for_each(|node| {
            if let InterfaceNode::Function(f) = node {
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature =
                    functions::signature::Signature::new(f, &counts, is_no_typed_objects);

                invokes.push_str(&functions::invoke::emit(
                    f,
                    &iface.ident,
                    &signature,
                    &counts,
                    is_no_typed_objects,
                ));
            }
        })
    });

    for node in &interface.nodes {
        if let InterfaceNode::Function(f) = node {
            let counts = idlc_codegen::counts::Counter::new(f);
            let signature = functions::signature::Signature::new(f, &counts, is_no_typed_objects);

            invokes.push_str(&functions::invoke::emit(
                f,
                &ident,
                &signature,
                &counts,
                is_no_typed_objects,
            ));
        }
    }

    let typed_objects = (!is_no_typed_objects)
        .then_some(format!(r#"typedef Object {ident};"#))
        .unwrap_or_default();

    format!(
        r#"{typed_objects}
#define {ident}_DEFINE_INVOKE(func, prefix, type) \
    int32_t func(ObjectCxt h, ObjectOp op, ObjectArg *a, ObjectCounts k) \
    {{ \
        type me = (type) h; \
        switch (ObjectOp_methodID(op)) {{ \
            case Object_OP_release: {{ \
                if (k != ObjectCounts_pack(0, 0, 0, 0)) {{ \
                    break; \
                }} \
                return prefix##release(me); \
            }} \
            case Object_OP_retain: {{ \
                if (k != ObjectCounts_pack(0, 0, 0, 0)) {{ \
                    break; \
                }} \
                return prefix##retain(me); \
            }} \
            {invokes} \
        }} \
        return Object_ERROR_INVALID; \
    }}
"#
    )
}
