//! Implementation of struct alignment and member checker
//!
//! Ensures every struct is aligned to the size of the largest member, this rule
//! holds for recursive structs as well.
//!
//! Also ensures recursive structs don't exist by holding a visited set for the
//! DFS search.
//!

use std::collections::{HashSet, VecDeque};

use crate::ast::{
    visitor::{walk_all, Visitor},
    Node,
};

use super::{ASTStore, CompilerPass};

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
        let mut visited: HashSet<&'ast str> = HashSet::new();
        visited.insert(&r#struct.ident);
        let mut stack = VecDeque::new();
        stack.extend(r#struct.fields.iter().cloned());
        let mut size = 0;
        while let Some(element) = stack.pop_front() {
            size += match &element.val.0 {
                crate::ast::Type::Primitive(p) => {
                    assert_eq!(
                        size % p.alignment(),
                        0,
                        "ICE: In struct `{}`; field `{}` didn't match alignment requirements of `{}`",
                        r#struct.ident,
                        &element.ident,
                        p.alignment()
                    );
                    p.size()
                }
                crate::ast::Type::Custom(c) => {
                    assert!(
                        !visited.contains(c.as_str()),
                        "Circular dependency detected for {c}"
                    );
                    let custom = self
                        .ast_store
                        .symbol_lookup(c)
                        .unwrap_or_else(|| panic!("Symbol {c} not found"));
                    for _ in 0..element.val.1.get() {
                        stack.extend(custom.fields.iter().cloned());
                    }
                    0
                }
            };
        }
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
