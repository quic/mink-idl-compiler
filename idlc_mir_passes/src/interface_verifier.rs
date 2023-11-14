use std::collections::HashMap;

use crate::MirCompilerPass;

use idlc_mir::*;

pub struct InterfaceVerifier<'mir> {
    mir: &'mir idlc_mir::Mir,
}

impl<'mir> MirCompilerPass<'_> for InterfaceVerifier<'mir> {
    type Output = ();

    fn run_pass(&'_ mut self) -> Result<Self::Output, crate::Error> {
        self.check_interface(self.mir)
    }
}

impl<'mir> InterfaceVerifier<'mir> {
    /// Constructor for InterfaceVerifier
    pub fn new(mir: &'mir idlc_mir::Mir) -> Self {
        Self { mir }
    }

    /// Interface check for duplication of names
    fn check_interface(&mut self, mir: &'_ idlc_mir::Mir) -> Result<(), crate::Error> {
        for node in &mir.nodes {
            if let Node::Interface(interface) = node.as_ref() {
                let mut consts: HashMap<String, idlc_ast::Ident> = HashMap::new();
                let mut functions: HashMap<String, idlc_ast::Ident> = HashMap::new();
                let mut errors: HashMap<String, idlc_ast::Ident> = HashMap::new();
                for iface in interface.iter() {
                    for iface_node in &iface.nodes {
                        match iface_node {
                            InterfaceNode::Const(const_) => {
                                let collision =
                                    consts.insert(const_.ident.to_string(), const_.ident.clone());
                                if collision.is_some() {
                                    return Err(crate::Error::AlreadyDefinedConst {
                                        first: const_.ident.clone(),
                                        second: consts
                                            .get(&const_.ident.to_string())
                                            .unwrap()
                                            .clone(),
                                    });
                                }
                            }
                            InterfaceNode::Function(function) => {
                                let mut params: HashMap<String, idlc_ast::Ident> = HashMap::new();
                                for param in &function.params {
                                    let old_value = match param {
                                        Param::In { ident, .. } => {
                                            params.insert(ident.to_string(), ident.clone())
                                        }
                                        Param::Out { ident, .. } => {
                                            params.insert(ident.to_string(), ident.clone())
                                        }
                                    };
                                    if let Some(collision) = old_value {
                                        return Err(crate::Error::DuplicateArgumentName {
                                            first: collision.clone(),
                                            second: params
                                                .get(&collision.to_string())
                                                .unwrap()
                                                .clone(),
                                        });
                                    }
                                }
                                let collision = functions
                                    .insert(function.ident.to_string(), function.ident.clone());
                                if collision.is_some() {
                                    return Err(crate::Error::AlreadyDefinedMethod {
                                        first: function.ident.clone(),
                                        second: functions
                                            .get(&function.ident.to_string())
                                            .unwrap()
                                            .clone(),
                                    });
                                }
                            }
                            InterfaceNode::Error(error) => {
                                let collision =
                                    errors.insert(error.ident.to_string(), error.ident.clone());
                                if collision.is_some() {
                                    return Err(crate::Error::AlreadyDefinedError {
                                        first: error.ident.clone(),
                                        second: errors
                                            .get(&error.ident.to_string())
                                            .unwrap()
                                            .clone(),
                                    });
                                }
                            }
                        }
                    }
                }
            };
        }
        Ok(())
    }
}
