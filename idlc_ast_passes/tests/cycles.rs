// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Tests to validate cyclical imports in structs and interfaces.
use idlc_ast_passes::cycles::Cycles;
use idlc_ast_passes::*;

use crate::idl_store::IDLStore;

fn verify(idl: &'static str) -> Result<Vec<String>, crate::Error> {
    let store = IDLStore::new();
    let name = std::path::PathBuf::from("cycles.idl");
    let node = idlc_ast::from_string(name.clone(), idl, true).unwrap();
    store.insert_canonical(&name, &node);
    let mut verifier = Cycles::new(&store);
    verifier.run_pass(&node)
}

#[test]
fn cyclic_struct() {
    assert!(matches!(
        verify(
            r"
        struct A {
            B b;
        };

        struct B {
            C c;
        };

        struct C {
            A a;
        };"
        ),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
fn cyclic_iface() {
    assert!(matches!(
        verify(
            r"
        interface A : B {
        };

        interface B : C {
        };

        interface C : A{
        };"
        ),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
fn ordering_struct() {
    assert_eq!(
        verify(
            r"
            struct A {
                B b;
                C c;
                D d;
            };

            struct D {
                B b;
                C c;
            };

            struct C {
                B b;
            };

            struct B {
                uint32 test;
            };
        "
        )
        .unwrap(),
        ["B", "C", "D", "A"]
    );
}
