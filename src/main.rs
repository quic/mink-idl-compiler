use pest::Parser;

#[derive(pest_derive::Parser)]
#[grammar = "../grammar/idl.pest"]
pub struct IDLParser;

fn main() {
    println!("Hello, world!");
}

#[test]
fn invalid_includes() {
    assert!(IDLParser::parse(Rule::include, r#"include "header.h""#).is_err());
    assert!(IDLParser::parse(Rule::include, r#"include "header.hidl""#).is_err());
    assert!(IDLParser::parse(Rule::include, r#"include "header.idls""#).is_err());
    assert!(IDLParser::parse(Rule::include, r#"include "header.idl.xyz""#).is_err());
    assert!(IDLParser::parse(Rule::include, r#"include "header.idl.idl""#).is_err());
    assert!(IDLParser::parse(Rule::include, r#"include"header.idl""#).is_err());
    assert!(IDLParser::parse(Rule::include, r#"include "./path/to\.header.idl""#).is_err());
}

#[test]
fn valid_includes() {
    assert!(IDLParser::parse(Rule::include, r#"include "header.idl""#).is_ok());
    assert!(IDLParser::parse(Rule::include, r#"include "header.idl""#).is_ok());
    assert!(IDLParser::parse(Rule::include, r#"include           "header.idl""#).is_ok());
    assert!(IDLParser::parse(Rule::include, r#"include   "/path/to/header.idl""#).is_ok());
    assert!(IDLParser::parse(Rule::include, r#"include   "/path/to/.header.idl""#).is_ok());
    assert!(IDLParser::parse(
        Rule::include,
        r#"include   "/path/to/.header with space.idl""#
    )
    .is_ok());
    assert!(IDLParser::parse(Rule::include, r#"include   "/.path/to/.header.idl""#).is_ok());
}

#[test]
fn primitive_types() {
    let widths = [8, 16, 32, 64];
    for width in widths {
        assert!(IDLParser::parse(Rule::primitive_types, &format!("int{width}")).is_ok());
        assert!(IDLParser::parse(Rule::primitive_types, &format!("uint{width}")).is_ok());
    }
}

#[test]
fn identifiers() {
    assert!(IDLParser::parse(Rule::ident, "IHelloworld").is_ok());
    assert!(IDLParser::parse(Rule::ident, "IHello123world_123").is_ok());
    assert!(IDLParser::parse(Rule::ident, "i_am_a_buffer").is_ok());
    assert!(IDLParser::parse(Rule::ident, "teller").is_ok());
    assert!(IDLParser::parse(Rule::ident, "s").is_ok());
}

#[test]
fn struct_fields() {
    assert!(IDLParser::parse(Rule::struct_field, "uint8 test;").is_ok());
    assert!(IDLParser::parse(Rule::struct_field, "int8 test123;").is_ok());
    assert!(IDLParser::parse(Rule::struct_field, "int8 test_123;").is_ok());
    assert!(IDLParser::parse(Rule::struct_field, "int8[64] test_123;").is_ok());

    assert!(IDLParser::parse(Rule::struct_field, "int8[64a] test_123;").is_err());
    assert!(IDLParser::parse(Rule::struct_field, "int8[] test_123;").is_err());
}

#[test]
fn r#struct() {
    assert!(IDLParser::parse(Rule::r#struct, "struct test {uint8 test;}").is_ok());
    assert!(IDLParser::parse(Rule::r#struct, "struct test { uint8[64] test; }").is_ok());
    assert!(IDLParser::parse(
        Rule::r#struct,
        "struct test { uint8[64] test; uint64 test2; }"
    )
    .is_ok());
    assert!(IDLParser::parse(
        Rule::r#struct,
        "struct test { uint8[64] test;\n\n\n\n uint64 test2;\n\n\n\n\n }"
    )
    .is_ok());

    assert!(IDLParser::parse(Rule::r#struct, "struct test { uint8[64] test\n; }").is_err());
    assert!(IDLParser::parse(Rule::r#struct, "struct test { uint8[\n64] test; }").is_err());
    assert!(IDLParser::parse(Rule::r#struct, "struct test { uint8[64]\ntest; }").is_err());
}

#[test]
fn values() {
    assert!(IDLParser::parse(Rule::value, "123").is_ok());
    assert!(IDLParser::parse(Rule::value, "0x123abf").is_ok());
}

#[test]
fn consts() {
    assert!(IDLParser::parse(Rule::r#const, "const uint8 foo = 123;").is_ok());
    assert!(IDLParser::parse(Rule::r#const, "const uint8 bar = 0x123abf;").is_ok());

    assert!(IDLParser::parse(Rule::r#const, "const uint8 foo = 0x123azf;").is_err());
    assert!(IDLParser::parse(Rule::r#const, "const uint8 bar = 123abf;").is_err());
    assert!(IDLParser::parse(Rule::r#const, "const uint8 bar = abc;").is_err());
}
