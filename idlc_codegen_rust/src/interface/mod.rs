use crate::globals::emit_const;

use idlc_mir::{Interface, InterfaceNode};

mod error;
mod functions;
pub mod mink_primitives;
mod variable_names;

pub fn emit_interface(interface: &Interface) -> String {
    use mink_primitives::*;
    let ident = &interface.ident;
    let mut trait_functions = Vec::new();
    let mut implementations = Vec::new();
    let mut invoke_arms = Vec::new();

    let mut constants = String::new();
    let mut errors = Vec::new();

    for node in &interface.nodes {
        match node {
            InterfaceNode::Const(c) => constants.push_str(&emit_const(c)),
            InterfaceNode::Error(e) => errors.push(e),
            InterfaceNode::Function(f) => {
                let signature = functions::signature::Signature::new(f);
                let counts = idlc_codegen::counts::Counter::new(f);
                let documentation = idlc_codegen::documentation::Documentation::new(
                    f,
                    idlc_codegen::documentation::DocumentationStyle::Rust,
                );

                implementations.push(functions::implementation::emit(
                    f,
                    &documentation,
                    &counts,
                    &signature,
                ));
                invoke_arms.push(functions::invoke::emit(f, &signature, &counts));
                trait_functions.push(functions::traits::emit(f, &documentation, &signature));
            }
        }
    }

    // Invoke functions need to have all of the base-class op-codes
    interface.iter().skip(1).for_each(|iface| {
        iface
            .nodes
            .iter()
            .filter_map(|node| {
                if let InterfaceNode::Function(f) = node {
                    Some(f)
                } else {
                    None
                }
            })
            .for_each(|f| {
                let signature = functions::signature::Signature::new(f);
                let counts = idlc_codegen::counts::Counter::new(f);
                invoke_arms.push(functions::invoke::emit(f, &signature, &counts))
            })
    });

    let mut trait_functions = trait_functions.join(";");
    if !trait_functions.is_empty() {
        trait_functions.push(';');
    }
    let implementations = implementations.concat();
    let invoke_arms = invoke_arms.join(",");

    let errors = error::emit(&errors);
    let base_ident = interface
        .base
        .as_ref()
        .map(|x| format!("I{} +", x.ident.as_ref()))
        .unwrap_or_default();

    let upcast_target = interface
        .base
        .as_ref()
        .map(|x| x.ident.as_ref())
        .unwrap_or(OBJECT);

    let wrapper = format!("{WRAPPER}::Wrapper::<dyn I{ident}>");

    let output = format!(
        r#"
    {errors}
    {constants}
    #[repr(transparent)]
    #[derive(Debug, Clone, PartialEq)]
    pub struct {ident}({OBJECT});

    pub trait I{ident}: {base_ident} 'static {{
        {trait_functions}
    }}

    impl {ident} {{
        {implementations}
    }}

    impl {ident} {{
        #[inline]
        /// Returns inner raw [`{OBJECT}`] out of [`{ident}`].
        pub fn into_raw(self) -> {OBJECT} {{
            self.0
        }}

        #[inline]
        /// Uses raw [`{OBJECT}`] to create a typed [`{ident}`] without performing type checking.
        ///
        /// # Safety
        /// This function exhibits safe behavior as long as the [`{OBJECT}`]
        /// used is a valid [`{ident}`].
        pub const unsafe fn from_raw(o: crate::object::Object) -> Self {{
            Self(o)
        }}

        #[inline]
        /// Uses raw &[`{OBJECT}`] to create a typed &[`{ident}`] without performing type checking.
        ///
        /// # Safety
        /// This function exhibits safe behavior as long as the &[`{OBJECT}`] used is a valid &[`{OBJECT}`]
        pub const unsafe fn from_raw_ref(o: &crate::object::Object) -> &Self {{
            std::mem::transmute(o)
        }}
    }}

    impl AsRef<{upcast_target}> for {ident} {{
        #[inline]
        fn as_ref(&self) -> &{upcast_target} {{
            unsafe {{ std::mem::transmute(self) }}
        }}
    }}

    impl std::ops::Deref for {ident} {{
        type Target = {upcast_target};

        #[inline]
        fn deref(&self) -> &Self::Target {{
            self.as_ref()
        }}
    }}

    impl From<{ident}> for {upcast_target} {{
        fn from(value: {ident}) -> {upcast_target} {{
            unsafe {{ std::mem::transmute(value) }}
        }}
    }}

    impl<T: I{ident} + 'static> From<T> for {ident} {{
        fn from(concrete: T) -> Self {{
            let cx = Box::new(unsafe {{ {wrapper}::new::<T>(Box::new(concrete)) }});
            unsafe {{ Self({OBJECT}::create(MARKER, Box::into_raw(cx).cast())) }}
        }}
    }}

    static {MARKER}: {INVOKE_FN} = invoke;
    unsafe extern "C" fn invoke({h}: {CONTEXT}, {op}: {OP_ID}, {args}: *mut {ARG}, {counts}: {COUNTS}) -> i32 {{
        debug_assert_eq!({h}.align_offset(std::mem::align_of::<{wrapper}>()), 0);
        let {cx} = {h}.cast::<{wrapper}>();
        match op {{
            {invoke_arms}
            {OP_RELEASE} => {{
                {WRAPPER}::release(cx)
            }},
            {OP_RETAIN} => {{
                {WRAPPER}::retain(cx)
            }},
            _ => {GENERIC_ERROR}::INVALID.into(),
        }}
    }}

    /// Downcasts to the value of type `T` that was used to create this object.
    ///
    /// This only works with objects that have been created with `[{ident}::from]`
    /// with the same type `T`.
    ///
    /// If the object has been created with a different type or is a remote object
    /// from another environment, potentially written in another language, this function
    /// will return [`None`]
    ///
    /// This function is useful when an implementation gives out an _opaque_ [`{OBJECT}`]
    /// which it recieves later but needs to get the concrete struct behind the object.
    #[inline]
    pub fn downcast_concrete<T: I{ident} + 'static>(obj: &{OBJECT}) -> Option<&T> {{
        {WRAPPER}::downcast_concrete::<T, dyn I{ident}>(obj, {MARKER})
    }}
    "#,
        h = variable_names::invoke::HANDLE,
        op = variable_names::invoke::OP_ID,
        args = variable_names::invoke::ARGS,
        counts = variable_names::invoke::COUNTS,
        cx = variable_names::invoke::CONTEXT,
        MARKER = variable_names::invoke::MARKER
    );
    match syn::parse_file(&output) {
        Ok(file) => prettyplease::unparse(&file),
        Err(e) => {
            idlc_errors::unrecoverable!("Syntactic error `{e}` for output:\n{output}");
        }
    }
}
