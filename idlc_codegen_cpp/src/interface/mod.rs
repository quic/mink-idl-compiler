// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_codegen::keywords::invoke::VERSION_FUNC_NAME;
use idlc_mir::{APIVersion, Interface, InterfaceNode};

mod functions;

use idlc_codegen_c::interface::variable_names::invoke::{ARGS, COUNTS, INDENT, OP_CODE, OP_PREFIX};
use idlc_codegen_c::types::{change_const_primitive, change_primitive};

pub fn emit_interface_impl(interface: &Interface) -> String {
    let ident = interface.ident.to_string();

    let mut base_iface = String::new();
    let mut constants = String::new();
    let mut errors = String::new();
    let mut op_codes = String::new();
    let mut func_titles = String::new();
    let mut implementations = String::new();

    // A closure to hold logic for the base class(es) and the root class
    let mut process_intf_node = |node: &InterfaceNode, is_root: bool| match node {
        InterfaceNode::Const(c) => {
            constants.push_str(&format!(
                r#"
    static constexpr {} {} = {}({});"#,
                change_primitive(c.r#type),
                c.ident,
                change_const_primitive(c.r#type),
                c.value
            ));
        }
        InterfaceNode::Error(e) => {
            errors.push_str(&format!(
                r#"
    static constexpr int32_t {} = INT32_C({});"#,
                e.ident, e.value
            ));
        }
        InterfaceNode::Function(f) => {
            let counts = idlc_codegen::counts::Counter::new(f);
            let signature = functions::signature::Signature::new(f, &counts);
            let documentation = idlc_codegen::documentation::Documentation::new(
                f,
                idlc_codegen::documentation::DocumentationStyle::C,
            );
            if is_root {
                let mut params = idlc_codegen_c::interface::functions::signature::iter_to_string(
                    signature.params(),
                );
                if !params.is_empty() {
                    params.remove(0);
                }
                func_titles.push_str(&format!(
                    r#"
    virtual int32_t {}({}) = 0;"#,
                    f.ident, params,
                ));
                op_codes.push_str(&format!(
                    r#"
    static constexpr ObjectOp {OP_PREFIX}_{} = {};"#,
                    f.ident, f.id,
                ));
            }
            implementations.push_str(&functions::implementation::emit(
                f,
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
        .flat_map(|iface| {
            base_iface.push_str(&format!("I{} ", &iface.ident.to_string()));
            iface.nodes.iter()
        })
        .map(|node| (node, false));

    // Create an iterator over root class nodes which generates a tuple for each
    // element to prepare for the common closure
    let iter_root_nodes = interface.nodes.iter().map(|node| (node, true));

    // For all base class nodes AND THEN root class nodes,
    for (i_node, is_root_node) in iter_base_nodes.chain(iter_root_nodes) {
        // process the node with a closure
        process_intf_node(i_node, is_root_node);
    }

    if !base_iface.is_empty() {
        let first_base_iface = base_iface.split_whitespace().next().unwrap();
        base_iface = format!(": public {first_base_iface} ");
    }

    let interface_version = interface.get_version();

    format!(
        r#"
// '{ident}' interface at version '{interface_version}'
class {ident};
class I{ident} {base_iface}{{
  public:{constants}
    static constexpr uint16_t PATCH_MASK  = 0x0FFF; /* 12 bits */
    static constexpr uint16_t MINOR_MASK  = 0x03FF; /* 10 bits */
    static constexpr uint16_t MAJOR_MASK  = 0x03FF; /* 10 bits */
    static constexpr uint16_t MINOR_SHIFT = UINT8_C(12);
    static constexpr uint16_t MAJOR_SHIFT = MINOR_SHIFT + UINT8_C(10);
{errors}

    virtual ~I{ident}() {{}}
{func_titles}

  protected:
{op_codes}
}};

class {ident} : public I{ident}, public ProxyBase {{
  public:
    {ident}() {{}}
    {ident}(Object impl) : ProxyBase(impl) {{}}
    virtual ~{ident}() {{}}
    virtual int32_t {VERSION_FUNC_NAME}(uint32_t *version_ptr) {{
        ObjectArg a[] = {{
            {{.b = (ObjectBuf) {{ version_ptr, sizeof(uint32_t) }} }},
        }};
        return invoke(Object_OP_version, a, ObjectCounts_pack(0, 1, 0, 0));
    }}
{implementations}
}};

"#
    )
}

pub fn emit_interface_invoke(interface: &Interface) -> String {
    let ident = interface.ident.to_string();

    let mut invokes = String::new();

    // need to have all of the base-class functions
    interface.iter().skip(1).for_each(|iface| {
        iface.nodes.iter().for_each(|node| {
            if let InterfaceNode::Function(f) = node {
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature = functions::signature::Signature::new(f, &counts);

                invokes.push_str(&functions::invoke::emit(f, &signature, &counts));
            }
        })
    });

    for node in &interface.nodes {
        if let InterfaceNode::Function(f) = node {
            let counts = idlc_codegen::counts::Counter::new(f);
            let signature = functions::signature::Signature::new(f, &counts);

            invokes.push_str(&functions::invoke::emit(f, &signature, &counts));
        }
    }

    let APIVersion { major, minor } = interface.get_version();

    format!(
        r#"
class {ident}ImplBase : protected ImplBase, public I{ident} {{
  public:
    {ident}ImplBase() {{}}
    virtual ~{ident}ImplBase() {{}}
    static constexpr uint16_t VERSION_MAJOR = UINT16_C({major});
    static constexpr uint16_t VERSION_MINOR = UINT16_C({minor});
    static constexpr uint16_t VERSION_PATCH = 0;
  protected:
{INDENT}virtual int32_t invoke(ObjectOp {OP_CODE}, ObjectArg* {ARGS}, ObjectCounts {COUNTS}) {{
{INDENT}{INDENT}switch (ObjectOp_methodID({OP_CODE})) {{
{INDENT}{INDENT}{INDENT}case Object_OP_version: {{
{INDENT}{INDENT}{INDENT}{INDENT}if (k != ObjectCounts_pack(0, 1, 0, 0) || a[0].b.size != 4){{
{INDENT}{INDENT}{INDENT}{INDENT}{INDENT}break;
{INDENT}{INDENT}{INDENT}{INDENT}}}
{INDENT}{INDENT}{INDENT}{INDENT}uint32_t *version_ptr = (uint32_t*)a[0].b.ptr;
{INDENT}{INDENT}{INDENT}{INDENT}*version_ptr = ((VERSION_MAJOR & MAJOR_MASK) << MAJOR_SHIFT) | \
{INDENT}{INDENT}{INDENT}{INDENT}               ((VERSION_MINOR & MINOR_MASK) << MINOR_SHIFT) | \
{INDENT}{INDENT}{INDENT}{INDENT}                (VERSION_PATCH & PATCH_MASK);
{INDENT}{INDENT}{INDENT}{INDENT}return Object_OK;
{INDENT}{INDENT}{INDENT}}}
{invokes}
{INDENT}{INDENT}{INDENT}default: {{ return Object_ERROR_INVALID; }}
{INDENT}{INDENT}}}
{INDENT}{INDENT}return Object_ERROR_INVALID;
{INDENT}}}
}};
"#
    )
}
