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
    pub fn size(self) -> usize {
        match self {
            Primitive::Uint8 | Primitive::Int8 => 1,
            Primitive::Uint16 | Primitive::Int16 => 2,
            Primitive::Uint32 | Primitive::Int32 | Primitive::Float32 => 4,
            Primitive::Uint64 | Primitive::Int64 | Primitive::Float64 => 8,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub ident: Ident,
    pub r#type: Primitive,
    pub value: String,
}

pub type Count = std::num::NonZeroU16;
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(Primitive),
    Interface(Option<String>),
    Struct(Struct),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub ident: Ident,
    pub val: (Type, Count),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub ident: Ident,
    pub fields: Vec<StructField>,
    pub origin: Option<PathBuf>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub doc: Option<String>,
    pub ident: Ident,
    pub params: Vec<Param>,
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq)]
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

fn parse_struct(struct_: &idlc_ast::Struct, idl_store: &mut IDLStore) -> Rc<Node> {
    let ident = struct_.ident.clone();
    let mut fields = Vec::<StructField>::new();
    for field in struct_.fields.iter() {
        let val = (Type::new(&field.val.0, idl_store), field.val.1);
        fields.push(StructField {
            ident: field.ident.clone(),
            val,
        });
    }
    Rc::new(Node::Struct(Struct {
        ident,
        fields,
        origin: None,
    }))
}

fn parse_interface(
    interface_: &idlc_ast::Interface,
    idl_store: &mut IDLStore,
    error_code: &mut i32,
    op_code: &mut u32,
) -> Interface {
    let class = interface_.ident.clone();
    let base = interface_.base.as_ref().map(|base| base.to_string());

    let mut iface_nodes = Vec::new();
    let base_node = base.map(|x| {
        parse_interface(
            &idl_store.iface_lookup(&x).unwrap(),
            idl_store,
            error_code,
            op_code,
        )
    });

    for node in interface_.nodes.iter() {
        match node {
            idlc_ast::InterfaceNode::Const(const_) => {
                iface_nodes.push(InterfaceNode::Const(Const::from(const_)))
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
                for param in function.params.iter() {
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
            idlc_ast::ParamTypeIn::Array(ty) => ParamTypeIn::Array(Type::new(ty, idl_store)),
            idlc_ast::ParamTypeIn::Value(ty) => ParamTypeIn::Value(Type::new(ty, idl_store)),
        }
    }
}

impl ParamTypeOut {
    fn new(src: &idlc_ast::ParamTypeOut, idl_store: &IDLStore) -> Self {
        match src {
            idlc_ast::ParamTypeOut::Array(ty) => ParamTypeOut::Array(Type::new(ty, idl_store)),
            idlc_ast::ParamTypeOut::Reference(ty) => {
                ParamTypeOut::Reference(Type::new(ty, idl_store))
            }
        }
    }
}

impl Param {
    fn new(src: &idlc_ast::Param, idl_store: &IDLStore) -> Self {
        match src {
            idlc_ast::Param::In { r#type, ident } => Param::In {
                r#type: ParamTypeIn::new(r#type, idl_store),
                ident: ident.clone(),
            },
            idlc_ast::Param::Out { r#type, ident } => Param::Out {
                r#type: ParamTypeOut::new(r#type, idl_store),
                ident: ident.clone(),
            },
        }
    }
}

impl From<&idlc_ast::Primitive> for Primitive {
    fn from(prim: &idlc_ast::Primitive) -> Self {
        match prim {
            idlc_ast::Primitive::Uint8 => Primitive::Uint8,
            idlc_ast::Primitive::Uint16 => Primitive::Uint16,
            idlc_ast::Primitive::Uint32 => Primitive::Uint32,
            idlc_ast::Primitive::Uint64 => Primitive::Uint64,
            idlc_ast::Primitive::Int8 => Primitive::Int8,
            idlc_ast::Primitive::Int16 => Primitive::Int16,
            idlc_ast::Primitive::Int32 => Primitive::Int32,
            idlc_ast::Primitive::Int64 => Primitive::Int64,
            idlc_ast::Primitive::Float32 => Primitive::Float32,
            idlc_ast::Primitive::Float64 => Primitive::Float64,
        }
    }
}

impl From<&idlc_ast::Const> for Const {
    fn from(const_: &idlc_ast::Const) -> Self {
        Const {
            ident: const_.ident.clone(),
            r#type: Primitive::from(&const_.r#type),
            value: const_.value.to_string(),
        }
    }
}

impl Type {
    fn new(ty: &idlc_ast::Type, idl_store: &IDLStore) -> Self {
        match ty {
            idlc_ast::Type::Primitive(primitive) => Type::Primitive(Primitive::from(primitive)),
            idlc_ast::Type::Interface => Type::Interface(None),
            idlc_ast::Type::Custom(custom) => {
                let ident = &custom.ident;
                match idl_store.iface_lookup(ident) {
                    Some(iface) => Type::Interface(Some(iface.ident.to_string())),
                    None => match idl_store.struct_lookup(ident) {
                        Some((r#struct, path)) => {
                            let mut fields = Vec::new();
                            for field in r#struct.fields.iter() {
                                let (ty, count) = &field.val;
                                fields.push(StructField {
                                    ident: field.ident.clone(),
                                    val: (Type::new(ty, idl_store), *count),
                                })
                            }
                            Type::Struct(Struct {
                                ident: r#struct.ident.clone(),
                                fields,
                                origin: Some(path),
                            })
                        }
                        None => panic!("Couldn't find any references of symbol {ident}"),
                    },
                }
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
    pub fn iter(&self) -> InterfaceIterator {
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
}
