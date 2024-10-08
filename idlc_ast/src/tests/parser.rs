// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

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
            r#"include"header.idl""#,
        ]
    );
}

#[test]
fn valid_includes() {
    valid!(
        include,
        [
            r#"include "header.idl""#,
            r#"include "header🅰.idl""#, // EMOJIs work!
            r#"include "header.idl""#,
            r#"include           "header.idl""#,
            r#"include   "/path/to/header.idl""#,
            r#"include   "/path/to/.header.idl""#,
            r#"include   "/path/to/.header with space.idl""#,
            r#"include   "/.path/to/.header.idl""#,
            r#"include "../path/to/header.idl""#,
            "include\t\"header.idl\"",
            "include\n\"header.idl\"",
        ]
    );
}

#[test]
fn primitive_types() {
    let widths = [8, 16, 32, 64];
    for width in widths {
        valid!(
            primitive_type,
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
            r"struct multiple_fields {
                  uint64 a;
                  uint64 b;
                  uint64 c;
                  uint64 d;
                  uint64 e;
                  uint64 f;
                  uint64 g;
                  uint64 h;
            };",
            "struct\ttest {uint8 test;};",
            "struct\ntest {uint8 test;};",
        ]
    );

    invalid!(
        r#struct,
        [
            "struct test { uint81[32] test; };",
            "struct interface {uint8 test;};",
            "struct test { uint8[32] 123test; };",
            "struct test { buffer untyped_buffer };",
            "structtest {uint8 test;};",
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
            "const float32 foo = -5.123213;",
            "const float32 bar = 5.123213;",
            "const uint8 bar = -0xabc;",
            "const               uint8 foo = 123;",
            "const\tuint8 bar = -0xabc;",
            "const\n\nuint8 bar = -0xabc;",
        ]
    );

    invalid!(
        r#const,
        [
            "const uint8 foo = 0x123azf;",
            "const uint8 bar = 123abf;",
            "const uint8 bar = abc;",
            "const uint8 bar = -abc;",
            "const float32 bar = 5.;",
            "constuint8 foo = 123;",
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
            "method bar(in uint32[] req, out float64 rsp);",
            "method bar(in interface[3] req, out float64 rsp);",
            "method bar(in IHWKey[] req, out float64 rsp);",
            "method bar();",
            "method bar(in interface x,    out interface y);",
            "method bar(in IHWKey x,    out IHWKeyFactory2 y);",
            r"method bar(in uint32 req,
                          out uint32 rsp);",
            r"method bar(in uint32 req,
                          out uint32 foo,
                          out uint32 bar);",
            "method\tfoo(in buffer req, out buffer rsp);",
            "method\nfoo(in buffer req, out buffer rsp);",
            "#[optional] method foo(in buffer req, out buffer rsp);",
            r"#[optional]
              method foo(in buffer req, out buffer rsp);",
        ]
    );

    invalid!(
        function,
        [
            "method foo(,);",
            "methods foo();",
            "method 123foo();",
            "method foo(int buffer req);",
            "method foo(in interface[] req);",
            "method foo(in buffer req in buffer rsp);",
            "method foo(buffer req);",
            "method foo(in req);",
            "method foo(in, in);",
            "method foo(in uint32 123req);",
            "method foo()",
            "method bar(in 2IHWKey x,    out IHWKeyFactory2 y);",
            "methodfoo();",
            "#[unsupported_attribute] method foo(in buffer req, out buffer rsp);",
            "#[optional]method foo(in buffer req, out buffer rsp);",
        ]
    );
}

#[test]
fn comments() {
    valid!(
        COMMENT,
        [
            "// foo\n",
            "// foo",
            "/// foo\n",
            "// foo\n//bar\n",
            "//** foo */",
            "/** foo */",
            "/**foo \n bar */",
            "/* foo \n bar */\n\n\n",
            "/** @} */",
        ]
    );
}

#[test]
fn documentation() {
    valid!(
        DOCUMENTATION,
        [
            "/**\nfoo\nbar */",
            "/**\n foo \n bar */",
            "/**\n foo\n\n\n bar */\n",
        ]
    );

    invalid!(
        DOCUMENTATION,
        [
            "/** foo */",
            "/** \nfoo \n /**  bar\n */",
            "/**    \n   foo \n bar \n*/\n\n\n",
        ]
    );
}

#[test]
fn interface() {
    valid!(
        interface,
        [
            "interface ITest {};",
            "interface ITest: IBase { error tmp; };",
            "interface\tITest: IBase { error tmp; };",
            "interface\nITest: IBase { error tmp; };",
        ]
    );

    invalid!(
        interface,
        [
            "interface 12ITest {};",
            "interface 12ITest {abc};",
            "interface ITest: IBase IBase2 {};",
            "interfaceITest {};",
        ]
    );
}

#[test]
fn errors() {
    valid!(
        error,
        [
            "error ERROR_FOO;",
            "error ERROR_BAR ; ",
            "error axybs;",
            "error\naxybs;",
            "error\taxybs;",
        ]
    );

    invalid!(error, ["error 12ERROR;", "errorERROR;",]);
}
