//! This library adds in the [Mink](http://go/mink) architecture over the input AST.
//!
//! # MIR Additions
//!
//! ### Function to [`u32`] mapping
//! Each function in `Mink` is represented as a [`u32`] although restricted by
//! implementations to `ObjectOp_METHOD_USERMAX` which is `0x3FFF`. Architecture
//! reserves the upper bits to transport specific bits.
//!
//! ### Error to [`i32`] mapping
//! Each error defined by the interface regardless of it's parent interface
//! enforces that the [`i32`] value starts at `Object_ERROR_USERBASE` which is
//! `10`. The MIR ensures that error numbers and the ordering in the IDLs are
//! matching.
//!
//! ### Interface segregation
//! Multiple ASTs may exist in the IDL however [`Mir`] only represents a single
//! interface. Information about constants and structures that are common
//! between multiple interfaces are defined by [`Mir::Common`] variant.
//!
//! All codegen backends that depend on this MIR are obligated to use all of
//! it's information to ensure the specification of Mink is upheld.

use std::rc::Rc;

use idlc_ast::{Const, Documentation, Ident, Node, Param};

#[derive(Debug, Clone, PartialEq)]
pub enum Mir {
    /// Representing any arbitrary interface nodes defined by the AST that is
    /// not an [`Node::Interface`]. This could be [`Node::Include`] which can be
    /// used by backends to figure out includes etc.
    Common(Node),
    /// Interface nodes that require transformation from the source IDL to
    /// concepts internal to the Mink architecture.
    Interface(Interface),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub ident: Ident,
    pub base: Option<Rc<Interface>>,
    pub nodes: Vec<InterfaceNode>,
}
#[derive(Debug, Clone, PartialEq)]

pub enum InterfaceNode {
    Const(Const),
    Function(Function),
    Error(Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub doc: Option<Documentation>,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub ident: Ident,
    pub value: i32,
}
