// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::{Interface, InterfaceNode};

pub mod functions;
pub mod variable_names;

use crate::types::change_const_primitive;
use variable_names::invoke::{ARGS, CONTEXT, COUNTS, INDENT, OP_CODE};

pub fn emit_interface_impl(interface: &Interface, is_no_typed_objects: bool) -> String {
    let ident = interface.ident.to_string();

    let mut constants = String::new();
    let mut errors = String::new();
    let mut op_codes = String::new();
    let mut implementations = String::new();

    // A closure to hold logic for the base class(es) and the root class
    let mut process_intf_node = |node: &InterfaceNode, prefix: &str, is_root: bool| match node {
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
            let signature = functions::signature::Signature::new(f, &counts, is_no_typed_objects);
            let documentation = idlc_codegen::documentation::Documentation::new(
                f,
                idlc_codegen::documentation::DocumentationStyle::C,
            );
            if is_root {
                op_codes.push_str(&format!("#define {}_OP_{} {}\n", ident, f.ident, f.id));
            }
            implementations.push_str(&functions::implementation::emit(
                f,
                &ident,
                prefix,
                &documentation,
                &counts,
                &signature,
            ));
        }
    };

    // Create an iterator over all base class(es) nodes which generates a tuple
    // for each element to prepare for the common closure
    let iter_base_nodes = interface
        .iter()
        .skip(1)
        .flat_map(|iface| iface.nodes.iter().map(|n| (n, &iface.ident)))
        .map(|(node, base_ident)| (node, base_ident, false));

    // Create an iterator over root class nodes which generates a tuple for each
    // element to prepare for the common closure
    let iter_root_nodes = interface
        .nodes
        .iter()
        .map(|node| (node, &interface.ident, true));

    // For all base class nodes AND THEN root class nodes,
    for (i_node, id, is_root_node) in iter_base_nodes.chain(iter_root_nodes) {
        // process the node with a closure
        process_intf_node(i_node, id, is_root_node);
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
{INDENT}return Object_invoke(self, Object_OP_release, 0, 0);
}}

static inline int32_t
{ident}_retain(Object self)
{{
{INDENT}return Object_invoke(self, Object_OP_retain, 0, 0);
}}
{implementations}
"#
    )
}

pub fn emit_interface_invoke(interface: &Interface, is_no_typed_objects: bool) -> String {
    let ident = interface.ident.to_string();

    let mut invokes = String::new();
    let mut weak_declarations = String::new();

    // need to have all of the base-class functions
    interface.iter().skip(1).for_each(|iface| {
        iface.nodes.iter().for_each(|node| {
            if let InterfaceNode::Function(f) = node {
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature =
                    functions::signature::Signature::new(f, &counts, is_no_typed_objects);
                invokes.push_str(&functions::invoke::emit(
                    f,
                    &mut weak_declarations,
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
                &mut weak_declarations,
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

    // weak_delcarations goes inside `func` to preserve the expected when the macro is instantiated
    // with `static IFoo_DEFINE_INVOKE`. It is OK to declare functions within functions for C.
    format!(
        r#"{typed_objects}
#ifdef __clang__
#define __compiler_pragma_pre \
    _Pragma("clang diagnostic push") \
    _Pragma("clang diagnostic ignored \"-Wignored-attributes\"")
#define __compiler_pragma_post _Pragma("clang diagnostic pop")
#else
#define __compiler_pragma_pre
#define __compiler_pragma_post
#endif

#define {ident}_DEFINE_INVOKE(func, prefix, type) \
    int32_t func(ObjectCxt {CONTEXT}, ObjectOp {OP_CODE}, ObjectArg *{ARGS}, ObjectCounts {COUNTS}) \
    {{ \
        __compiler_pragma_pre \
        {weak_declarations} \
        __compiler_pragma_post \
        type me = (type) {CONTEXT}; \
        switch (ObjectOp_methodID({OP_CODE})) {{ \
            case Object_OP_release: {{ \
                if ({COUNTS} != ObjectCounts_pack(0, 0, 0, 0)) {{ \
                    break; \
                }} \
                return prefix##release(me); \
            }} \
            case Object_OP_retain: {{ \
                if ({COUNTS} != ObjectCounts_pack(0, 0, 0, 0)) {{ \
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
