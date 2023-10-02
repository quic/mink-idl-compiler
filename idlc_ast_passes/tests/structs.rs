//! Tests to check alignment and size requirements for structs
use idlc_ast_passes::*;

use crate::idl_store::IDLStore;

fn verify(idl: &'static str) -> Result<(), idlc_ast_passes::Error> {
    let store = IDLStore::new();
    let name = std::path::PathBuf::from("struct-verifier.idl");
    let node = idlc_ast::from_string(name.to_path_buf(), idl).unwrap();
    store.insert_canonical(&name, &node);
    let mut cycles = cycles::Cycles::new(&store);
    let toposort = cycles.run_pass(&node)?;
    Ok(idlc_ast_passes::struct_verifier::StructVerifier::run_pass(
        &store, &toposort,
    )?)
}

#[test]
fn unaligned_no_custom() {
    assert!(verify(
        r#"
        struct Unaligned {
            uint8 start;
            uint16 unaligned;
            float32 still_works;
        };"#
    )
    .is_err());
}

#[test]
fn unaligned() {
    assert!(verify(
        r#"
        struct Custom {
            uint16 aligned;
        };
        struct Unaligned {
            uint8 start;
            Custom aligned;
            float32 still_works;
        };"#
    )
    .is_err());
}

#[test]
fn unaligned_arrays() {
    assert!(verify(
        r#"
        struct Custom {
            uint16 aligned;
        };
        struct Unaligned {
            uint8 start;
            Custom[2] aligned;
            float32 still_works;
        };"#
    )
    .is_err());
}

#[test]
fn aligned_no_custom() {
    assert!(verify(
        r#"
        struct Aligned {
            uint8 start;
            uint8 padding;
            uint16 aligned;
            float32 still_works;
        };"#
    )
    .is_ok());
}

#[test]
fn aligned() {
    assert!(verify(
        r#"
        struct Custom {
            uint16 aligned;
        };
        struct Unaligned {
            uint8 start;
            uint8 padding;
            Custom aligned;
            float32 still_works;
        };"#
    )
    .is_ok());
}

#[test]
fn aligned_arr() {
    assert!(verify(
        r#"
        struct Custom {
            uint16 aligned;
        };
        struct Unaligned {
            uint8[40] start;
            uint8[2] padding;
            Custom[35] aligned;
            float32 still_works;
        };"#
    )
    .is_ok());
}

#[test]
fn embedded_object() {
    assert!(verify(
        r#"
        interface Foo {};
        struct A {
            Foo f;
        };"#
    )
    .is_ok());
}

#[test]
fn embedded_object_custom() {
    assert!(verify(
        r#"
        interface Foo {};
        struct Custom {
            uint16 aligned;
        };
        struct A {
            Foo f;
            Custom[2] aligned;
            uint32 n;
        };"#
    )
    .is_ok());
}

#[test]
fn embedded_object_arr() {
    assert!(verify(
        r#"
        interface Foo {};
        struct A {
            Foo[4] f;
        };"#
    )
    .is_ok());
}

#[test]
fn structs_out_of_order() {
    assert!(verify(
        r#"
        struct s2 {
            s1 s;
            uint8 a;
        };
        struct s1 { uint8 a; };"#
    )
    .is_ok());
}

#[test]
#[should_panic]
fn unknown_struct_def() {
    _ = verify(
        r#"
        struct s2 {
            s1 s;
            uint8 a;
        };"#,
    )
}

#[test]
fn field_same_name() {
    assert!(verify(
        r#"
        struct s2 {
            uint8 a;
            uint8 a;
        };"#
    )
    .is_err());
}

#[test]
#[should_panic(expected = "Duplicate symbol detected")]
fn struct_interface_same_name() {
    _ = verify(
        r#"
        interface IFoo {};
        struct IFoo {
            int64 a;
        };"#,
    );
}

#[test]
#[should_panic(expected = "Duplicate symbol detected")]
fn struct_interface_same_name_ordered() {
    _ = verify(
        r#"
        struct IFoo {
            int64 a;
        };
        interface IFoo {};
        "#,
    );
}

#[test]
#[should_panic(expected = "Duplicate symbol detected")]
fn duplicate_consts() {
    _ = verify(
        r#"
        const uint8 FOO = 10;
        const uint32 FOO = 32;
        "#,
    );
}
