#[cfg(test)]
mod tests;
pub mod visitor;

use std::{num::NonZeroU16, path::Path};

use pest::{
    iterators::{Pair, Pairs},
    Parser as PestParser,
};

#[derive(pest_derive::Parser, Debug)]
#[grammar = "../grammar/idl.pest"]
pub struct Parser;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IoError for '{1}' due to `{0}`")]
    Io(#[source] std::io::Error, String),
    #[error("IDL Parsing failure:\n{0}\n")]
    AstGenerationFailure(Box<pest::error::Error<Rule>>),
    #[error("Unknown primitive type {0} encountered.")]
    UnknownPrimitiveType(String),
    #[error("Cannot parse integer")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Documentation for this node doesn't exist yet")]
    UnsupportedDocumentation,
}
impl From<pest::error::Error<Rule>> for Error {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Self::AstGenerationFailure(Box::new(value))
    }
}

macro_rules! ast_unwrap {
    ($e: expr) => {
        // Safety: PST to AST is a 1-to-1 transition and can never fail.
        unsafe { ($e).unwrap_unchecked() }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span) -> Self {
        Span {
            start: value.start(),
            end: value.end(),
        }
    }
}

/// Identifiers are utf-8 strings with a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub span: Span,
    pub ident: String,
}
impl<R: pest::RuleType + Ord> From<Pair<'_, R>> for Ident {
    fn from(value: Pair<'_, R>) -> Self {
        Self {
            span: Span {
                start: value.as_span().start(),
                end: value.as_span().end(),
            },
            ident: value.as_str().to_string(),
        }
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

/// Maximum allowed size for a struct array [`u16::MAX`]
type Count = NonZeroU16;

#[derive(Debug, Clone, PartialEq)]
/// AST structure for an IDL.
///
/// The layout is defined as a tree with a root node named
/// [`Node::CompilationUnit`] and all it's children may or may not contain
/// branches.
pub enum Node {
    /// Denotes an `include "foo.idl"`
    Include(String),
    /// Denotes a `const <type> <ident> = <val>;` decl.
    Const(Const),
    /// Denotes a structure with arbitrary amount of fields.
    Struct(Struct),
    /// Denotes an interface with arbitrary amount of sub nodes,
    ///
    /// These subnodes are limited to what [`InterfaceNode`] defines and doesn't
    /// allow the full features of a [`Node`]
    Interface(Interface),
    /// Root of the tree
    CompilationUnit(String, Vec<Node>),
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
            Node::CompilationUnit(_, _) => "Unit",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub ident: Ident,
    pub fields: Vec<StructField>,
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
    Function {
        doc: Option<Documentation>,
        ident: Ident,
        params: Vec<Param>,
    },
    Error(Ident),
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

#[derive(Debug, Clone, PartialEq)]
pub enum ParamTypeOut {
    Array(Type),
    Reference(Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    In { r#type: ParamTypeIn, ident: Ident },
    Out { r#type: ParamTypeOut, ident: Ident },
}

impl Param {
    pub fn r#type(&self) -> &Type {
        match self {
            Param::In { r#type, ident: _ } => match r#type {
                ParamTypeIn::Array(t) | ParamTypeIn::Value(t) => t,
            },
            Param::Out { r#type, ident: _ } => match r#type {
                ParamTypeOut::Array(t) | ParamTypeOut::Reference(t) => t,
            },
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
    Custom(String),
}

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        if let Ok(primitive) = Primitive::try_from(value) {
            Self::Primitive(primitive)
        } else {
            Self::Custom(value.to_string())
        }
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

impl Node {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| Error::Io(e, path.as_ref().display().to_string()))?;
        Self::from_string(path.as_ref().display().to_string(), content)
    }

    pub fn from_string<S: AsRef<str>>(root: String, s: S) -> Result<Self, Error> {
        let pst = Parser::parse(Rule::idl, s.as_ref())?;
        Ok(Node::from((root, pst)))
    }
}

impl<'a> From<(String, Pairs<'a, Rule>)> for Node {
    fn from(mut compile_unit: (String, Pairs<'a, Rule>)) -> Self {
        let idl = ast_unwrap!(compile_unit.1.next());
        assert_eq!(idl.as_rule(), Rule::idl);
        let mut nodes = Vec::new();

        for inner in idl.into_inner() {
            let span = Span {
                start: inner.as_span().start(),
                end: inner.as_span().end(),
            };
            match inner.as_rule() {
                Rule::include => {
                    let path = ast_unwrap!(inner.into_inner().next());
                    nodes.push(Node::Include(path.as_str().to_string()));
                }
                Rule::r#struct => {
                    let mut struct_pst = inner.into_inner();
                    let ident: Ident = ast_unwrap!(struct_pst.next()).into();
                    let mut fields = Vec::<StructField>::new();
                    for rule in struct_pst {
                        match rule.as_rule() {
                            Rule::struct_field => {
                                let mut iter = rule.into_inner();
                                let r#type = Type::from(ast_unwrap!(iter.next()).as_str());
                                let next = ast_unwrap!(iter.next());
                                let (elem, ident) = match next.as_rule() {
                                    Rule::array => {
                                        let array_len: Count = ast_unwrap!(next.as_str().parse());
                                        let ident = ast_unwrap!(iter.next()).as_str().to_string();
                                        (array_len, ident)
                                    }
                                    Rule::ident => {
                                        let ident = next.as_str().to_string();
                                        (unsafe { NonZeroU16::new_unchecked(1) }, ident)
                                    }
                                    _ => unreachable!(),
                                };

                                fields.push(StructField {
                                    ident: Ident {
                                        span: Span::from(next.as_span()),
                                        ident,
                                    },
                                    val: (r#type, elem),
                                });
                            }
                            Rule::COMMENT => {
                                // Currently unsupported for structs due to varying styles
                            }
                            _ => unreachable!(),
                        }
                    }
                    nodes.push(Node::Struct(Struct { ident, fields }));
                }
                Rule::r#const => {
                    nodes.push(Node::Const(Const::from(inner.into_inner())));
                }
                Rule::interface => {
                    let mut interface = inner.into_inner();
                    let mut pairs = ast_unwrap!(interface.next()).into_inner();
                    let ident = ast_unwrap!(pairs.next()).as_str().to_string();
                    let base = pairs.next().map(|base| Ident {
                        span: base.as_span().into(),
                        ident: base.as_str().to_string(),
                    });

                    let mut iface_nodes = Vec::new();
                    let mut comment: Option<Documentation> = None;
                    for rule in interface {
                        match rule.as_rule() {
                            Rule::r#const | Rule::function | Rule::error => {
                                let node = InterfaceNode::from((comment, rule));
                                comment = None;
                                iface_nodes.push(node);
                            }
                            Rule::COMMENT => {
                                comment = Documentation::try_from(rule).ok();
                            }
                            _ => unreachable!(),
                        }
                    }
                    nodes.push(Node::Interface(Interface {
                        ident: Ident { span, ident },
                        base,
                        nodes: iface_nodes,
                    }));
                }
                _ => {}
            }
        }

        Node::CompilationUnit(compile_unit.0, nodes)
    }
}

impl<'a> From<Pairs<'a, Rule>> for Const {
    fn from(mut inner: Pairs<'a, Rule>) -> Self {
        let r#type = Primitive::try_from(ast_unwrap!(inner.next()).as_str());
        let ident = ast_unwrap!(inner.next()).into();
        let value = ast_unwrap!(inner.next()).as_str().to_string();
        Const {
            ident,
            r#type: ast_unwrap!(r#type),
            value,
        }
    }
}

impl<'a> From<(Option<Documentation>, Pair<'a, Rule>)> for InterfaceNode {
    fn from(pair: (Option<Documentation>, Pair<'a, Rule>)) -> Self {
        let (doc, pair) = pair;
        match pair.as_rule() {
            Rule::error => InterfaceNode::Error(Ident {
                span: pair.as_span().into(),
                ident: pair.into_inner().as_str().to_string(),
            }),
            Rule::r#const => InterfaceNode::Const(Const::from(pair.into_inner())),
            Rule::function => {
                let mut inner = pair.into_inner();
                let ident = ast_unwrap!(inner.next()).into();
                let mut params = Vec::new();
                for param in inner {
                    params.push(Param::from(param));
                }
                InterfaceNode::Function { doc, ident, params }
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> From<Pair<'a, Rule>> for Param {
    fn from(value: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(value.as_rule(), Rule::param);
        let mut params = value.into_inner();
        let mutability = ast_unwrap!(params.next()).as_str();
        let r#type = ast_unwrap!(params.next());
        let ident = ast_unwrap!(params.next()).into();
        match mutability {
            "in" => {
                let r#type = ParamTypeIn::from(r#type);
                Param::In { r#type, ident }
            }
            "out" => {
                let r#type = ParamTypeOut::from(r#type);
                Param::Out { r#type, ident }
            }
            _ => unreachable!(),
        }
    }
}

impl From<Pair<'_, Rule>> for ParamTypeIn {
    fn from(rule: Pair<Rule>) -> Self {
        debug_assert_eq!(rule.as_rule(), Rule::param_type);
        let mut inner = rule.into_inner();
        let r#type = Type::from(ast_unwrap!(inner.next()).as_str());
        if let Type::Custom(r#type) = &r#type {
            if r#type == "buffer" {
                return ParamTypeIn::Array(Type::Primitive(Primitive::Uint8));
            }
        }

        if let Some(pair) = inner.next() {
            debug_assert_eq!(pair.as_rule(), Rule::param_arr);
            ParamTypeIn::Array(r#type)
        } else {
            ParamTypeIn::Value(r#type)
        }
    }
}
impl From<Pair<'_, Rule>> for ParamTypeOut {
    fn from(rule: Pair<Rule>) -> Self {
        debug_assert_eq!(rule.as_rule(), Rule::param_type);
        let mut inner = rule.into_inner();
        let r#type = Type::from(ast_unwrap!(inner.next()).as_str());
        if let Type::Custom(r#type) = &r#type {
            if r#type == "buffer" {
                return ParamTypeOut::Array(Type::Primitive(Primitive::Uint8));
            }
        }

        if let Some(pair) = inner.next() {
            debug_assert_eq!(pair.as_rule(), Rule::param_arr);
            ParamTypeOut::Array(r#type)
        } else {
            ParamTypeOut::Reference(r#type)
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Documentation {
    type Error = Error;
    fn try_from(value: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        debug_assert_eq!(value.as_rule(), Rule::COMMENT);
        let comment = value
            .into_inner()
            .next()
            .ok_or(Error::UnsupportedDocumentation)?;
        match comment.as_rule() {
            Rule::DOCUMENTATION => {
                let raw = comment.as_str();
                let window = raw[3..raw.len() - 2].trim();
                Ok(Documentation(window.to_string()))
            }
            _ => unreachable!(),
        }
    }
}

pub fn dump_pst<P: AsRef<Path>>(path: P) {
    use std::time::Instant;

    let inp = std::fs::read_to_string(path).unwrap();
    let now = Instant::now();
    let pst = Parser::parse(Rule::idl, &inp);
    let duration = now.elapsed();
    match pst {
        Ok(pst) => println!("{pst:#?}"),
        Err(e) => eprintln!("Parsing failed:\n{e}\n"),
    }
    eprintln!("'dump_pst' completed in {duration:?}");
}

pub fn dump<P: AsRef<Path>>(path: P) {
    use std::time::Instant;

    let now = Instant::now();
    let ast = Node::from_file(path).unwrap();
    let duration = now.elapsed();
    println!("{ast:#?}");
    eprintln!("'dump_ast' completed in {duration:?}");
}
