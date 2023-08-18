//! Tests to validate cyclical imports in structs and interfaces.

use std::rc::Rc;

use idlc_ast::Node;
use idlc_ast_passes::cycles::Cycles;
use idlc_ast_passes::*;

fn verify(idl: &'static str) -> Result<Vec<String>, crate::Error> {
    let store = ASTStore::new();
    let name = "cycles.idl";
    let node = Rc::new(Node::from_string(name.to_string(), idl).unwrap());
    store.insert(name, &node);
    let mut verifier = Cycles::new(&store);
    verifier.run_pass(node.as_ref())
}

#[test]
fn cyclic_struct() {
    assert!(matches!(
        verify(
            r#"
        struct A {
            B b;
        };

        struct B {
            C c;
        };

        struct C {
            A a;
        };"#
        ),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
fn cyclic_iface() {
    assert!(matches!(
        verify(
            r#"
        interface A : B {
        };

        interface B : C {
        };

        interface C : A{
        };"#
        ),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
fn ordering_struct() {
    assert_eq!(
        verify(
            r#"
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
        "#
        )
        .unwrap(),
        ["B", "C", "D", "A"]
    );
}
