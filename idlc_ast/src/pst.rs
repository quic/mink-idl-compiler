// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::{path::PathBuf, rc::Rc};

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

// Import all AST types
use super::ast::{
    Const, Count, Documentation, Function, FunctionAttribute, Ident, Interface, InterfaceNode,
    Node, Param, ParamTypeIn, ParamTypeOut, Primitive, Span, Struct, StructField, Type,
};

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
    #[error("Cannot parse float")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Documentation for this node doesn't exist yet")]
    UnsupportedDocumentation,
    #[error("Parsed float translates to infinite")]
    FloatIsInfinite,
}
impl From<pest::error::Error<Rule>> for Error {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Self::AstGenerationFailure(Box::new(value))
    }
}

#[derive(Parser)]
#[grammar = "idl_grammar.pest"]
pub(crate) struct IDLParser;

macro_rules! ast_unwrap {
    ($e: expr) => {{
        #[cfg(debug_assertions)]
        {
            $e.unwrap()
        }

        #[cfg(not(debug_assertions))]
        // Safety: PST to AST is a 1-to-1 transition and can never fail.
        unsafe {
            ($e).unwrap_unchecked()
        }
    }};
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span) -> Self {
        Self {
            start: value.start(),
            end: value.end(),
        }
    }
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

impl InterfaceNode {
    fn new(
        doc: Option<Documentation>,
        pair: Pair<'_, Rule>,
        allow_undefined_behavior: bool,
    ) -> Self {
        match pair.as_rule() {
            Rule::error => Self::Error(Ident {
                span: pair.as_span().into(),
                ident: pair.into_inner().nth(1).unwrap().as_str().to_string(),
            }),
            Rule::r#const => Self::Const(parse_const(pair, allow_undefined_behavior)),
            Rule::function => {
                let mut inner = pair.into_inner();
                let mut attributes = Vec::new();
                let function_keyword = ast_unwrap!(inner.next());
                for attribute in function_keyword.into_inner() {
                    if attribute.as_rule() != Rule::attribute {
                        break;
                    }

                    let span = attribute.as_span();
                    let attr = FunctionAttribute::from(attribute);
                    if attributes.contains(&attr) {
                        idlc_errors::unrecoverable!("Duplicate attribute at:\n`{span:#?}`");
                    } else {
                        attributes.push(attr);
                    }
                }
                let ident = ast_unwrap!(inner.next()).into();
                let mut params = Vec::new();
                for param in inner {
                    if param.as_rule() == Rule::param {
                        params.push(Param::from(param));
                    }
                }
                Self::Function(Function {
                    doc,
                    ident,
                    params,
                    attributes,
                })
            }
            _ => unreachable!(),
        }
    }
}

impl<'a> From<Pair<'a, Rule>> for FunctionAttribute {
    fn from(value: Pair<'a, Rule>) -> Self {
        debug_assert_eq!(value.as_rule(), Rule::attribute);
        let attribute = ast_unwrap!(value.into_inner().next());
        debug_assert_eq!(attribute.as_rule(), Rule::supported_attributes);
        match attribute.as_str() {
            "optional" => Self::Optional,
            attr => {
                idlc_errors::unrecoverable!("Unknown function attribute `{attr}`")
            }
        }
    }
}

impl<'a> From<Pair<'a, Rule>> for Param {
    fn from(value: Pair<'a, Rule>) -> Self {
        let mut params = value.into_inner();
        let mutability = ast_unwrap!(params.next()).as_str();
        let r#type = ast_unwrap!(params.next());
        let ident = ast_unwrap!(params.next()).into();
        match mutability {
            "in" => {
                let r#type = ParamTypeIn::from(r#type);
                Self::In { r#type, ident }
            }
            "out" => {
                let r#type = ParamTypeOut::from(r#type);
                Self::Out { r#type, ident }
            }
            _ => unreachable!(),
        }
    }
}

impl From<Pair<'_, Rule>> for Type {
    fn from(value: Pair<'_, Rule>) -> Self {
        Primitive::try_from(value.as_str()).map_or_else(
            |_| {
                if value.as_str() == "interface" {
                    Self::Interface
                } else {
                    Self::Custom(Ident {
                        span: Span::from(value.as_span()),
                        ident: value.as_str().to_string(),
                    })
                }
            },
            Self::Primitive,
        )
    }
}

impl From<Pair<'_, Rule>> for ParamTypeIn {
    fn from(rule: Pair<Rule>) -> Self {
        debug_assert_eq!(rule.as_rule(), Rule::param_type);
        let mut inner = rule.into_inner();
        let r#type = Type::from(ast_unwrap!(inner.next()));
        if let Type::Custom(r#type) = &r#type {
            if r#type == "buffer" {
                return Self::Value(Type::UntypedBuffer);
            }
        }

