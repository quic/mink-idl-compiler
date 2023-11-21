use std::collections::HashSet;

use crate::MirCompilerPass;

use idlc_mir::*;

pub struct InterfaceVerifier<'mir> {
    mir: &'mir idlc_mir::Mir,
}

impl<'mir> InterfaceVerifier<'mir> {
    /// Constructor for InterfaceVerifier
    pub fn new(mir: &'mir idlc_mir::Mir) -> Self {
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

            for from in src.iter() {
                for node in &from.nodes {
                    match node {
                        InterfaceNode::Const(c) => consts.add_ident(&c.ident, from),
                        InterfaceNode::Error(e) => consts.add_ident(&e.ident, from),
                        InterfaceNode::Function(f) => functions.add_ident(&f.ident, from),
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
