use super::{
    Const, Count, Documentation, Function, Ident, Interface, InterfaceNode, Node, Param, Primitive,
    Struct, StructField, Type,
};

#[allow(unused_variables)]
pub trait Visitor<'ast>: Sized {
    fn visit_include(&mut self, include: &'ast std::path::Path) {}

    fn visit_const(&mut self, constant: &'ast Const) {
        walk_const(self, constant);
    }

    fn visit_ident(&mut self, ident: &'ast Ident) {}

    fn visit_expr(&mut self, expr: &'ast str) {}

    fn visit_struct(&mut self, r#struct: &'ast Struct) {
        walk_struct(self, r#struct);
    }

    fn visit_struct_field(&mut self, field: &'ast StructField) {
        walk_struct_field(self, field);
    }

    fn visit_ty(&mut self, ty: (&'ast Type, Count)) {
        walk_ty(self, ty);
    }

    fn visit_primitive_ty(&mut self, ty: (&'ast Primitive, Count)) {}
    fn visit_custom_ty(&mut self, ty: (&'ast str, Count)) {}
    fn visit_interface(&mut self, iface: &'ast Interface) {
        walk_iface(self, iface);
    }
    fn visit_iface_node(&mut self, node: &'ast InterfaceNode) {
        walk_iface_node(self, node);
    }
    fn visit_fn(&mut self, function: &'ast Function) {
        walk_fn(self, function);
    }
    fn visit_error(&mut self, error: &'ast Ident) {}
    fn visit_doc(&mut self, doc: &'ast Documentation) {}
    fn visit_fn_param(&mut self, param: &'ast Param) {}
    fn visit_root_ident(&mut self, root_ident: &'ast std::path::Path) {}
}

pub fn walk_const<'a, V: Visitor<'a>>(visitor: &mut V, constant: &'a Const) {
    visitor.visit_ident(&constant.ident);
    visitor.visit_primitive_ty((&constant.r#type, Count::new(1).unwrap()));
    visitor.visit_expr(&constant.value)
}

pub fn walk_struct<'a, V: Visitor<'a>>(visitor: &mut V, r#struct: &'a Struct) {
    visitor.visit_ident(&r#struct.ident);
    for field in &r#struct.fields {
        visitor.visit_struct_field(field);
    }
}

pub fn walk_struct_field<'a, V: Visitor<'a>>(visitor: &mut V, field: &'a StructField) {
    visitor.visit_ident(&field.ident);
    let (ty, ele) = &field.val;
    visitor.visit_ty((ty, *ele));
}

pub fn walk_ty<'a, V: Visitor<'a>>(visitor: &mut V, ty: (&'a Type, Count)) {
    match ty.0 {
        Type::Primitive(p) => visitor.visit_primitive_ty((p, ty.1)),
        Type::Custom(c) => visitor.visit_custom_ty((c, ty.1)),
    }
}

pub fn walk_iface<'a, V: Visitor<'a>>(visitor: &mut V, iface: &'a Interface) {
    visitor.visit_ident(&iface.ident);
    if let Some(base) = &iface.base {
        visitor.visit_ident(base);
    }
    for node in &iface.nodes {
        visitor.visit_iface_node(node);
    }
}

pub fn walk_iface_node<'a, V: Visitor<'a>>(visitor: &mut V, node: &'a InterfaceNode) {
    match node {
        InterfaceNode::Const(c) => visitor.visit_const(c),
        InterfaceNode::Function(f) => visitor.visit_fn(f),
        InterfaceNode::Error(e) => visitor.visit_error(e),
    }
}

pub fn walk_fn<'a, V: Visitor<'a>>(visitor: &mut V, function: &'a Function) {
    visitor.visit_ident(&function.ident);
    if let Some(doc) = &function.doc {
        visitor.visit_doc(doc);
    }
    for param in &function.params {
        visitor.visit_fn_param(param);
    }
}

pub fn walk_all<'a, V: Visitor<'a>>(visitor: &mut V, root: &'a Node) {
    let Node::CompilationUnit(root_ident, nodes) = root else {
        unreachable!("ICE: walk_all was called without root being the starting node.")
    };
    visitor.visit_root_ident(root_ident);
    for node in nodes {
        match node.as_ref() {
            Node::Include(i) => visitor.visit_include(i),
            Node::Const(c) => visitor.visit_const(c),
            Node::Struct(s) => visitor.visit_struct(s),
            Node::Interface(i) => visitor.visit_interface(i),
            _ => unreachable!("ICE: node had a variant that wasn't expected."),
        }
    }
}