        if let Some(pair) = inner.next() {
            match pair.as_rule() {
                Rule::unbounded_array => Self::Array(r#type, None),
                Rule::bounded_array => {
                    let array_len: Count = ast_unwrap!(pair.into_inner().as_str().parse());
                    Self::Array(r#type, Some(array_len))
                }
                _ => unreachable!(),
            }
        } else {
            Self::Value(r#type)
        }
    }
}
impl From<Pair<'_, Rule>> for ParamTypeOut {
    fn from(rule: Pair<Rule>) -> Self {
        debug_assert_eq!(rule.as_rule(), Rule::param_type);
        let mut inner = rule.into_inner();
        let r#type = Type::from(ast_unwrap!(inner.next()));
        if let Type::Custom(r#type) = &r#type {
            if r#type == "buffer" {
                return Self::Reference(Type::UntypedBuffer);
            }
        }

        if let Some(pair) = inner.next() {
            match pair.as_rule() {
                Rule::unbounded_array => Self::Array(r#type, None),
                Rule::bounded_array => {
                    let array_len: Count = ast_unwrap!(pair.into_inner().as_str().parse());
                    Self::Array(r#type, Some(array_len))
                }
                _ => unreachable!(),
            }
        } else {
            Self::Reference(r#type)
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
                let raw = comment.as_str().trim();
                // removes leading `/*\n` and trailing '/'. This is guaranteed to exist since
                // grammar only accepts doxygen style comments.
                let window = &raw[2..raw.len() - 1];
                Ok(Self(window.to_string()))
            }
            _ => unreachable!(),
        }
    }
}

fn parse_include(pair: Pair<Rule>) -> Rc<Node> {
    let path = ast_unwrap!(pair.into_inner().next());
    Rc::new(Node::Include(PathBuf::from(path.as_str())))
}

fn parse_struct(pair: Pair<Rule>) -> Rc<Node> {
    let mut struct_pst = pair.into_inner().skip(1);
    let ident: Ident = ast_unwrap!(struct_pst.next()).into();
    let mut fields = Vec::<StructField>::new();
    for rule in struct_pst {
        match rule.as_rule() {
            Rule::struct_field => {
                let mut iter = rule.into_inner();
                let r#type = Type::from(ast_unwrap!(iter.next()));
                let next = ast_unwrap!(iter.next());
                let (elem, ident) = match next.as_rule() {
                    Rule::bounded_array => {
                        let array_len: Count =
                            ast_unwrap!(next.clone().into_inner().as_str().parse());
                        let ident = ast_unwrap!(iter.next()).as_str().to_string();
                        (array_len, ident)
                    }
                    Rule::ident => {
                        let ident = next.as_str().to_string();
                        (unsafe { Count::new_unchecked(1) }, ident)
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
            r => unreachable!("Unknown rule `{r:?}`"),
        }
    }
    Rc::new(Node::Struct(Struct { ident, fields }))
}

fn parse_const(pair: Pair<Rule>, allow_undefined_behavior: bool) -> Const {
    let mut inner = pair.into_inner().skip(1);

    let ty = ast_unwrap!(inner.next()).as_str();
    let ident = ast_unwrap!(inner.next()).into();
    let value = ast_unwrap!(inner.next()).as_str();
    let primitive = if allow_undefined_behavior {
        Primitive::try_from(ty).unwrap()
    } else {
        Primitive::new(ty, value).unwrap_or_else(|e| {
            idlc_errors::unrecoverable!("'{value}' isn't in range for type '{ty}' [{e}]")
        })
    };

    Const {
        ident,
        r#type: primitive,
        value: value.to_string(),
    }
}

fn parse_interface(pair: Pair<Rule>, allow_undefined_behavior: bool) -> Rc<Node> {
    let span = Span::from(pair.as_span());
    let mut interface = pair.into_inner().skip(1);
    let mut pairs = ast_unwrap!(interface.next()).into_inner();
    let ident = ast_unwrap!(pairs.next()).as_str().to_string();
    let base = pairs
        .next()
        .filter(|base| base.as_rule() == Rule::ident)
        .map(|base| Ident {
            span: base.as_span().into(),
            ident: base.as_str().to_string(),
        });

    let mut iface_nodes = Vec::new();
    let mut comment: Option<Documentation> = None;
    for rule in interface {
        match rule.as_rule() {
            Rule::r#const | Rule::function | Rule::error => {
                let node = InterfaceNode::new(comment, rule, allow_undefined_behavior);
                comment = None;
                iface_nodes.push(node);
            }
            Rule::COMMENT => {
                comment = Documentation::try_from(rule).ok();
            }
            _ => unreachable!(),
        }
    }
    Rc::new(Node::Interface(Interface {
        ident: Ident { span, ident },
        base,
        nodes: iface_nodes,
    }))
}

pub fn parse_to_ast(input: &str, allow_undefined_behavior: bool) -> Result<Vec<Rc<Node>>, Error> {
    let mut pairs = IDLParser::parse(Rule::idl, input)?;
    let mut nodes = Vec::new();

    for p in pairs.next().unwrap().into_inner() {
        match p.as_rule() {
            Rule::include => nodes.push(parse_include(p)),
            Rule::r#struct => nodes.push(parse_struct(p)),
            Rule::r#const => nodes.push(Rc::new(Node::Const(parse_const(
                p,
                allow_undefined_behavior,
            )))),
            Rule::interface => nodes.push(parse_interface(p, allow_undefined_behavior)),
            Rule::EOI => (),
            _ => {}
        }
    }
    Ok(nodes)
}

pub fn dump<P: AsRef<std::path::Path>>(path: P) {
    use std::time::Instant;

    let inp = std::fs::read_to_string(path).unwrap();
    let now = Instant::now();
    let pst = IDLParser::parse(Rule::idl, &inp);
    let duration = now.elapsed();
    match pst {
        Ok(pst) => println!("{pst:#?}"),
        Err(e) => eprintln!("Parsing failed:\n{e}\n"),
    }
    eprintln!("'dump_pst' completed in {duration:?}");
}
