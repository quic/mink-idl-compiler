use std::collections::{HashMap, HashSet};

use crate::ast::{Identifiable, InterfaceNode, Node, Param};

use super::Error;
type IdentSet<'a> = HashSet<&'a str>;
type IdentMap<'a> = HashMap<&'a str, IdentSet<'a>>;

pub fn contains_duplicate_symbols(ast: &Node) -> Result<(), Error> {
    let mut map = IdentMap::new();
    let Node::CompilationUnit(root, nodes) = ast else { return Err(Error::AstDoesntContainRoot); };

    for node in nodes {
        let map_err = |expr: Result<(), String>| {
            expr.map_err(|ident| Error::DuplicateDefinition {
                root: format!("{} {}", node.r#type(), node.ident().unwrap()),
                ident,
            })
        };

        match node {
            Node::Const(c) => map_err(insert_if_not_duplicate(&mut map, root, c.ident()))?,
            Node::Struct(s) => {
                map_err(insert_if_not_duplicate(&mut map, root, s.ident()))?;
                let mut field_set = IdentSet::new();
                for field in s.fields() {
                    map_err(insert_set(&mut field_set, field.ident()))?;
                }
            }
            Node::Interface(i) => map_err(validate_iface(&mut map, i.ident(), i.nodes()))?,
            _ => {}
        }
    }
    Ok(())
}

fn insert_if_not_duplicate<'a>(
    map: &mut IdentMap<'a>,
    iface: &'a str,
    ident: &'a str,
) -> Result<(), String> {
    let set = map.get_mut(iface);
    if let Some(set) = set {
        insert_set(set, ident)
    } else {
        let mut set = HashSet::new();
        set.insert(ident);
        map.insert(iface, set);
        Ok(())
    }
}

fn insert_set<'a>(set: &mut IdentSet<'a>, ident: &'a str) -> Result<(), String> {
    if set.contains(ident) {
        Err(ident.to_string())
    } else {
        set.insert(ident);
        Ok(())
    }
}

fn validate_iface<'a>(
    map: &mut IdentMap<'a>,
    iface: &'a str,
    nodes: &'a [InterfaceNode],
) -> Result<(), String> {
    if map.contains_key(iface) {
        return Err(iface.to_string());
    }
    for node in nodes {
        match node {
            InterfaceNode::Const(c) => insert_if_not_duplicate(map, iface, c.ident())?,
            InterfaceNode::Error(e) => insert_if_not_duplicate(map, iface, e)?,
            InterfaceNode::Function {
                doc: _,
                ident,
                params,
            } => {
                insert_if_not_duplicate(map, iface, ident)?;
                let mut param_set = HashSet::new();
                for param in params {
                    match param {
                        Param::In { r#type: _, ident } | Param::Out { r#type: _, ident } => {
                            insert_set(&mut param_set, ident)?;
                        }
                    }
                }
            }
        }
    }
    map.insert(iface, HashSet::new());
    Ok(())
}
