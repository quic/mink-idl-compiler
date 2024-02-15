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
//! All codegen backends that depend on this MIR are obligated to use all of
//! it's information to ensure the specification of Mink is upheld.
//!
//! # Duplication
//! If you notice, a lot of things that are duplicated from [`idlc_ast`] and
//! this observation is correct, this is done to avoid the codegen backends
//! directly depending on AST and for MIR to produce an interface to shield
//! codegens from AST changes; AST changes tomorrow which don't require MIR
//! changes should not require codegen changes
use idlc_ast::Ast;
pub use idlc_ast::Ident;
use idlc_ast_passes::idl_store::IDLStore;

use std::path::{Path, PathBuf};
use std::rc::Rc;

/// Code from 0 to 9 are reserved for generic IDL-generated code, so starting from 10.
const ERROR_CODE_START: i32 = 10;
/// User defined method op-codes can range from 0 - 0x3FFF (inclusive) as defined by the Mink specification.
const MAX_OP_CODE: u32 = 0x3fff;

#[derive(Debug, Clone, PartialEq)]
/// Represents the Mink specifications over the source AST.
pub struct Mir {
    /// Tag denoting the AST name.
    ///
    /// This doesn't have to be unique.
    pub tag: PathBuf,
    /// Root node for the [`Mir`] tree.
    pub nodes: Vec<Rc<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Include(PathBuf),
    Const(Const),
    Struct(Struct),
    Interface(Interface),
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
    #[inline]
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::Uint8 | Self::Int8 => 1,
            Self::Uint16 | Self::Int16 => 2,
            Self::Uint32 | Self::Int32 | Self::Float32 => 4,
            Self::Uint64 | Self::Int64 | Self::Float64 => 8,
        }
    }
}

