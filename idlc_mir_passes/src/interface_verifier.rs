// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashSet;

use crate::MirCompilerPass;

use idlc_mir::{InterfaceNode, Node, ParamTypeIn, ParamTypeOut, Struct, Type};

pub struct InterfaceVerifier<'mir> {
    mir: &'mir idlc_mir::Mir,
}

impl<'mir> InterfaceVerifier<'mir> {
    #[must_use]
    pub const fn new(mir: &'mir idlc_mir::Mir) -> Self {
        Self { mir }
    }
}

impl<'mir> MirCompilerPass<'_> for InterfaceVerifier<'mir> {
    type Output = ();

    fn run_pass(&'_ mut self) -> Self::Output {
        for src in self.mir.nodes.iter().filter_map(|x| {
            if let Node::Interface(src) = x.as_ref() {
                Some(src)
            } else {
                None
            }
        }) {
            let mut consts = CollisionDetector::new(src);
            let mut functions = CollisionDetector::new(src);

            for from in src {
                for node in &from.nodes {
                    match node {
                        InterfaceNode::Const(c) => consts.add_ident(&c.ident, from),
                        InterfaceNode::Error(e) => consts.add_ident(&e.ident, from),
                        InterfaceNode::Function(f) => {
                            functions.add_ident(&f.ident, from);

                            let mut args_array_in = false;
                            let mut args_value_in = false;
                            let mut args_array_out = false;
                            let mut args_value_out = false;
                            for param in &f.params {
                                match param {
                                    idlc_mir::Param::In { r#type, ident: _ } => match r#type {
                                        ParamTypeIn::Array(t, cnt) => {
                                            if let idlc_mir::Type::Interface(i) = t {
                                                let iface_name =
                                                    i.as_deref().unwrap_or("interface").to_string();
                                                if !cnt.is_some() {
                                                    idlc_errors::unrecoverable!(
                                                        "Interface `{}`, method `{}` should not have unbounded array of interface",
                                                        iface_name, f.ident
                                                    );
                                                } else if cnt.unwrap()
                                                    == std::num::NonZeroU16::new(1).unwrap()
                                                {
                                                    idlc_errors::warn!(
                                                        "Interface `{}`, method `{}` has the array size of 1. It is better to use non-array interface instead222",
                                                        iface_name, f.ident
                                                    );
                                                }

                                                args_array_in = true;
                                            };
                                            if let Type::Struct(_) | Type::Primitive(_) = t {
                                                if let Type::Struct(Struct::Big(s)) = t {
                                                    if s.contains_interfaces() {
                                                        idlc_errors::unrecoverable!(
                                                            "Struct with Object inside cannot be used as an array",
                                                        );
                                                    }
                                                }
                                                if cnt.is_some() {
                                                    idlc_errors::unrecoverable!(
                                                        "Interface `{}`, method `{}` should not have bounded array of primitive/struct",
                                                        src.ident, f.ident
                                                    );
                                                }
                                            }
                                        }
                                        ParamTypeIn::Value(t) => {
                                            if let Type::Interface(_) = t {
                                                args_value_in = true;
                                            };
                                        }
                                    },
                                    idlc_mir::Param::Out { r#type, ident: _ } => match r#type {
                                        ParamTypeOut::Array(t, cnt) => {
                                            if let Type::Interface(i) = t {
                                                let iface_name =
                                                    i.as_deref().unwrap_or("interface").to_string();
                                                if !cnt.is_some() {
                                                    idlc_errors::unrecoverable!(
                                                        "Interface `{}`, method `{}` should not have unbounded array of interface",
                                                        iface_name, f.ident
                                                    );
                                                } else if cnt.unwrap()
                                                    == std::num::NonZeroU16::new(1).unwrap()
                                                {
                                                    idlc_errors::warn!(
                                                        "Interface `{}`, method `{}` has the array size of 1. It is better to use non-array interface instead111",
                                                        iface_name, f.ident
                                                    );
                                                }
                                                args_array_out = true;
                                            };
                                            if let Type::Struct(_) | Type::Primitive(_) = t {
                                                if let Type::Struct(
                                                    Struct::Big(s) | Struct::Small(s),
                                                ) = t
                                                {
                                                    if s.contains_interfaces() {
                                                        idlc_errors::unrecoverable!(
                                                            "Struct with Object inside cannot be used as an array",
                                                        );
                                                    }
                                                }
                                                if cnt.is_some() {
                                                    idlc_errors::unrecoverable!(
                                                        "Interface `{}`, method `{}` should not have bounded array of primitive/struct",
                                                        src.ident, f.ident
                                                    );
                                                }
                                            }
                                        }
                                        ParamTypeOut::Reference(t) => {
                                            if let Type::Interface(_) = t {
                                                args_value_out = true;
                                            };
                                        }
                                    },
                                }
                            }
                            if (args_array_in && args_value_in)
                                || (args_array_out && args_value_out)
                            {
                                idlc_errors::unrecoverable!(
                                    "Interface `{}`, method `{}` has both object array and non-array object arguments",
                                    src.ident, f.ident,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

struct CollisionDetector<'a> {
    src: &'a idlc_mir::Interface,
    inner: HashSet<&'a idlc_ast::Ident>,
}

impl<'a> CollisionDetector<'a> {
    pub fn new(src: &'a idlc_mir::Interface) -> Self {
        Self {
            src,
            inner: HashSet::new(),
        }
    }

    pub fn add_ident(&mut self, ident: &'a idlc_ast::Ident, from: &'a idlc_mir::Interface) {
        if !self.inner.insert(ident) {
            let orig = self.inner.get(ident).unwrap();
            idlc_errors::unrecoverable!(
                "Collision deteced for identifier `{ident}`. Initially defined in interface `{}:{}`, later defined again at `{}:{}`",
                self.src.ident, orig.span.start,
                from.ident, ident.span.start,
            );
        }
    }
}
