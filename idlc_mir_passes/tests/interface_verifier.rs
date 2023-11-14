use idlc_ast_passes::cycles::Cycles;
use idlc_ast_passes::idl_store::IDLStore;
use idlc_ast_passes::struct_verifier::StructVerifier;
use idlc_ast_passes::CompilerPass;
use idlc_mir_passes::MirCompilerPass;

fn verify(a_idl: &str) -> Result<(), idlc_mir_passes::Error> {
    let mut store = IDLStore::new();
    let name = std::path::PathBuf::from("mir.idl");
    let node = idlc_ast::from_string(name.to_path_buf(), a_idl).unwrap();
    store.insert_canonical(&name, &node);

    let ast = store.get_ast(&name).unwrap();
    let struct_ordering = Cycles::new(&store).run_pass(&ast).unwrap();
    let _ = StructVerifier::run_pass(&store, &struct_ordering);
    let mir = idlc_mir::parse_to_mir(&ast, &mut store);
    idlc_mir_passes::interface_verifier::InterfaceVerifier::new(&mir).run_pass()
}

#[test]
fn duplicated_const() {
    assert!(verify(
        r#"
        interface IFoo {
            const int32 A = 32;
            const int32 A = 42;
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_method() {
    assert!(verify(
        r#"
        interface IFoo {
            method num();
            method num2();
            method num();
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_method_with_different_arg() {
    assert!(verify(
        r#"
        interface IFoo {
            method num();
            method num(in int32 ai, out int32 ao);
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_base_method() {
    assert!(verify(
        r#"
        interface Foo {
            method num();
        };
        interface IFoo : Foo {
            method num();
            method num2();
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_error() {
    assert!(verify(
        r#"
        interface IFoo {
            error ERROR_FOO;
            error ERROR_FOO;
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_base_error() {
    assert!(verify(
        r#"
        interface Foo {
            error ERROR_FOO;
        };
        interface IFoo : Foo {
            error ERROR_FOO;
            error ERROR_IFOO;
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_argument_name() {
    assert!(verify(
        r#"
        interface IFoo {
            method num(in int32 a, out int32 a);
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_argument_name2() {
    assert!(verify(
        r#"
        interface IFoo {
            method num(out int32 a, in int32 a);
        };
        "#
    )
    .is_err());
}

#[test]
fn duplicated_argument_name3() {
    assert!(verify(
        r#"
        interface IFoo {
            method num(in int32 a, in int32 a, out int32 b, out int32 b);
        };
        "#
    )
    .is_err());
}

#[test]
fn unique_argument_name() {
    assert!(verify(
        r#"
        interface IFoo {
            method num(in int32 a, out int32 b);
            method num2(in int32 a, out int32 b);
        };
        "#
    )
    .is_ok());
}
