use super::{invalid, valid};
use pest::Parser;

#[test]
fn invalid_includes() {
    invalid!(
        include,
        [
            r#"include "header.h""#,
            r#"include "header.hidl""#,
            r#"include "header.idls""#,
            r#"include "header.idl.xyz""#,
            r#"include "header.idl.idl""#,
            r#"include"header.idl""#,
            r#"include "./path/to\.header.idl""#,
        ]
    );
}

#[test]
fn valid_includes() {
    valid!(
        include,
        [
            r#"include "header.idl""#,
            r#"include "header.idl""#,
            r#"include           "header.idl""#,
            r#"include   "/path/to/header.idl""#,
            r#"include   "/path/to/.header.idl""#,
            r#"include   "/path/to/.header with space.idl""#,
            r#"include   "/.path/to/.header.idl""#,
        ]
    );
}

#[test]
fn primitive_types() {
    let widths = [8, 16, 32, 64];
    for width in widths {
        valid!(
            primitive_types,
            [&format!("int{width}"), &format!("uint{width}"),]
        );
    }
}

#[test]
fn identifiers() {
    valid!(
        ident,
        [
            "IHelloworld",
            "IHello123world_123",
            "i_am_a_buffer",
            "teller",
            "s",
        ]
    );
}

#[test]
fn struct_fields() {
    valid!(
        struct_field,
        [
            "uint8 test;",
            "int8 test123;",
            "int8 test_123;",
            "int8[64] test_123;",
        ]
    );

    invalid!(struct_field, ["int8[64a] test_123;", "int8[] test_123;",]);
}

#[test]
fn r#struct() {
    valid!(
        r#struct,
        [
            "struct test {uint8 test;};",
            "struct test { uint8[64] test; };",
            "struct test { uint8[64] test; uint64 test2; };",
            "struct test { uint8[64] test\n;\n\n\n\n uint64 test2;\n\n\n\n\n };",
            r#"struct ISecureImage_programHeader {
                  uint64 p_type;
                  uint64 p_offset;
                  uint64 p_vaddr;
                  uint64 p_paddr;
                  uint64 p_filesz;
                  uint64 p_memsz;
                  uint64 p_flags;
                  uint64 p_align;
            };"#,
        ]
    );

    invalid!(
        r#struct,
        [
            "struct test { uint81[32] test; };",
            "struct test { uint8[32] 123test; };",
        ]
    );
}

#[test]
fn values() {
    valid!(value, ["123", "0x123abf", "-123",]);
}

#[test]
fn consts() {
    valid!(
        r#const,
        [
            "const uint8 foo = 123;",
            "const uint8 bar = 0x123abf;",
            "const int8 foo = -5;",
        ]
    );

    invalid!(
        r#const,
        [
            "const uint8 foo = 0x123azf;",
            "const uint8 bar = 123abf;",
            "const uint8 bar = abc;",
            "const uint8 bar = -abc;",
            "const uint8 bar = -0xabc;",
        ]
    );
}

#[test]
fn function() {
    valid!(
        function,
        [
            "method foo(in buffer req, out buffer rsp);",
            "method bar(in uint32 req, out float64 rsp);",
            "method bar();",
            "method bar(in interface x,    out interface y);",
            "method bar(in IHWKey x,    out IHWKeyFactory2 y);",
            r#"method bar(in uint32 req,
                          out uint32 rsp);"#,
            r#"method bar(in uint32 req,
                          out uint32 foo,
                          out uint32 bar);"#,
        ]
    );

    invalid!(
        function,
        [
            "method foo(,);",
            "methods foo();",
            "method 123foo();",
            "method foo(int buffer req);",
            "method foo(buffer req);",
            "method foo(in req);",
            "method foo(in, in);",
            "method foo(in uint32 123req);",
            "method foo()",
            "method bar(in 2IHWKey x,    out IHWKeyFactory2 y);",
        ]
    );
}

#[test]
fn comments() {
    valid!(
        COMMENT,
        [
            "// foo",
            "// foo\n//bar",
            "/** foo */",
            "/** foo \n bar */",
            "/** foo \n bar */\n\n\n",
        ]
    );
}
