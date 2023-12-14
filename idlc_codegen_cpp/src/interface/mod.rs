use idlc_mir::{Interface, InterfaceNode};

mod functions;

use idlc_codegen_c::types::{change_const_primitive, change_primitive};

pub fn emit_interface_impl(interface: &Interface) -> String {
    let ident = interface.ident.to_string();

    let mut base_iface = String::new();
    let mut constants = String::new();
    let mut errors = String::new();
    let mut op_codes = String::new();
    let mut func_titles = String::new();
    let mut implementations = String::new();

    // need to have all of the base-class functions, error-codes and const values
    interface.iter().skip(1).for_each(|iface| {
        base_iface.push_str(&format!("I{} ", &iface.ident.to_string()));
        iface.nodes.iter().for_each(|node| match node {
            InterfaceNode::Const(c) => {
                constants.push_str(&format!(
                    r#"static const {} {} = {}({});
    "#,
                    change_primitive(c.r#type),
                    c.ident,
                    change_const_primitive(c.r#type),
                    c.value
                ));
            }
            InterfaceNode::Error(e) => {
                errors.push_str(&format!(
                    r#"static const int32_t {} = INT32_C({});
    "#,
                    e.ident, e.value
                ));
            }
            InterfaceNode::Function(f) => {
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature = functions::signature::Signature::new(f, &counts);
                let documentation = idlc_codegen::documentation::Documentation::new(
                    f,
                    idlc_codegen::documentation::DocumentationStyle::Rust,
                );

                implementations.push_str(&functions::implementation::emit(
                    f,
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
                    r#"static const {} {} = {}({});
    "#,
                    change_primitive(c.r#type),
                    c.ident,
                    change_const_primitive(c.r#type),
                    c.value
                ));
            }
            InterfaceNode::Error(e) => {
                errors.push_str(&format!(
                    r#"static const int32_t {} = INT32_C({});
    "#,
                    e.ident, e.value
                ));
            }
            InterfaceNode::Function(f) => {
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature = functions::signature::Signature::new(f, &counts);
                let documentation = idlc_codegen::documentation::Documentation::new(
                    f,
                    idlc_codegen::documentation::DocumentationStyle::Rust,
                );
                let mut params = idlc_codegen_c::interface::functions::signature::iter_to_string(
                    signature.params(),
                );
                if !params.is_empty() {
                    params.remove(0);
                }
                func_titles.push_str(&format!(
                    r#"virtual int32_t {}({}) = 0;
    "#,
                    f.ident, params,
                ));
                op_codes.push_str(&format!(
                    r#"static const ObjectOp OP_{} = {};
    "#,
                    f.ident, f.id,
                ));
                implementations.push_str(&functions::implementation::emit(
                    f,
                    &documentation,
                    &counts,
                    &signature,
                ));
            }
        }
    }

    if !base_iface.is_empty() {
        base_iface = format!(": public {base_iface}");
    }

    format!(
        r#"
class I{ident} {base_iface}{{
  public:
    {constants}
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

    format!(
        r#"
class {ident}ImplBase : protected ImplBase, public I{ident} {{
  public:
    {ident}ImplBase() {{}}
    virtual ~{ident}ImplBase() {{}}

  protected:
    virtual int32_t invoke(ObjectOp op, ObjectArg* a, ObjectCounts k) {{
        switch (ObjectOp_methodID(op)) {{
            {invokes}
            default: {{ return Object_ERROR_INVALID; }}
        }}
        return Object_ERROR_INVALID;
    }}
}};
"#
    )
}
