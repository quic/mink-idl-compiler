use std::{num::NonZeroU16, path::PathBuf, rc::Rc};

use crate::Error;

/// Maximum allowed size for a struct array [`u16::MAX`]
pub type Count = NonZeroU16;

#[derive(Debug, Clone, PartialEq)]
/// AST structure for an IDL.
pub struct Ast {
    /// Tag denoting the AST name.
    pub tag: PathBuf,
    /// Nodes for the AST tree.
    pub nodes: Vec<Rc<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
/// enum for the different types of nodes in the AST.
pub enum Node {
    /// Denotes an `include "foo.idl"`
    Include(PathBuf),
    /// Denotes a `const <type> <ident> = <val>;` decl.
    Const(Const),
    /// Denotes a structure with arbitrary amount of fields.
    Struct(Struct),
    /// Denotes an interface with arbitrary amount of sub nodes,
    ///
    /// These subnodes are limited to what [`InterfaceNode`] defines and doesn't
    /// allow the full features of a [`Node`]
    Interface(Interface),
}
impl Node {
    pub fn ident(&self) -> Option<&Ident> {
        match self {
            Node::Const(c) => Some(&c.ident),
            Node::Struct(s) => Some(&s.ident),
            Node::Interface(i) => Some(&i.ident),
            _ => None,
        }
    }

    pub fn r#type(&self) -> &'static str {
        match self {
            Node::Include(_) => "include",
            Node::Const(_) => "const",
            Node::Struct(_) => "struct",
            Node::Interface(_) => "interface",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub ident: Ident,
    pub fields: Vec<StructField>,
}

impl Struct {
    pub fn new_object(ident: &Ident) -> Self {
        Struct {
            ident: ident.clone(),
            fields: vec![
                StructField {
                    ident: Ident::new_without_span("invoke".to_string()),
                    val: (
                        Type::Primitive(Primitive::Uint64),
                        NonZeroU16::new(1).unwrap(),
                    ),
                },
                StructField {
                    ident: Ident::new_without_span("context".to_string()),
                    val: (
                        Type::Primitive(Primitive::Uint64),
                        NonZeroU16::new(1).unwrap(),
                    ),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub ident: Ident,
    pub base: Option<Ident>,
    pub nodes: Vec<InterfaceNode>,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Documentation(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceNode {
    Const(Const),
    Function(Function),
    Error(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub doc: Option<Documentation>,
    pub ident: Ident,
    pub params: Vec<Param>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub ident: Ident,
    pub r#type: Primitive,
    pub value: String,
}

impl Const {
    pub fn r#type(&self) -> &Primitive {
        &self.r#type
    }
    pub fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamTypeIn {
    Array(Type),
    Value(Type),
}

impl AsRef<Type> for ParamTypeIn {
    #[inline]
    fn as_ref(&self) -> &Type {
        match self {
            ParamTypeIn::Array(t) | ParamTypeIn::Value(t) => t,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamTypeOut {
    Array(Type),
    Reference(Type),
}

impl AsRef<Type> for ParamTypeOut {
    #[inline]
    fn as_ref(&self) -> &Type {
        match self {
            ParamTypeOut::Array(t) | ParamTypeOut::Reference(t) => t,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    In { r#type: ParamTypeIn, ident: Ident },
    Out { r#type: ParamTypeOut, ident: Ident },
}

impl Param {
    #[inline]
    pub fn ident(&self) -> &Ident {
        match self {
            Param::In { r#type: _, ident } => ident,
            Param::Out { r#type: _, ident } => ident,
        }
    }
}

impl AsRef<Type> for Param {
    fn as_ref(&self) -> &Type {
        match self {
            Param::In { r#type, ident: _ } => r#type.as_ref(),
            Param::Out { r#type, ident: _ } => r#type.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub ident: Ident,
    pub val: (Type, Count),
}

impl StructField {
    pub fn r#type(&self) -> &(Type, Count) {
        &self.val
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(Primitive),
    Interface,
    Custom(Ident),
}
impl Type {
    pub const fn interface_size() -> usize {
        Primitive::Uint64.size() * 2
    }

    pub const fn interface_align() -> usize {
        Type::interface_size()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Primitive {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
}

impl Primitive {
    pub const fn size(self) -> usize {
        match self {
            Primitive::Uint8 | Primitive::Int8 => 1,
            Primitive::Uint16 | Primitive::Int16 => 2,
            Primitive::Uint32 | Primitive::Int32 | Primitive::Float32 => 4,
            Primitive::Uint64 | Primitive::Int64 | Primitive::Float64 => 8,
        }
    }

    pub const fn alignment(self) -> usize {
        // All primitive types are required to be aligned to it's size.
        self.size()
    }
}

impl TryFrom<&str> for Primitive {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "uint8" => Ok(Self::Uint8),
            "uint16" => Ok(Self::Uint16),
            "uint32" => Ok(Self::Uint32),
            "uint64" => Ok(Self::Uint64),
            "int8" => Ok(Self::Int8),
            "int16" => Ok(Self::Int16),
            "int32" => Ok(Self::Int32),
            "int64" => Ok(Self::Int64),
            "float32" => Ok(Self::Float32),
            "float64" => Ok(Self::Float64),
            _ => Err(Error::UnknownPrimitiveType(value.to_string())),
        }
    }
}

/// Identifiers are utf-8 strings with a span.
#[derive(Debug, Clone)]
pub struct Ident {
    pub span: Span,
    pub ident: String,
}

impl Ident {
    #[inline]
    pub const fn new_without_span(ident: String) -> Self {
        Ident {
            span: Span { start: 0, end: 0 },
            ident,
        }
    }
}
impl PartialEq<Ident> for Ident {
    fn eq(&self, other: &Ident) -> bool {
        self.ident == other.ident
    }
}
impl Eq for Ident {}
impl std::hash::Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
    }
}
impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.ident, f)
    }
}

impl PartialEq<str> for Ident {
    fn eq(&self, other: &str) -> bool {
        self.ident == other
    }
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        &self.ident
    }
}
impl std::ops::Deref for Ident {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.ident
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
