use std::collections::HashSet;

use crate::ast::{
    visitor::{walk_all, Visitor},
    Ident, Node,
};

use super::Error;

#[derive(Debug, Clone)]
struct DuplicateDetector<'a> {
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

pub fn contains_duplicate_symbols(ast: &Node) -> Result<(), Error> {
    let mut visitor = DuplicateDetector::new();
    walk_all(&mut visitor, ast);
    visitor.is_success()
}
