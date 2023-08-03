//! As long as the current AST doesn't have duplicate symbols that's all we care
//! about since minkidl only generates interface files for the input AST not the
//! dependencies.

use std::collections::HashSet;

use crate::ast::{
    visitor::{walk_all, Visitor},
    Ident, Node,
};

use super::{CompilerPass, Error};

#[derive(Default, Debug, Clone)]
pub struct DuplicateDetector {
    set: HashSet<Ident>,
    dup: Option<Ident>,
}

impl DuplicateDetector {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_success(&self) -> Result<(), Error> {
        if let Some(occ2) = &self.dup {
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

impl Visitor<'_> for DuplicateDetector {
    fn visit_ident(&mut self, ident: &'_ Ident) {
        if self.dup.is_none() && !self.set.insert(ident.clone()) {
            // Duplicate was found
            self.dup = Some(ident.clone());
        }
    }
}

impl CompilerPass<'_> for DuplicateDetector {
    type Output = ();

    fn run_pass(&mut self, ast: &'_ Node) -> Result<Self::Output, Error> {
        walk_all(self, ast);
        self.is_success()
    }
}
