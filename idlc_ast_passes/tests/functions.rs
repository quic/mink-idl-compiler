//! Tests to check alignment and size requirements for structs
use idlc_ast_passes::{functions::Functions, CompilerPass};

fn verify(idl: &'static str) {
    let node = idlc_ast::from_string("test.idl".into(), idl).unwrap();
    let mut functions = Functions::new();
    functions.run_pass(&node).unwrap();
}

#[should_panic = "Function `Duplicates::foo` has duplicate parameter `param1`"]
#[test]
fn duplicate_params() {
    verify(
        r"
        interface Duplicates {
            method foo(out uint8 param1, in uint32 param2, out interface param1);
        };",
    );
}
