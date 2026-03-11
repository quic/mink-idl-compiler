// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::collections::HashSet;

use idlc_ast::{InterfaceNode, Node, SemanticVersion};

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
            // The default version for all functions is 1.0
            let mut current_version = SemanticVersion { major: 1, minor: 0 };
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
                    // Check the version of this function
                    // - Gather any and all listed version attributes
                    let version_attrs: Vec<&SemanticVersion> = function
                        .attributes
                        .iter()
                        .filter_map(|attr| {
                            if let idlc_ast::FunctionAttribute::Version(a) = attr {
                                Some(a)
                            } else {
                                None
                            }
                        })
                        .collect();
                    // - Ensure that no more than 1 version is listed
                    if version_attrs.len() > 1 {
                        idlc_errors::unrecoverable!(
                            "Function `{}::{}` has multiple 'version' attributes: {}",
                            interface.ident,
                            function.ident,
                            version_attrs
                                .iter()
                                .map(|a| a.to_string())
                                .collect::<Vec<String>>()
                                .join(", "),
                        );
                    }
                    // - Ensure that method versions are monotonically increasing
                    for func_ver in version_attrs {
                        if func_ver < &current_version {
                            idlc_errors::unrecoverable!(
                                "Function `{}::{}` version `{}` cannot be less than the previous method `{}`",
                                interface.ident,
                                function.ident,
                                func_ver,
                                current_version,
                            );
                        }
                        if func_ver > &current_version {
                            // Carry the current version forward to the next function
                            current_version = *func_ver;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
