// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use convert_case::Casing;

pub fn emit(errors: &[&idlc_mir::mir::Error]) -> String {
    let prologue = r"
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Error(crate::object::Error);

impl From<Error> for crate::object::Error {
    #[inline(always)]
    fn from(e: Error) -> Self {
        e.0
    }
}
impl From<crate::object::Error> for Error {
    #[inline(always)]
    fn from(e: crate::object::Error) -> Self {
        Self(e)
    }
}

impl std::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}
impl std::error::Error for Error {}
";
    let mut match_arms = String::new();
    let mut defines = String::new();
    for error in errors {
        let ident = error.ident.to_uppercase();
        let value = error.value;
        let upper_camel_case_ident = ident.to_case(convert_case::Case::UpperCamel);
        defines += &format!(
            "pub const {ident}: Error = Error(unsafe {{ crate::object::Error::new_unchecked({value}) }});\n"
        );
        match_arms += &format!("{ident} => write!(f, \"{upper_camel_case_ident}\"),\n",);
    }

    let debug_impl = format!(
        r#"
    impl std::fmt::Debug for Error {{
        #[inline]
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
            match *self {{
                {match_arms}
                _ => write!(f, "{{}}", self.0)
            }}
        }}
    }}
    "#
    );

    prologue.to_owned() + &defines + &debug_impl
}
