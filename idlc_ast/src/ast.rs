use std::{num::NonZeroU16, path::PathBuf, rc::Rc};

use crate::Error;

/// Maximum allowed size for a struct array [`u16::MAX`]
pub type Count = NonZeroU16;

#[derive(Debug, Clone, PartialEq, Eq)]
/// AST structure for an IDL.
pub struct Ast {
    /// Tag denoting the AST name.
    pub tag: PathBuf,
    /// Nodes for the AST tree.
    pub nodes: Vec<Rc<Node>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    #[must_use]
    pub const fn ident(&self) -> Option<&Ident> {
        match self {
            Self::Const(c) => Some(&c.ident),
            Self::Struct(s) => Some(&s.ident),
            Self::Interface(i) => Some(&i.ident),
            _ => None,
        }
    }

    #[must_use]
    pub const fn r#type(&self) -> &'static str {
        match self {
            Self::Include(_) => "include",
            Self::Const(_) => "const",
            Self::Struct(_) => "struct",
            Self::Interface(_) => "interface",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub ident: Ident,
    pub fields: Vec<StructField>,
}

impl Struct {
    #[must_use]
    pub fn new_object(ident: &Ident) -> Self {
        Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interface {
    pub ident: Ident,
    pub base: Option<Ident>,
    pub nodes: Vec<InterfaceNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Documentation(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceNode {
    Const(Const),
    Function(Function),
    Error(Ident),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionAttribute {
    Optional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub doc: Option<Documentation>,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub attributes: Vec<FunctionAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Const {
    pub ident: Ident,
    pub r#type: Primitive,
    pub value: String,
}

impl Const {
    #[must_use]
    pub const fn r#type(&self) -> &Primitive {
        &self.r#type
    }
    #[must_use]
    pub const fn value(&self) -> &String {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamTypeIn {
    Array(Type, Option<Count>),
    Value(Type),
}

impl AsRef<Type> for ParamTypeIn {
    #[inline]
    fn as_ref(&self) -> &Type {
        match self {
            Self::Array(t, _) | Self::Value(t) => t,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamTypeOut {
    Array(Type, Option<Count>),
    Reference(Type),
}

impl AsRef<Type> for ParamTypeOut {
    #[inline]
    fn as_ref(&self) -> &Type {
        match self {
            Self::Array(t, _) | Self::Reference(t) => t,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    In { r#type: ParamTypeIn, ident: Ident },
    Out { r#type: ParamTypeOut, ident: Ident },
}

impl Param {
    #[inline]
    #[must_use]
    pub const fn ident(&self) -> &Ident {
        match self {
            Self::In { r#type: _, ident } => ident,
            Self::Out { r#type: _, ident } => ident,
        }
    }
}

impl AsRef<Type> for Param {
    fn as_ref(&self) -> &Type {
        match self {
            Self::In { r#type, ident: _ } => r#type.as_ref(),
            Self::Out { r#type, ident: _ } => r#type.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub ident: Ident,
    pub val: (Type, Count),
}

impl StructField {
    #[must_use]
    pub const fn r#type(&self) -> &(Type, Count) {
        &self.val
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    UntypedBuffer,
    Primitive(Primitive),
    Interface,
    Custom(Ident),
}
impl Type {
    #[must_use]
    pub const fn interface_size() -> usize {
        Primitive::Uint64.size() * 2
    }

    #[must_use]
    pub const fn interface_align() -> usize {
        Self::interface_size()
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
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::Uint8 | Self::Int8 => 1,
            Self::Uint16 | Self::Int16 => 2,
            Self::Uint32 | Self::Int32 | Self::Float32 => 4,
            Self::Uint64 | Self::Int64 | Self::Float64 => 8,
        }
    }

    #[must_use]
    pub const fn alignment(self) -> usize {
        // All primitive types are required to be aligned to it's size.
        self.size()
    }

    /// Ensures the type is checked for range
    pub(crate) fn new(r#type: &str, value: &str) -> Result<Self, Error> {
        let value = value.trim();
        let radix = if value.starts_with("0x") || value.starts_with("-0x") {
            16
        } else {
            10
        };
        let value = &value.replace("0x", "");
        match r#type {
            "uint8" => {
                u8::from_str_radix(value, radix)?;
                Ok(Self::Uint8)
            }
            "uint16" => {
                u16::from_str_radix(value, radix)?;
                Ok(Self::Uint16)
            }
            "uint32" => {
                u32::from_str_radix(value, radix)?;
                Ok(Self::Uint32)
            }
            "uint64" => {
                u64::from_str_radix(value, radix)?;
                Ok(Self::Uint64)
            }
            "int8" => {
                i8::from_str_radix(value, radix)?;
                Ok(Self::Int8)
            }
            "int16" => {
                i16::from_str_radix(value, radix)?;
                Ok(Self::Int16)
            }
            "int32" => {
                i32::from_str_radix(value, radix)?;
                Ok(Self::Int32)
            }
            "int64" => {
                i64::from_str_radix(value, radix)?;
                Ok(Self::Int64)
            }
            "float32" => {
                if value.parse::<f32>()?.is_infinite() {
                    return Err(Error::FloatIsInfinite);
                }
                Ok(Self::Float32)
            }
            "float64" => {
                if value.parse::<f64>()?.is_infinite() {
                    return Err(Error::FloatIsInfinite);
                }
                Ok(Self::Float64)
            }
            _ => Err(Error::UnknownPrimitiveType(value.to_string())),
        }
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
    #[must_use]
    pub const fn new_without_span(ident: String) -> Self {
        Self {
            span: Span { start: 0, end: 0 },
            ident,
        }
    }

    #[inline]
    #[must_use]
    pub const fn new(ident: String, span: Span) -> Self {
        Self { span, ident }
    }
}
impl PartialEq<Self> for Ident {
    fn eq(&self, other: &Self) -> bool {
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
