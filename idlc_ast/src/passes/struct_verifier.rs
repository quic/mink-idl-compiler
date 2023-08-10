//! Implementation of struct alignment and member checker
//!
//! Ensures every struct is aligned to the size of the largest member, this rule
//! holds for recursive structs as well.
//!
//! Also ensures recursive structs don't exist by holding a visited set for the
//! DFS search.
//!
//! FIXME: Currently maxes out on [`MAX_SIZE`] which doesn't denote a cycle and could just be a huge struct.
//! Ideally we want to detect cycles using the toposort.

use std::collections::VecDeque;

use crate::ast::{
    visitor::{walk_all, Visitor},
    Node,
};

use super::{ASTStore, CompilerPass};

const MAX_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct StructVerifier<'ast> {
    ast_store: &'ast ASTStore,
}

impl<'ast> StructVerifier<'ast> {
    pub fn new(ast_store: &'ast ASTStore) -> Self {
        Self { ast_store }
    }
}

impl<'ast> Visitor<'ast> for StructVerifier<'ast> {
    fn visit_struct(&mut self, r#struct: &'ast crate::ast::Struct) {
        let mut stack = VecDeque::new();
        stack.extend(r#struct.fields.iter().cloned());
        let mut offset = 0;
        let mut alignment = 0;
        while let Some(element) = stack.pop_front() {
            offset += match &element.val.0 {
                crate::ast::Type::Primitive(p) => {
                    assert_eq!(
                        offset % p.alignment(),
                        0,
                        "struct `{}`; [sub-]field `{}` didn't match alignment requirements of `{}`",
                        r#struct.ident,
                        &element.ident,
                        p.alignment()
                    );
                    alignment = alignment.max(p.alignment());
                    p.size()
                }
                crate::ast::Type::Custom(c) => {
                    let custom = self
                        .ast_store
                        .symbol_lookup(c)
                        .unwrap_or_else(|| panic!("Symbol {c} not found"));
                    for _ in 0..element.val.1.get() {
                        for field in &custom.fields {
                            stack.push_front(field.clone());
                        }
                    }
                    0
                }
            };

            if offset >= MAX_SIZE {
                panic!(
                    "Struct sizes are limited @ {MAX_SIZE}. Possible recursive structures? \
                    These are unsupported and will possible never be supported due to \
                    language restrictions."
                );
            }
        }

        assert_eq!(
            offset % alignment,
            0,
            "struct's natural alignment is `{alignment}`; however size of struct is `{offset}` which isn't aligned."
        );
    }
}

impl<'ast> CompilerPass<'ast> for StructVerifier<'ast> {
    type Output = ();

    fn run_pass(&'ast mut self, ast: &'ast Node) -> Result<Self::Output, super::Error> {
        walk_all(self, ast);
        // TODO: Actually report errors instead of panicking!
        Ok(())
    }
}
