// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

/// Reserved keywords for each supported output language.
///
/// C17: [https://www.open-std.org/JTC1/SC22/WG14/www/docs/n2310.pdf](https://www.open-std.org/JTC1/SC22/WG14/www/docs/n2310.pdf)
/// C++23: [https://www.open-std.org/JTC1/SC22/WG21/docs/papers/2023/n4950.pdf](https://docs.oracle.com/javase/specs/jls/se22/html/jls-3.html#jls-3.9)
/// Java: [https://docs.oracle.com/javase/specs/jls/se22/html/jls-3.html#jls-3.9](https://docs.oracle.com/javase/specs/jls/se22/html/jls-3.html#jls-3.9)
///
/// Rust keywords are not added here since the language provides a way to escape
/// keywords using the `r#` syntax. See
/// [raw-identifiers](https://doc.rust-lang.org/rust-by-example/compatibility/raw_identifiers.html#raw-identifiers)

/// C17 reserved keywords.
pub const C_KEYWORDS: &[&str] = &[
    "auto",
    "break",
    "case",
    "char",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    "_Alignas",
    "_Alignof",
    "_Atomic",
    "_Bool",
    "_Complex",
    "_Generic",
    "_Imaginary",
    "_Noreturn",
    "_Static_assert",
    "_Thread_local",
];

/// C++23 reserved keywords.
pub const CPP_KEYWORDS: &[&str] = &[
    "alignas",
    "alignof",
    "asm",
    "auto",
    "bool",
    "break",
    "case",
    "catch",
    "char",
    "char8_t",
    "char16_t",
    "char32_t",
    "class",
    "concept",
    "const",
    "consteval",
    "constexpr",
    "constinit",
    "const_cast",
    "continue",
    "co_await",
    "co_return",
    "co_yield",
    "decltype",
    "default",
    "delete",
    "do",
    "double",
    "dynamic_cast",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "float",
    "for",
    "friend",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "nullptr",
    "operator",
    "private",
    "protected",
    "public",
    "register",
    "reinterpret_cast",
    "requires",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "static_cast",
    "struct",
    "switch",
    "template",
    "this",
    "thread_local",
    "throw",
    "true",
    "try",
    "typedef",
    "typeid",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "wchar_t",
    "while",
];

/// Java reserved keywords (keywords + reserved literals per JLS §3.9).
pub const JAVA_KEYWORDS: &[&str] = &[
    "abstract",
    "continue",
    "for",
    "new",
    "switch",
    "assert",
    "default",
    "if",
    "package",
    "synchronized",
    "boolean",
    "do",
    "goto",
    "private",
    "this",
    "break",
    "double",
    "implements",
    "protected",
    "throw",
    "byte",
    "else",
    "import",
    "public",
    "throws",
    "case",
    "enum",
    "instanceof",
    "return",
    "transient",
    "catch",
    "extends",
    "int",
    "short",
    "try",
    "char",
    "final",
    "interface",
    "static",
    "void",
    "class",
    "finally",
    "long",
    "strictfp",
    "volatile",
    "const",
    "float",
    "native",
    "super",
    "while",
];

/// Returns `true` when `input` is a C17 reserved keyword.
pub fn is_reserved_for_c(input: &str) -> bool {
    C_KEYWORDS.contains(&input)
}

/// Returns `true` when `input` is a C++23 reserved keyword.
pub fn is_reserved_for_cpp(input: &str) -> bool {
    CPP_KEYWORDS.contains(&input)
}

/// Returns `true` when `input` is a Java reserved keyword.
pub fn is_reserved_for_java(input: &str) -> bool {
    JAVA_KEYWORDS.contains(&input)
}

/// Checks if the given input is a reserved keyword in any of the supported backend languages.
pub fn is_reserved_keyword(input: &str) -> bool {
    is_reserved_for_c(input) || is_reserved_for_cpp(input) || is_reserved_for_java(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_keywords_reserved() {
        for kw in C_KEYWORDS {
            assert!(is_reserved_for_c(kw), "{kw} not detected as C keyword");
        }
    }

    #[test]
    fn cpp_keywords_reserved() {
        for kw in CPP_KEYWORDS {
            assert!(is_reserved_for_cpp(kw), "{kw} not detected as C++ keyword");
        }
    }

    #[test]
    fn java_keywords_reserved() {
        for kw in JAVA_KEYWORDS {
            assert!(
                is_reserved_for_java(kw),
                "{kw} not detected as Java keyword"
            );
        }
    }

    #[test]
    fn is_reserved_union() {
        for kw in C_KEYWORDS.iter().chain(CPP_KEYWORDS).chain(JAVA_KEYWORDS) {
            assert!(is_reserved_keyword(kw), "{kw} not in union");
        }
    }
}
