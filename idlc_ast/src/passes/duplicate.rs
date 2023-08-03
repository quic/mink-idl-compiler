//! As long as the current AST doesn't have duplicate symbols that's all we care
//! about since minkidl only generates interface files for the input AST not the
//! dependencies.

use std::collections::HashSet;

use crate::ast::{
    visitor::{walk_all, Visitor},
    Ident, Node,
};

use super::{CompilerPass, Error};

#[derive(Debug, Clone)]
pub struct DuplicateDetector<'a> {
    set: HashSet<&'a Ident>,
    dup: Option<&'a Ident>,
}

impl DuplicateDetector<'_> {
    fn new() -> Self {
        Self {
            set: HashSet::new(),
            dup: None,
        }
    }

    fn is_success(&self) -> Result<(), Error> {
        if let Some(occ2) = self.dup {
            let occ1 = (*self.set.get(occ2).unwrap()).clone();
            Err(Error::DuplicateDefinition {
                occ1,
                occ2: occ2.clone(),
            })
        } else {
            Ok(())
        }
    }
}

impl<'ast> Visitor<'ast> for DuplicateDetector<'ast> {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        if self.dup.is_none() {
            if !self.set.insert(ident) {
                // Duplicate was found
                self.dup = Some(ident);
            }
        }
    }
}

impl CompilerPass<'_, ()> for DuplicateDetector<'_> {
    fn run_pass(ast: &'_ Node) -> Result<(), Error> {
        let mut visitor = Self::new();
        walk_all(&mut visitor, ast);
        visitor.is_success()
    }
}
