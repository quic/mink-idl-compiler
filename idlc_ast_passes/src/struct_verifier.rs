// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! Implementation of struct alignment and member checker
//!
//! Ensures every struct is aligned to the size of the largest member, this rule
//! holds for recursive structs as well.
//!
//! Also ensures recursive structs don't exist by holding a visited set for the
//! DFS search.

use std::collections::{HashMap, HashSet};

use idlc_ast::{Ident, Type};

use crate::idl_store::IDLStore;

type Size = usize;
type Alignment = usize;

#[derive(Debug, Clone, Copy)]
pub struct StructVerifier;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("struct member `{member}` in `{parent}` was not aligned to required alignment `{alignment}`; offset is `{offset}`")]
    StructMemberNotAligned {
        member: String,
        parent: String,
        alignment: usize,
        offset: usize,
    },
    #[error("struct `{parent}` is not aligned to it's natural alignment `{alignment}`; size is `{size}`")]
    StructNotAligned {
        parent: String,
        alignment: usize,
        size: usize,
    },
    #[error("struct `{parent}` contains duplicate field names: `{names:?}`")]
    StructFieldSameName { parent: Ident, names: Vec<Ident> },
}

impl StructVerifier {
    pub fn run_pass(idl_store: &IDLStore, toposort: &[String]) -> Result<(), Error> {
        let mut store: HashMap<String, (Size, Alignment)> = HashMap::new();
        for r#struct in toposort {
            let (node, _) = idl_store.struct_lookup(r#struct).unwrap();
            let mut size = 0;
            let mut alignment = 0;
            let mut fields: HashSet<&Ident> = HashSet::new();

            for field in &node.fields {
                let ident = &field.ident;
                if let Some(existing) = fields.get(&ident) {
                    let existing: Ident = (*existing).clone();
                    return Err(Error::StructFieldSameName {
                        parent: node.ident.clone(),
                        names: vec![existing, ident.clone()],
                    });
                }
                fields.insert(ident);

                let (ty, count) = field.r#type();
                let count = count.get() as usize;

                let (i_size, i_alignment) = match ty {
                    Type::Primitive(p) => (p.size(), p.alignment()),
                    Type::Custom(c) => *store.get(&c.ident).unwrap(),
                    Type::Interface => (Type::interface_size(), Type::interface_align()),
                    _ => unreachable!(),
                };
                if size % i_alignment != 0 {
                    return Err(Error::StructMemberNotAligned {
                        member: field.ident.to_string(),
                        parent: r#struct.to_string(),
                        alignment: i_alignment,
                        offset: size,
                    });
                }

                size += i_size * count;
                alignment = alignment.max(i_alignment);
            }

            if size % alignment != 0 {
                return Err(Error::StructNotAligned {
                    parent: r#struct.to_string(),
                    alignment,
                    size,
                });
            }
            store.insert(r#struct.clone(), (size, alignment));
        }

        Ok(())
    }
}
