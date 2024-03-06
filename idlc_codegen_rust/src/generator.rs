use std::collections::HashMap;

use idlc_codegen::Descriptor;
use idlc_mir::Node;

use idlc_codegen::MINKIDL_HEADER_COMMENT;

use crate::{
    globals::{emit_const, emit_struct},
    interface::emit,
};

pub struct Generator;

impl idlc_codegen::Generator for Generator {
    fn generate(mir: &idlc_mir::Mir) -> Descriptor {
        let mut base = std::path::PathBuf::from(
            mir.tag
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_lowercase(),
        );
        base.set_extension("rs");
        let mut interfaces = HashMap::new();
        let prologue = &format!("// {MINKIDL_HEADER_COMMENT}\n");
        interfaces.insert(base.clone(), prologue.to_owned());

        for node in &mir.nodes {
            match node.as_ref() {
                Node::Const(c) => {
                    interfaces.get_mut(&base).unwrap().push_str(&emit_const(c));
                }
                Node::Struct(s) => {
                    interfaces
                        .get_mut(&base)
                        .unwrap()
                        .push_str(&emit_struct(s.as_ref()));
                }
                Node::Interface(i) => {
                    let mut interface_content = String::new();
                    for base in i.iter().skip(1) {
                        interface_content.push_str(&format!(
                            "use crate::interfaces::{}::*;\n",
                            base.ident.to_lowercase()
                        ));
                    }
                    interface_content.push('\n');
                    interface_content.push_str(&emit(i));
                    let name = format!("{}.rs", i.ident.to_lowercase());
                    if name == base.to_str().unwrap() {
                        interfaces
                            .get_mut(&base)
                            .unwrap()
                            .push_str(&interface_content);
                    } else {
                        interfaces.insert(name.into(), format!("{prologue}{interface_content}"));
                    }
                }
                _ => (),
            }
        }

        interfaces
            .into_iter()
            .filter(|(_, content)| content != prologue)
            .collect()
    }
}
