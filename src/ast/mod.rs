#[cfg(test)]
mod tests;

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
        unsafe { ($e).unwrap_unchecked() }
    };
}

/// Identifiers are utf-8 strings.
type Ident = String;
/// Maximum allowed size for a struct array [`u16::MAX`]
type Count = NonZeroU16;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Include(String),
    Const(Const),
    Struct {
        ident: Ident,
        fields: Vec<StructField>,
    },
    Interface {
        name: Ident,
        base: Option<Ident>,
        nodes: Vec<InterfaceNode>,
    },
    CompilationUnit(String, Vec<Node>),
}
impl Node {
    pub fn ident(&self) -> Option<&Ident> {
        match self {
            Node::Const(c) => Some(c.ident()),
            Node::Struct { ident, fields: _ } => Some(ident),
            Node::Interface {
                name,
                base: _,
                nodes: _,
            } => Some(name),
            Node::CompilationUnit(root, _) => Some(root),
            Node::Include(_) => None,
        }
    }

    pub fn r#type(&self) -> &'static str {
        match self {
            Node::Include(_) => "include",
            Node::Const(_) => "const",
            Node::Struct {
                ident: _,
                fields: _,
            } => "struct",
            Node::Interface {
                name: _,
                base: _,
                nodes: _,
            } => "interface",
            Node::CompilationUnit(_, _) => "Unit",
        }
    }
}

pub trait Identifiable {
    fn ident(&self) -> &Ident;
}

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Documentation(String);
impl Documentation {
    pub fn doc(&self) -> &String {
        &self.0
    }
}

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
    ident: Ident,
    r#type: Primitive,
    value: String,
}
impl Identifiable for Const {
    fn ident(&self) -> &Ident {
        &self.ident
    }
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
impl Identifiable for Param {
    fn ident(&self) -> &Ident {
        match self {
            Param::In { r#type: _, ident } | Param::Out { r#type: _, ident } => ident,
        }
    }
}
impl Param {
    pub fn r#type(&self) -> &Type {
        match self {
            Param::In { r#type, ident: _ } => match r#type {
                ParamTypeIn::Array(t) | ParamTypeIn::Value(t) => t,
            },
            Param::Out { r#type, ident } => match r#type {
                ParamTypeOut::Array(t) | ParamTypeOut::Reference(t) => t,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    ident: Ident,
    val: (Type, Count),
}
impl Identifiable for StructField {
    fn ident(&self) -> &Ident {
        &self.ident
    }
}
impl StructField {
    pub fn r#type(&self) -> &(Type, Count) {
        &self.val
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(Primitive),
    Ident(Ident),
}

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        if let Ok(primitive) = Primitive::try_from(value) {
            Self::Primitive(primitive)
        } else {
            Self::Ident(value.to_string())
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
            match inner.as_rule() {
                Rule::include => {
                    let path = ast_unwrap!(inner.into_inner().next());
                    nodes.push(Node::Include(path.as_str().to_string()));
                }
                Rule::r#struct => {
                    let mut struct_pst = inner.into_inner();
                    let struct_name = ast_unwrap!(struct_pst.next()).as_str().to_string();
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
                                    ident,
                                    val: (r#type, elem),
                                });
                            }
                            Rule::COMMENT => {
                                // Currently unsupported for structs due to varying styles
                            }
                            _ => unreachable!(),
                        }
                    }
                    nodes.push(Node::Struct {
                        ident: struct_name,
                        fields,
                    });
                }
                Rule::r#const => {
                    nodes.push(Node::Const(Const::from(inner.into_inner())));
                }
                Rule::interface => {
                    let mut interface = inner.into_inner();
                    let mut pairs = ast_unwrap!(interface.next()).into_inner();
                    let name = ast_unwrap!(pairs.next()).as_str().to_string();
                    let base = pairs.next().map(|base| base.as_str().to_string());

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
                    nodes.push(Node::Interface {
                        name,
                        base,
                        nodes: iface_nodes,
                    });
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
        let ident = ast_unwrap!(inner.next()).as_str().to_string();
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
            Rule::error => InterfaceNode::Error(pair.into_inner().as_str().to_string()),
            Rule::r#const => InterfaceNode::Const(Const::from(pair.into_inner())),
            Rule::function => {
                let mut inner = pair.into_inner();
                let ident = ast_unwrap!(inner.next()).as_str().to_string();
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
        let ident = ast_unwrap!(params.next()).as_str().to_string();
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
        if let Type::Ident(r#type) = &r#type {
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
        if let Type::Ident(r#type) = &r#type {
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
