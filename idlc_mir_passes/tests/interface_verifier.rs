use idlc_ast_passes::cycles::Cycles;
use idlc_ast_passes::idl_store::IDLStore;
use idlc_ast_passes::struct_verifier::StructVerifier;
use idlc_ast_passes::CompilerPass;
use idlc_mir_passes::MirCompilerPass;

fn verify(a_idl: &str) {
    let mut store = IDLStore::new();
    let name = std::path::PathBuf::from("mir.idl");
    let node = idlc_ast::from_string(name.clone(), a_idl, true).unwrap();
    store.insert_canonical(&name, &node);

    let ast = store.get_ast(&name).unwrap();
    let struct_ordering = Cycles::new(&store).run_pass(&ast).unwrap();
    let _ = StructVerifier::run_pass(&store, &struct_ordering);
    let mir = idlc_mir::parse_to_mir(&ast, &mut store);
    idlc_mir_passes::interface_verifier::InterfaceVerifier::new(&mir).run_pass();
}

#[should_panic]
#[test]
fn duplicated_const() {
    verify(
        r"
        interface IFoo {
            const int32 A = 32;
            const int32 A = 42;
        };
        ",
    );
}

#[should_panic]
#[test]
fn duplicated_method() {
    verify(
        r"
        interface IFoo {
            method num();
            method num2();
            method num();
        };
        ",
    );
}

#[should_panic]
#[test]
fn duplicated_method_with_different_arg() {
    verify(
        r"
        interface IFoo {
            method num();
            method num(in int32 ai, out int32 ao);
        };
        ",
    );
}

#[should_panic]
#[test]
fn duplicated_base_method() {
    verify(
        r"
        interface Foo {
            method num();
        };
        interface IFoo : Foo {
            method num();
            method num2();
        };
        ",
    );
}

#[should_panic]
#[test]
fn duplicated_error() {
    verify(
        r"
        interface IFoo {
            error ERROR_FOO;
            error ERROR_FOO;
        };
        ",
    );
}

#[should_panic]
#[test]
fn duplicated_base_error() {
    verify(
        r"
        interface Foo {
            error ERROR_FOO;
        };
        interface IFoo : Foo {
            error ERROR_FOO;
            error ERROR_IFOO;
        };
        ",
    );
}

#[should_panic]
#[test]
fn duplicated_base_const() {
    verify(
        r"
        interface Foo {
            error FOO;
            const uint8 FOO = 10;
        };
        ",
    );
}

#[should_panic]
#[test]
fn obj_with_obj_array_input() {
    verify(
        r"
        interface Foo {
            method num();
        };
        interface IFoo {
            method num(in Foo[] a, in Foo b);
        };
        ",
    );
}

#[should_panic]
#[test]
fn obj_with_obj_array_output() {
    verify(
        r"
        interface Foo {
            method num();
        };
        interface IFoo {
            method num(out Foo a, out Foo[] b);
        };
        ",
    );
}
