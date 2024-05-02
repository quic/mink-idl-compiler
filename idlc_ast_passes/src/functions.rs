// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashSet;

use idlc_ast::{InterfaceNode, Node};

use crate::CompilerPass;

#[derive(Default)]
pub struct Functions;

impl Functions {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl<'ast> CompilerPass<'ast> for Functions {
    type Output = ();

    fn run_pass(&'ast mut self, ast: &'ast idlc_ast::Ast) -> Result<Self::Output, crate::Error> {
        for interface in ast.nodes.iter().filter_map(|node| match node.as_ref() {
            Node::Interface(i) => Some(i),
            _ => None,
        }) {
            for node in &interface.nodes {
                if let InterfaceNode::Function(function) = node {
                    let mut params = HashSet::new();
                    for param in &function.params {
                        let ident = param.ident();
                        if !params.insert(ident) {
                            idlc_errors::unrecoverable!(
                                "Function `{}::{}` has duplicate parameter `{}`",
                                interface.ident,
                                function.ident,
                                ident
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
