#[test]
#[should_panic(expected = "isn't in range for type 'int32'")]
fn out_of_bound_pedantic() {
    let name = std::path::PathBuf::from("struct-verifier.idl");
    let idl = "const int32 FOO = 0x80000000;";
    idlc_ast::from_string(name, idl, true).unwrap_err();
}

#[test]
fn out_of_bound_non_pedantic() {
    let name = std::path::PathBuf::from("struct-verifier.idl");
    let idl = "const int32 FOO = 0x80000000;";
    idlc_ast::from_string(name, idl, false).unwrap();
}
