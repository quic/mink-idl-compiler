// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use crate::types::change_primitive;

use idlc_mir::{Interface, InterfaceNode};

mod functions;
pub mod mink_primitives;

pub fn emit_interface(interface: &Interface, input_name: &str) -> String {
    use mink_primitives::{JMINK_OBJECT, MINK_OBJECT, MINK_PROXY, OP_ID, PROXY};
    let ident = &interface.ident;

    let mut constants = String::new();
    let mut errors = String::new();
    let mut op_codes = String::new();
    let mut traits = String::new();
    let mut implementations = String::new();
    let mut invokes = String::new();

    interface.iter().for_each(|iface| {
        iface.nodes.iter().for_each(|node| match node {
            InterfaceNode::Const(r#const) => {
                let const_ident = r#const.ident.to_uppercase();
                let ty = change_primitive(r#const.r#type);
                let value = &r#const.value;
                constants.push_str(&format!(
                    r#"{ty} {ident}_{const_ident} = {value};
    "#
                ))
            }
            InterfaceNode::Error(e) => {
                let error_ident = e.ident.to_uppercase();
                let value = &e.value;
                errors.push_str(&format!(
                    r#"int {ident}_{error_ident} = {value};
    "#
                ))
            }
            InterfaceNode::Function(f) => {
                let fn_ident = &f.ident;
                let id = f.id;
                op_codes.push_str(&format!(
                    r#"int {ident}_{OP_ID}_{fn_ident} = {id};
    "#
                ));
                let counts = idlc_codegen::counts::Counter::new(f);
                let signature = functions::signature::Signature::new(f);
                let documentation = idlc_codegen::documentation::Documentation::new(
                    f,
                    idlc_codegen::documentation::DocumentationStyle::Java,
                );

                implementations.push_str(&functions::implementation::emit(
                    f,
                    &iface.ident,
                    &documentation,
                    &counts,
                    &signature,
                ));
                invokes.push_str(&functions::invoke::emit(f, ident, &iface.ident, &signature));
                traits.push_str(&functions::traits::emit(f, &documentation, &signature));
            }
        });
    });

    let mut base_ident = input_name.to_string();
    base_ident.push_str(
        &interface
            .base
            .as_ref()
            .map(|x| format!(",{}", x.ident.as_ref()))
            .unwrap_or_default(),
    );

    let base_for_proxy = interface.base.as_ref().map_or(MINK_PROXY.to_string(), |x| {
        format!("{0}.{PROXY}", x.ident.as_ref())
    });
    let base_for_mink_obj = interface
        .base
        .as_ref()
        .map_or(JMINK_OBJECT.to_string(), |x| {
            format!("{0}.{MINK_OBJECT}", x.ident.as_ref())
        });

    if interface.base.is_some() {
        format!(
            r#"
public interface {ident} extends {base_ident} {{
    {constants}
    {errors}
    {op_codes}
    {traits}
    class Proxy extends {base_for_proxy} implements {ident} {{
        public Proxy(IMinkObject o) {{
            super(o);
        }}
        {implementations}
    }}
    class MinkObject extends {base_for_mink_obj} {{
        public MinkObject({ident} obj) {{
            super(obj);
        }}
        public void invoke(int methodID, byte[][] bi, int[] boSizes, byte[][] bo, IMinkObject[] oi, IMinkObject[] oo)throws InvokeException {{
            if (isNull()) throw new InvokeException(IMinkObject.ERROR_BADOBJ);
            switch (methodID) {{
                {invokes}
                default: {{
                    super.invoke(methodID, bi, boSizes, bo, oi, oo);
                }}
            }}
        }}
    }}
}}
    "#,
        )
    } else {
        format!(
            r#"
public interface {ident} extends {base_ident} {{
    {constants}
    {errors}
    {op_codes}
    {traits}
    class Proxy extends {base_for_proxy} implements {ident} {{
        public Proxy(IMinkObject o) {{
            super(o);
        }}
        {implementations}
    }}
    class MinkObject extends {base_for_mink_obj} {{
        protected {ident} mObj;
        public MinkObject({ident} obj) {{
            super();
            mObj = obj;
        }}
        @Override
        public void retain() {{
            super.retain();
        }}
        @Override
        public void release() {{
            super.release();
            if (mRefs.get() == 0) mObj = null;
        }}
        @Override
        public boolean isNull() {{
            return mObj == null;
        }}
        public void invoke(int methodID, byte[][] bi, int[] boSizes, byte[][] bo, IMinkObject[] oi, IMinkObject[] oo)throws InvokeException {{
            if (isNull()) throw new InvokeException(IMinkObject.ERROR_BADOBJ);
            switch (methodID) {{
                {invokes}
                default: {{
                    throw new InvokeException(IMinkObject.ERROR_INVALID);
                }}
            }}
        }}
    }}
}}
    "#,
        )
    }
}
