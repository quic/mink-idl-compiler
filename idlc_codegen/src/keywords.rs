// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

macro_rules! keyword_gen {
    ($($lang: literal: [$($value: literal),*]),*) => {
        &[$($($value,)*)*]
    };
}

/// This crate has to ensure that all codegenerations succeed, so a keyword that isn't reserved in
/// Rust might be reserved in a different target language. This summarizes all the codegen backends
/// we support.
///
/// C17: [https://www.open-std.org/JTC1/SC22/WG14/www/docs/n2310.pdf](https://www.open-std.org/JTC1/SC22/WG14/www/docs/n2310.pdf)
/// C++23: [https://www.open-std.org/JTC1/SC22/WG21/docs/papers/2023/n4950.pdf](https://docs.oracle.com/javase/specs/jls/se22/html/jls-3.html#jls-3.9)
/// Java: [https://docs.oracle.com/javase/specs/jls/se22/html/jls-3.html#jls-3.9](https://docs.oracle.com/javase/specs/jls/se22/html/jls-3.html#jls-3.9)/
///
/// # Footnotes
///
/// Rust keywords are not added here since the language provides a way to escape
/// keywords using the `r#` syntax. See
/// [raw-identifiers](https://doc.rust-lang.org/rust-by-example/compatibility/raw_identifiers.html#raw-identifiers)
const RESERVED_KEYWORDS: &[&str] = keyword_gen! {
    "C17": [
        "auto", "break", "case", "char", "const", "continue", "default", "do", "double",
        "else", "enum", "extern", "float", "for", "goto", "if", "inline", "int",
        "long", "register", "restrict", "return", "short", "signed", "sizeof", "static",
        "struct", "switch", "typedef", "union", "unsigned", "void", "volatile",
        "while", "_Alignas", "_Alignof", "_Atomic", "_Bool", "_Complex", "_Generic",
        "_Imaginary", "_Noreturn", "_Static_assert", "_Thread_local"
    ],
    "C++-23": [
        "alignas", "alignof", "asm", "auto", "bool", "break", "case", "catch",
        "char", "char8_t", "char16_t", "char32_t", "class", "concept", "const",
        "consteval", "constexpr", "constinit", "const_cast", "continue", "co_await",
        "co_return", "co_yield", "decltype", "default", "delete", "do", "double",
        "dynamic_cast", "else", "enum", "explicit", "export", "extern", "false",
        "float", "for", "friend", "goto", "if", "inline", "int", "long", "mutable",
        "namespace", "new", "noexcept", "nullptr", "operator", "private",
        "protected", "public", "register", "reinterpret_cast", "requires", "return",
        "short", "signed", "sizeof", "static", "static_assert", "static_cast",
        "struct", "switch", "template", "this", "thread_local", "throw", "true",
        "try", "typedef", "typeid", "typename", "union", "unsigned", "using",
        "virtual", "void", "volatile", "wchar_t", "while"
    ],
    "Java": [
        "abstract", "continue", "for", "new", "switch", "assert", "default", "if",
        "package", "synchronized", "boolean", "do", "goto", "private", "this", "break",
        "double", "implements", "protected", "throw", "byte", "else", "import",
        "public", "throws", "case", "enum", "instanceof", "return", "transient",
        "catch", "extends", "int", "short", "try", "char", "final", "interface",
        "static", "void", "class", "finally", "long", "strictfp", "volatile", "const",
        "float", "native", "super", "while"
    ]
};

/// Checks if the given input is a reserved keyword supported by any of the backend languages.
pub fn is_reserved_keyword(input: &str) -> bool {
    RESERVED_KEYWORDS.contains(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_reserved() {
        for keyword in RESERVED_KEYWORDS {
            assert!(is_reserved_keyword(keyword));
        }
    }
}
