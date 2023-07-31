use super::Error;
use petgraph::{
    algo::{is_cyclic_directed, toposort},
    stable_graph::{NodeIndex, StableDiGraph},
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::ast::{Identifiable, InterfaceNode, Node, Type};

type IncludeGraph<'a> = StableDiGraph<String, ()>;
type Symbol = String;
type Symbols = HashSet<Symbol>;
type Source = String;

#[derive(Debug)]
pub struct SymbolTable(HashMap<Source, Symbols>);
impl SymbolTable {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn add_source(&mut self, source: Source, ident: Symbol) {
        let map = &mut self.0;
        if let Some(set) = map.get_mut(&source) {
            set.insert(ident);
        } else {
            let mut set = HashSet::new();
            set.insert(ident);
            map.insert(source, set);
        }
    }
}

pub struct Includes<'a> {
    root: &'a Node,
    cache: RefCell<HashMap<Source, Rc<Node>>>,
}

impl<'a> Includes<'a> {
    pub fn new(ast: &'a Node) -> Self {
        Self {
            root: ast,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn symbol_table(&self) -> Result<SymbolTable, Error> {
        let (mut unresolved, _) = get_symbols(self.root)?;
        if unresolved.is_empty() {
            return Ok(SymbolTable::new());
        }

        let mut symbol_table = SymbolTable::new();
        let includes = self.toposort()?;

        for include in &includes[..includes.len() - 1] {
            if unresolved.is_empty() {
                break;
            }

            let ast = self.get_ast(include)?;
            let (i_unresolved, defined) = get_symbols(&ast)?;
            assert!(
                i_unresolved.is_empty(),
                "Expected {include} to not have any unresolved symbols but it had {unresolved:?}"
            );
            for symbol in defined {
                if unresolved.contains(&symbol) {
                    unresolved.remove(&symbol);
                    symbol_table.add_source(include.clone(), symbol);
                }
            }
        }
        if !unresolved.is_empty() {
            return Err(Error::UnresolvedSymbols(unresolved));
        }

        Ok(symbol_table)
    }

    fn get_ast(&self, key: &str) -> Result<Rc<Node>, Error> {
        {
            let mut cache = self.cache.borrow_mut();
            if !cache.contains_key(key) {
                let ast = Node::from_file(key)?;
                cache.insert(key.to_string(), Rc::new(ast));
            }
        }

        Ok(Rc::clone(self.cache.borrow().get(key).unwrap()))
    }

    fn toposort(&self) -> Result<Vec<String>, Error> {
        let mut graph = IncludeGraph::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        let mut stack = vec![Rc::new(self.root.clone())];
        while let Some(ast) = stack.pop() {
            let Node::CompilationUnit(root, nodes) = &*ast else { return Err(Error::AstDoesntContainRoot) };
            let deps = get_dependencies(nodes);
            let root_idx = get_idx(&mut graph, &mut node_map, root);
            for dep in deps {
                let dep_idx = get_idx(&mut graph, &mut node_map, dep);
                graph.add_edge(dep_idx, root_idx, ());
                stack.push(self.get_ast(dep)?);
            }
            if is_cyclic_directed(&graph) {
                return Err(Error::CyclicalInclude);
            }
        }

        // Safety: Cycles can never be formed if we reach here since we do
        // incremental cycle checking
        unsafe {
            let toposort = toposort(&graph, None).unwrap_unchecked();
            Ok(toposort
                .into_iter()
                .map(|nidx| graph.node_weight(nidx).unwrap_unchecked().clone())
                .collect())
        }
    }
}

fn get_dependencies(nodes: &[Node]) -> impl Iterator<Item = &str> {
    nodes.iter().filter_map(|node| {
        if let Node::Include(include) = node {
            Some(include.as_str())
        } else {
            None
        }
    })
}

fn get_idx<'a>(
    graph: &'a mut IncludeGraph,
    map: &'a mut HashMap<String, NodeIndex>,
    key: &'a str,
) -> NodeIndex {
    if let Some(&val) = map.get(key) {
        val
    } else {
        let key = key.to_string();
        let idx = graph.add_node(key.clone());
        map.insert(key, idx);
        idx
    }
}

fn get_symbols(ast: &Node) -> Result<(Symbols, Symbols), Error> {
    let mut unresolved = Symbols::new();
    let mut defined = Symbols::new();
    let Node::CompilationUnit(_, nodes) = ast else { return  Err(Error::AstDoesntContainRoot); };
    for node in nodes {
        match node {
            Node::Struct { ident, fields } => {
                for field in fields {
                    // See if fields are undefined
                    if let Type::Ident(ident) = &field.r#type().0 {
                        if !defined.contains(ident.as_str()) {
                            unresolved.insert(ident.clone());
                        }
                    }
                }
                // Define the struct
                defined.insert(ident.clone());
                unresolved.remove(ident.as_str());
            }
            Node::Interface(i) => {
                for node in i.nodes() {
                    if let InterfaceNode::Function {
                        doc: _,
                        ident: _,
                        params,
                    } = node
                    {
                        for param in params {
                            if let Type::Ident(ident) = param.r#type() {
                                if !defined.contains(ident.as_str()) {
                                    unresolved.insert(ident.clone());
                                }
                            }
                        }
                    }
                }
                defined.insert(i.ident().clone());
                unresolved.remove(i.ident());
            }
            _ => {}
        }
    }
    Ok((unresolved, defined))
}
