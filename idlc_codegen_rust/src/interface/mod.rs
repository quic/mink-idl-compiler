// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

use crate::globals::emit_const;

use idlc_mir::{APIVersion, Interface, InterfaceNode, VERSION_FUNC_NAME};

mod error;
mod functions;
pub mod mink_primitives;
mod variable_names;

pub fn emit(interface: &Interface) -> String {
    use mink_primitives::{
        ARG, CONTEXT, COUNTS, GENERIC_ERROR, INVOKE_FN, OBJECT, OP_ID, OP_RELEASE, OP_RETAIN,
        OP_VERSION, TYPED_OBJECT_TRAIT, WRAPPER,
    };
    let ident = &interface.ident;
    let mut trait_functions = Vec::new();
    let mut implementations = Vec::new();
    let mut invoke_arms = Vec::new();

    let mut constants = String::new();
    let mut errors = Vec::new();

    let interface_version = interface.get_version();

    for node in &interface.nodes {
        match node {
            InterfaceNode::Const(c) => constants.push_str(&emit_const(c)),
            InterfaceNode::Error(e) => errors.push(e),
            InterfaceNode::Function(f) => {
                if f.deprecated_in().is_none_or(|v| v < interface_version) {
                    let signature = functions::signature::Signature::new(f);
                    let counts = idlc_codegen::counts::Counter::new(f);
                    let documentation = idlc_codegen::documentation::Documentation::new(
                        f,
                        idlc_codegen::documentation::DocumentationStyle::Rust,
                    );

                    implementations.push(functions::implementation::emit(
                        f,
                        &documentation,
                        counts,
                        &signature,
                    ));
                    invoke_arms.push(functions::invoke::emit(f, &signature, counts));
                    trait_functions.push(functions::traits::emit(f, &documentation, &signature));
                }
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
                if f.deprecated_in().is_none_or(|v| v < interface_version) {
                    let signature = functions::signature::Signature::new(f);
                    invoke_arms.push(functions::invoke::emit(
                        f,
                        &signature,
                        idlc_codegen::counts::Counter::new(f),
                    ));
                }
            });
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

    let upcast_target = interface.base.as_ref().map_or(OBJECT, |x| x.ident.as_ref());

    let wrapper = format!("{WRAPPER}::Wrapper::<dyn I{ident}>");

    let interface_version = interface.get_version();
    let APIVersion { major, minor } = interface_version;

    let output = format!(
        r#"
    {errors}
    {constants}
    #[repr(transparent)]
    #[derive(Debug, Clone, PartialEq)]
    pub struct {ident}({OBJECT});
    unsafe impl Sync for {ident} {{}}
    unsafe impl Send for {ident} {{}}
    unsafe impl {TYPED_OBJECT_TRAIT} for {ident} {{}}

    pub trait I{ident}: {base_ident} 'static {{
        {trait_functions}
    }}

    /// '{ident}' interface at version '{interface_version}'
    impl {ident} {{
        #[inline]
        pub fn r#{VERSION_FUNC_NAME}(&self) -> Result<(u32), Error> {{
            let mut r#version = std::mem::MaybeUninit::<u32>::uninit();
            let mut args = [
                crate::object::Arg {{
                    bi: crate::object::BufIn {{
                        ptr: std::ptr::addr_of_mut!(r#version).cast(),
                        size: std::mem::size_of::<u32>(),
                    }},
                }},
            ];
            match unsafe {{
                self.0.invoke({OP_VERSION}, args.as_mut_ptr(), crate::object::pack_counts(0, 1, 0, 0))
            }} {{
                0 => {{
                    let r#version = unsafe {{ r#version.assume_init() }};
                    Ok((r#version))
                }}
                err => Err(unsafe {{ std::mem::transmute(err) }}),
            }}
        }}
        {implementations}
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
            {OP_VERSION} => {{
                if counts != crate::object::pack_counts(0, 1, 0, 0) {{
                    return std::mem::transmute(crate::object::error::generic::GENERIC);
                }}
                let args = std::slice::from_raw_parts_mut(args, 1);
                let r#a_orig = args[0].b.size;
                if r#a_orig < std::mem::size_of::<u32>() {{
                    return {GENERIC_ERROR}::SIZE_OUT.into();
                }}
                let r#a_lenout = &mut *std::ptr::addr_of_mut!(args[0].b.size);
                let r#a = std::slice::from_raw_parts_mut(
                    args[0].b.ptr.cast::<u8>(),
                    r#a_orig / std::mem::size_of::<u8>(),
                );
                let value: u32 = IDLVersion::new({major}, {minor}, 0).into();
                r#a.copy_from_slice(&value.to_le_bytes());
                *r#a_lenout = std::mem::size_of::<u32>();
                0
            }},
            _ => {GENERIC_ERROR}::INVALID.into(),
        }}
    }}

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
    #[repr(transparent)]
    pub struct IDLVersion(u32);

    impl IDLVersion {{
        // Field widths
        pub const MAJOR_BITS: u32 = 10;
        pub const MINOR_BITS: u32 = 10;
        pub const PATCH_BITS: u32 = 12;

        // Bit positions (LSB = bit 0)
        pub const PATCH_SHIFT: u32 = 0;
        pub const MINOR_SHIFT: u32 = Self::PATCH_SHIFT + Self::PATCH_BITS; // 12
        pub const MAJOR_SHIFT: u32 = Self::MINOR_SHIFT + Self::MINOR_BITS; // 22

        // Masks (unshifted)
        pub const MAJOR_MASK: u32 = (1u32 << Self::MAJOR_BITS) - 1; // 0x3FF
        pub const MINOR_MASK: u32 = (1u32 << Self::MINOR_BITS) - 1; // 0x3FF
        pub const PATCH_MASK: u32 = (1u32 << Self::PATCH_BITS) - 1; // 0xFFF

        /// Pack (major, minor, patch) into a single u32.
        /// Values are truncated to their field widths.
        pub const fn new(major: u16, minor: u16, patch: u16) -> Self {{
            let major = (major as u32) & Self::MAJOR_MASK;
            let minor = (minor as u32) & Self::MINOR_MASK;
            let patch = (patch as u32) & Self::PATCH_MASK;

            Self((major << Self::MAJOR_SHIFT) |
                 (minor << Self::MINOR_SHIFT) |
                 (patch << Self::PATCH_SHIFT))
        }}

        /// Like `new`, but returns None if any component is out of range.
        pub const fn try_new(major: u16, minor: u16, patch: u16) -> Option<Self> {{
            if (major as u32) > Self::MAJOR_MASK {{ return None; }}
            if (minor as u32) > Self::MINOR_MASK {{ return None; }}
            if (patch as u32) > Self::PATCH_MASK {{ return None; }}
            Some(Self::new(major, minor, patch))
        }}

        #[inline(always)]
        pub const fn major(self) -> u16 {{
            (((self.0 >> Self::MAJOR_SHIFT) & Self::MAJOR_MASK) as u16)
        }}

        #[inline(always)]
        pub const fn minor(self) -> u16 {{
            (((self.0 >> Self::MINOR_SHIFT) & Self::MINOR_MASK) as u16)
        }}

        #[inline(always)]
        pub const fn patch(self) -> u16 {{
            (((self.0 >> Self::PATCH_SHIFT) & Self::PATCH_MASK) as u16)
        }}

        /// Access the raw packed value (host endianness).
        #[inline(always)]
        pub const fn as_u32(self) -> u32 {{
            self.0
        }}

        /// Construct from a raw packed value (host endianness).
        #[inline(always)]
        pub const fn from_u32(raw: u32) -> Self {{
            Self(raw)
        }}

        #[inline(always)]
        pub const fn to_le_bytes(self) -> [u8; 4] {{
            self.0.to_le_bytes()
        }}

        #[inline(always)]
        pub const fn from_le_bytes(bytes: [u8; 4]) -> Self {{
            Self(u32::from_le_bytes(bytes))
        }}
    }}

    // Optional ergonomic conversions:
    impl From<u32> for IDLVersion {{
        fn from(v: u32) -> Self {{ Self(v) }}
    }}
    impl From<IDLVersion> for u32 {{
        fn from(v: IDLVersion) -> u32 {{ v.0 }}
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
    pub fn downcast_concrete<R, T: I{ident} + 'static>(obj: &{OBJECT}, f: impl FnMut(&T) -> R) -> Option<R> {{
        {WRAPPER}::downcast_concrete::<R, T, dyn I{ident}>(obj, {MARKER}, f)
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