impl Ord for Primitive {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size().cmp(&other.size())
    }
}
impl PartialOrd for Primitive {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Const {
    pub ident: Ident,
    pub r#type: Primitive,
    pub value: String,
}

pub type Count = std::num::NonZeroU16;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Primitive(Primitive),
    Struct(Struct),
    Interface(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub ident: Ident,
    pub val: (Type, Count),
}

impl StructField {
    pub fn size(&self) -> usize {
        let count = self.val.1;
        let size = match &self.val.0 {
            Type::Primitive(p) => p.size(),
            Type::Struct(s) => s.as_ref().size(),
            Type::Interface(_) => Primitive::Uint64.size() * 2,
        };

        size * usize::from(count.get())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Struct {
    /// This variant of struct can be bundled with other primitives as it's
    /// guaranteed to be less than [`Struct::BUNDLED_SIZE_MAX`] bytes long.
    Small(StructInner),
    Big(StructInner),
}

impl From<StructInner> for Struct {
    fn from(value: StructInner) -> Self {
        let size = value.size();
        Self::new(value, size)
    }
}

impl Struct {
    /// A bundle is a collection of parameter values that are sent or received
    /// as a single invoke argument. Data parameters that are of a fixed size
    /// less than or equal to 16 bytes are called small parameters and may be
    /// bundled.  If there are two or more small input parameters, then all
    /// small input parameters are placed in an input bundle which is sent as
    /// the first input buffer argument. Otherwise, no input bundle is sent, and
    /// all input data parameters are sent as discrete arguments.
    pub const BUNDLED_SIZE_MAX: usize = 16;

    #[inline]
    pub const fn new(inner: StructInner, size: usize) -> Self {
        if size <= Self::BUNDLED_SIZE_MAX {
            Self::Small(inner)
        } else {
            Self::Big(inner)
        }
    }
}
impl AsRef<StructInner> for Struct {
    #[inline]
    fn as_ref(&self) -> &StructInner {
        match self {
            Self::Small(s) | Self::Big(s) => s,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructInner {
    pub ident: Ident,
    pub fields: Vec<StructField>,
    pub origin: Option<PathBuf>,
}

impl StructInner {
    #[inline]
    pub fn size(&self) -> usize {
        self.fields.iter().fold(0, |acc, e| acc + e.size())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub ident: Ident,
    pub base: Option<Rc<Interface>>,
    pub nodes: Vec<InterfaceNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceNode {
    Const(Const),
    Function(Function),
    Error(Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamTypeIn {
    Array(Type, Option<Count>),
    Value(Type),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamTypeOut {
    Array(Type, Option<Count>),
    Reference(Type),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    In { r#type: ParamTypeIn, ident: Ident },
    Out { r#type: ParamTypeOut, ident: Ident },
}
impl Param {
    #[inline]
    #[must_use]
    pub const fn r#type(&self) -> &Type {
        match self {
            Self::In { r#type, ident: _ } => match r#type {
                ParamTypeIn::Array(t, _) | ParamTypeIn::Value(t) => t,
            },
            Self::Out { r#type, ident: _ } => match r#type {
                ParamTypeOut::Array(t, _) | ParamTypeOut::Reference(t) => t,
            },
        }
    }

    #[inline]
    #[must_use]
    pub const fn ident(&self) -> &Ident {
        match self {
            Self::In { r#type: _, ident } | Self::Out { r#type: _, ident } => ident,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_input(&self) -> bool {
        matches!(
            self,
            Self::In {
                r#type: _,
                ident: _
            }
        )
    }

    #[inline]
    #[must_use]
    pub const fn is_output(&self) -> bool {
        matches!(
            self,
            Self::Out {
                r#type: _,
                ident: _
            }
        )
    }

    #[must_use]
    pub const fn is_array(&self) -> bool {
        matches!(
            self,
            Self::In {
                r#type: ParamTypeIn::Array(_, _),
                ident: _,
            } | Self::Out {
                r#type: ParamTypeOut::Array(_, _),
                ident: _,
            }
        )
    }

    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        matches!(self.r#type(), Type::Primitive(_))
    }

    #[must_use]
    pub const fn is_primitive_value(&self) -> bool {
        !self.is_array() && self.is_primitive()
    }

    #[must_use]
    pub const fn is_small_struct(&self) -> bool {
        matches!(self.r#type(), Type::Struct(Struct::Small(_)))
    }

    #[must_use]
    pub const fn is_small_struct_value(&self) -> bool {
        !self.is_array() && self.is_small_struct()
    }
}

impl PartialOrd for Param {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Param {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (
                Self::In {
                    r#type: _,
                    ident: _,
                },
                Self::Out {
                    r#type: _,
                    ident: _,
                },
            ) => match (self.r#type(), other.r#type()) {
                (Type::Interface(_), Type::Interface(_)) => {
                    if self.is_array() && !other.is_array() {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Less
                    }
                }
                (Type::Primitive(_) | Type::Struct(_), _) => std::cmp::Ordering::Less,
                (Type::Interface(_), Type::Primitive(_) | Type::Struct(_)) => {
                    std::cmp::Ordering::Greater
                }
            },
            (
                Self::Out {
                    r#type: _,
                    ident: _,
                },
                Self::In {
                    r#type: _,
                    ident: _,
                },
            ) => match (self.r#type(), other.r#type()) {
                (Type::Interface(_), Type::Interface(_)) => {
                    if !self.is_array() && other.is_array() {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }
                (Type::Interface(_), Type::Primitive(_) | Type::Struct(_)) => {
                    std::cmp::Ordering::Greater
                }
                (Type::Primitive(_) | Type::Struct(_), Type::Interface(_)) => {
                    std::cmp::Ordering::Less
                }
                _ => std::cmp::Ordering::Greater,
            },
            _ => match (self.r#type(), other.r#type()) {
                (Type::Primitive(_), Type::Interface(_)) => std::cmp::Ordering::Less,
                (Type::Struct(_), Type::Interface(_)) => std::cmp::Ordering::Less,
                (Type::Interface(_), Type::Interface(_)) => {
                    if self.is_array() && !other.is_array() {
                        std::cmp::Ordering::Greater
                    } else if !self.is_array() && other.is_array() {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }
                (Type::Interface(_), _) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub doc: Option<String>,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub ident: Ident,
    pub value: i32,
}

fn parse_include(path: &Path) -> Rc<Node> {
    Rc::new(Node::Include(path.to_path_buf()))
}

fn parse_const(const_: &idlc_ast::Const) -> Rc<Node> {
    Rc::new(Node::Const(Const::from(const_)))
}

fn parse_struct(struct_: &idlc_ast::Struct, idl_store: &IDLStore) -> Rc<Node> {
    let ident = struct_.ident.clone();
    let mut fields = Vec::<StructField>::new();
    let mut size = 0;
    for field in &struct_.fields {
        let val = (Type::new(&field.val.0, idl_store), field.val.1);
        let field = StructField {
            ident: field.ident.clone(),
            val,
        };
        size += field.size();
        fields.push(field);
    }

    Rc::new(Node::Struct(Struct::new(
        StructInner {
            ident,
            fields,
            origin: None,
        },
        size,
    )))
}

fn parse_interface(
    interface_: &idlc_ast::Interface,
    idl_store: &mut IDLStore,
    error_code: &mut i32,
    op_code: &mut u32,
) -> Interface {
    let class = interface_.ident.clone();
    let base = interface_
        .base
        .as_ref()
        .map(std::string::ToString::to_string);

    let mut iface_nodes = Vec::new();
    let base_node = base.map(|x| {
        parse_interface(
            &idl_store.iface_lookup(&x).unwrap(),
            idl_store,
            error_code,
            op_code,
        )
    });

    for node in &interface_.nodes {
        match node {
            idlc_ast::InterfaceNode::Const(const_) => {
                iface_nodes.push(InterfaceNode::Const(Const::from(const_)));
            }
            idlc_ast::InterfaceNode::Error(error) => {
                iface_nodes.push(InterfaceNode::Error(Error {
                    ident: error.clone(),
                    value: *error_code,
                }));
                *error_code = error_code
                    .checked_add(1)
                    .expect("Error should be under i32::MAX");
            }
            idlc_ast::InterfaceNode::Function(function) => {
                let doc = function
                    .doc
                    .as_ref()
                    .map(|idlc_ast::Documentation(s)| s.to_string());
                let ident = function.ident.clone();
                let mut params = Vec::new();
                for param in &function.params {
                    params.push(Param::new(param, idl_store));
                }
                iface_nodes.push(InterfaceNode::Function(Function {
                    doc,
                    ident,
                    params,
                    id: *op_code,
                }));
                if *op_code > MAX_OP_CODE {
                    panic!("Numbers of functions should be lesser than 0x3fff");
                } else {
                    *op_code += 1;
                }
            }
        }
    }

    Interface {
        ident: class,
        base: base_node.map(Rc::new),
        nodes: iface_nodes,
    }
}

pub fn parse_to_mir(ast: &Ast, idl_store: &mut IDLStore) -> Mir {
    let mut nodes = Vec::new();
    for node in &ast.nodes {
        match &**node {
            idlc_ast::Node::Include(path) => nodes.push(parse_include(path)),
            idlc_ast::Node::Const(const_) => nodes.push(parse_const(const_)),
            idlc_ast::Node::Struct(struct_) => nodes.push(parse_struct(struct_, idl_store)),
            idlc_ast::Node::Interface(interface) => {
                let mut err_code = ERROR_CODE_START;
                let mut op_code = 0;
                nodes.push(Rc::new(Node::Interface(parse_interface(
                    interface,
                    idl_store,
                    &mut err_code,
                    &mut op_code,
                ))));
            }
        }
    }

    Mir {
        tag: ast.tag.clone(),
        nodes,
    }
}

impl ParamTypeIn {
    fn new(src: &idlc_ast::ParamTypeIn, idl_store: &IDLStore) -> Self {
        match src {
            idlc_ast::ParamTypeIn::Array(ty, cnt) => Self::Array(Type::new(ty, idl_store), *cnt),
            idlc_ast::ParamTypeIn::Value(ty) => Self::Value(Type::new(ty, idl_store)),
        }
    }
}

impl ParamTypeOut {
    fn new(src: &idlc_ast::ParamTypeOut, idl_store: &IDLStore) -> Self {
        match src {
            idlc_ast::ParamTypeOut::Array(ty, cnt) => Self::Array(Type::new(ty, idl_store), *cnt),
            idlc_ast::ParamTypeOut::Reference(ty) => Self::Reference(Type::new(ty, idl_store)),
        }
    }
}

impl Param {
    fn new(src: &idlc_ast::Param, idl_store: &IDLStore) -> Self {
        match src {
            idlc_ast::Param::In { r#type, ident } => Self::In {
                r#type: ParamTypeIn::new(r#type, idl_store),
                ident: ident.clone(),
            },
            idlc_ast::Param::Out { r#type, ident } => Self::Out {
                r#type: ParamTypeOut::new(r#type, idl_store),
                ident: ident.clone(),
            },
        }
    }
}

impl From<&idlc_ast::Primitive> for Primitive {
    fn from(prim: &idlc_ast::Primitive) -> Self {
        match prim {
            idlc_ast::Primitive::Uint8 => Self::Uint8,
            idlc_ast::Primitive::Uint16 => Self::Uint16,
            idlc_ast::Primitive::Uint32 => Self::Uint32,
            idlc_ast::Primitive::Uint64 => Self::Uint64,
            idlc_ast::Primitive::Int8 => Self::Int8,
            idlc_ast::Primitive::Int16 => Self::Int16,
            idlc_ast::Primitive::Int32 => Self::Int32,
            idlc_ast::Primitive::Int64 => Self::Int64,
            idlc_ast::Primitive::Float32 => Self::Float32,
            idlc_ast::Primitive::Float64 => Self::Float64,
        }
    }
}

impl From<&idlc_ast::Const> for Const {
    fn from(const_: &idlc_ast::Const) -> Self {
        Self {
            ident: const_.ident.clone(),
            r#type: Primitive::from(&const_.r#type),
            value: const_.value.to_string(),
        }
    }
}

impl Type {
    fn new(ty: &idlc_ast::Type, idl_store: &IDLStore) -> Self {
        match ty {
            idlc_ast::Type::Primitive(primitive) => Self::Primitive(Primitive::from(primitive)),
            idlc_ast::Type::Interface => Self::Interface(None),
            idlc_ast::Type::Custom(custom) => {
                let ident = &custom.ident;
                idl_store.iface_lookup(ident).map_or_else(
                    || match idl_store.struct_lookup(ident) {
                        Some((r#struct, path)) => {
                            let mut fields = Vec::new();
                            let mut size = 0;
                            for field in &r#struct.fields {
                                let (ty, count) = &field.val;
                                let field = StructField {
                                    ident: field.ident.clone(),
                                    val: (Self::new(ty, idl_store), *count),
                                };
                                size += field.size();
                                fields.push(field);
                            }
                            Self::Struct(Struct::new(
                                StructInner {
                                    ident: r#struct.ident.clone(),
                                    fields,
                                    origin: Some(path),
                                },
                                size,
                            ))
                        }
                        None => panic!("Couldn't find any references of symbol {ident}"),
                    },
                    |iface| Self::Interface(Some(iface.ident.to_string())),
                )
            }
        }
    }
}

pub struct InterfaceIterator<'a> {
    interface: Option<&'a Interface>,
}

impl<'a> IntoIterator for &'a Interface {
    type Item = &'a Interface;
    type IntoIter = InterfaceIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        InterfaceIterator {
            interface: Some(self),
        }
    }
}

impl<'a> Iterator for InterfaceIterator<'a> {
    type Item = &'a Interface;

    fn next(&mut self) -> Option<Self::Item> {
        match self.interface {
            Some(base) => {
                self.interface = base.base.as_deref();
                Some(base)
            }
            None => None,
        }
    }
}

impl Interface {
    #[inline]
    #[must_use]
    pub const fn iter(&self) -> InterfaceIterator {
        InterfaceIterator {
            interface: Some(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_errors_only_of_base() {
        let iface = Interface {
            ident: Ident::new_without_span("A".to_string()),
            base: Some(Rc::new(Interface {
                ident: Ident::new_without_span("B".to_string()),
                base: Some(Rc::new(Interface {
                    ident: Ident::new_without_span("C".to_string()),
                    base: None,
                    nodes: vec![
                        InterfaceNode::Error(Error {
                            ident: Ident::new_without_span("ERROR_1".to_string()),
                            value: 10,
                        }),
                        InterfaceNode::Const(Const {
                            ident: Ident::new_without_span("CONST_1".to_string()),
                            r#type: Primitive::Uint8,
                            value: "10".to_string(),
                        }),
                    ],
                })),
                nodes: vec![InterfaceNode::Error(Error {
                    ident: Ident::new_without_span("ERROR_SOMETHING_ELSE".to_string()),
                    value: 10,
                })],
            })),
            nodes: vec![InterfaceNode::Error(Error {
                ident: Ident::new_without_span("THIS_SHOULDNT_SHOW_UP".to_string()),
                value: 10,
            })],
        };
        let error_iterator = iface.iter().skip(1).flat_map(|iface| {
            iface.nodes.iter().filter_map(|node| {
                let InterfaceNode::Error(e) = node else {
                    return None;
                };
                Some((iface.ident.ident.as_str(), e))
            })
        });
        let out: Vec<(&str, &Error)> = error_iterator.collect();
        assert_eq!(
            out,
            [
                (
                    "B",
                    &Error {
                        ident: Ident::new_without_span("ERROR_SOMETHING_ELSE".to_string()),
                        value: 10
                    }
                ),
                (
                    "C",
                    &Error {
                        ident: Ident::new_without_span("ERROR_1".to_string()),
                        value: 10
                    }
                )
            ]
        );
    }

    #[test]
    fn sorting_params() {
        let mut params = [
            Param::Out {
                r#type: ParamTypeOut::Reference(Type::Interface(None)),
                ident: Ident::new_without_span("interface3".to_string()),
            },
            Param::Out {
                r#type: ParamTypeOut::Array(Type::Primitive(Primitive::Uint16), None),
                ident: Ident::new_without_span("primitive4".to_string()),
            },
            Param::Out {
                r#type: ParamTypeOut::Array(Type::Interface(None), None),
                ident: Ident::new_without_span("interface6".to_string()),
            },
            Param::Out {
                r#type: ParamTypeOut::Reference(Type::Interface(None)),
                ident: Ident::new_without_span("interface4".to_string()),
            },
            Param::Out {
                r#type: ParamTypeOut::Array(Type::Primitive(Primitive::Uint16), None),
                ident: Ident::new_without_span("primitive5".to_string()),
            },
            Param::Out {
                r#type: ParamTypeOut::Array(
                    Type::Struct(
                        StructInner {
                            ident: Ident::new_without_span(String::new()),
                            fields: Vec::new(),
                            origin: None,
                        }
                        .into(),
                    ),
                    None,
                ),
                ident: Ident::new_without_span("struct3".to_string()),
            },
            Param::Out {
                r#type: ParamTypeOut::Array(
                    Type::Struct(
                        StructInner {
                            ident: Ident::new_without_span(String::new()),
                            fields: Vec::new(),
                            origin: None,
                        }
                        .into(),
                    ),
                    None,
                ),
                ident: Ident::new_without_span("struct4".to_string()),
            },
            Param::In {
                r#type: ParamTypeIn::Array(Type::Interface(None), None),
                ident: Ident::new_without_span("interface5".to_string()),
            },
            Param::In {
                r#type: ParamTypeIn::Value(Type::Interface(None)),
                ident: Ident::new_without_span("interface1".to_string()),
            },
            Param::In {
                r#type: ParamTypeIn::Array(Type::Primitive(Primitive::Uint16), None),
                ident: Ident::new_without_span("primitive1".to_string()),
            },
            Param::In {
                r#type: ParamTypeIn::Value(Type::Interface(None)),
                ident: Ident::new_without_span("interface2".to_string()),
            },
            Param::In {
                r#type: ParamTypeIn::Value(Type::Primitive(Primitive::Uint16)),
                ident: Ident::new_without_span("primitive2".to_string()),
            },
            Param::In {
                r#type: ParamTypeIn::Array(
                    Type::Struct(
                        StructInner {
                            ident: Ident::new_without_span(String::new()),
                            fields: Vec::new(),
                            origin: None,
                        }
                        .into(),
                    ),
                    None,
                ),
                ident: Ident::new_without_span("struct1".to_string()),
            },
            Param::In {
                r#type: ParamTypeIn::Value(Type::Primitive(Primitive::Float32)),
                ident: Ident::new_without_span("primitive3".to_string()),
            },
        ];
        params.sort();
        assert_eq!(
            params
                .iter()
                .map(|x| x.ident().ident.as_str())
                .collect::<Vec<_>>(),
            [
                "primitive1",
                "primitive2",
                "struct1",
                "primitive3",
                "primitive4",
                "primitive5",
                "struct3",
                "struct4",
                "interface1",
                "interface2",
                "interface3",
                "interface4",
                "interface5",
                "interface6"
            ]
        );
    }
}
