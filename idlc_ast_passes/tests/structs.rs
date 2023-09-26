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
