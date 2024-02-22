use idlc_ast_passes::cycles::Cycles;
use idlc_ast_passes::idl_store::IDLStore;
use idlc_ast_passes::struct_verifier::StructVerifier;
use idlc_ast_passes::CompilerPass;
use idlc_mir::mir;

fn create_mir(a_idl: &str) -> mir::Mir {
    let mut store = IDLStore::new();
    let name = std::path::PathBuf::from("mir.idl");
    let node = idlc_ast::from_string(name.clone(), a_idl, true).unwrap();
    store.insert_canonical(&name, &node);

    let ast = store.get_ast(&name).unwrap();
    let struct_ordering = Cycles::new(&store).run_pass(&ast).unwrap();
    let _ = StructVerifier::run_pass(&store, &struct_ordering);
    idlc_mir::parse_to_mir(&ast, &mut store)
}

#[test]
fn op_code_test() {
    let mir = create_mir(
        r"
            const uint32 A = 0;

            struct s1 {
                uint32 a;
            };

            interface iface {
                method num();
            };

            interface iface2 : iface {
                method num2_1();
                method num2_2();
            };

            interface iface3 : iface2 {
                method num3_1();
                method num3_2();
                method num3_3();
            };

            interface iface4 : iface3 {
                method num4();
            };

            interface iface5 : iface4 {
                method num5();
            };
        ",
    );
    let node = &**mir.nodes.last().unwrap();
    let mut out: Vec<(&str, u32)> = vec![];
    if let idlc_mir::Node::Interface(interface) = node {
        let fn_iterator = interface.iter().flat_map(|iface| {
            iface.nodes.iter().filter_map(|node| {
                let idlc_mir::InterfaceNode::Function(f) = node else {
                    return None;
                };
                Some((f.ident.as_ref(), f.id))
            })
        });
        out = fn_iterator.collect();
    }

    assert_eq!(
        out,
        [
            ("num5", 7,),
            ("num4", 6,),
            ("num3_1", 3,),
            ("num3_2", 4,),
            ("num3_3", 5,),
            ("num2_1", 1,),
            ("num2_2", 2,),
            ("num", 0,),
        ]
    );
}

#[test]
fn err_code_test() {
    let mir = create_mir(
        r"
            interface iface {
                error ERROR_ONE;
            };

            interface iface2 : iface {
                method num(in int32 ia, out int32 oa);
                error ERROR_TWO_1;
                error ERROR_TWO_2;
            };

            interface iface3 : iface2 {
                error ERROR_THREE;
            };

            interface iface4 : iface3 {
                error ERROR_FOUR_1;
                error ERROR_FOUR_2;
            };

            interface iface5 : iface4 {
                error ERROR_FIVE_1;
                error ERROR_FIVE_2;
            };
        ",
    );
    let node = &**mir.nodes.last().unwrap();
    let mut out: Vec<(&str, i32)> = vec![];
    if let idlc_mir::Node::Interface(interface) = node {
        let fn_iterator = interface.iter().flat_map(|iface| {
            iface.nodes.iter().filter_map(|node| {
                let idlc_mir::InterfaceNode::Error(e) = node else {
                    return None;
                };
                Some((e.ident.as_ref(), e.value))
            })
        });
        out = fn_iterator.collect();
    }

    assert_eq!(
        out,
        [
            ("ERROR_FIVE_1", 16,),
            ("ERROR_FIVE_2", 17,),
            ("ERROR_FOUR_1", 14,),
            ("ERROR_FOUR_2", 15,),
            ("ERROR_THREE", 13,),
            ("ERROR_TWO_1", 11,),
            ("ERROR_TWO_2", 12,),
            ("ERROR_ONE", 10,),
        ]
    );
}
