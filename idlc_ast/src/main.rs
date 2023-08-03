// Binary targets might not use all the functions.
#![allow(unused)]

use ast::visitor::Visitor;

use crate::passes::{duplicate, includes, ASTStore, CompilerPass, Error};

mod ast;
mod passes;

#[derive(clap::Parser)]
#[command(author, version, about = None, long_about)]
/// Parse .idl files into AST.
///
/// TODO(arvimuku): Currently the syntatic analysis phase is done although
/// there's no semantic analysis done for external types. I'm wondering how i'd
/// address the dependencies nicely especially detect cyclic dependencies and
/// such would need to include some sort of cycle detection + topological
/// sorting + creating a giant paste table.
///
/// The C-compiler for example returns `nested too deeply`, rust detects cyclic
/// imports nicer.
struct Args {
    /// Path of IDL to dump AST for
    path: String,

    #[arg(long, value_enum)]
    /// Dump various phases of the compiler and exit.
    dump: Option<Dumpable>,
}

fn check<T: std::fmt::Debug, E: std::fmt::Display>(r: Result<T, E>) -> T {
    match r {
        Ok(t) => {
            dbg!(&t);
            t
        }
        Err(e) => {
            eprintln!("{e}");
            panic!();
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum Dumpable {
    /// Parse Syntax Tree
    Pst,
    /// Abstract Syntax Tree
    Ast,
}

// #[derive(Debug, Clone)]
// struct RustCodegen;
// impl RustCodegen {
//     fn into_rust_type(r#type: &ast::Type) -> &str {
//         match r#type {
//             ast::Type::Primitive(primitive) => Self::into_rust_primitive(primitive),
//             ast::Type::Custom(ident) => ident.as_str(),
//         }
//     }

//     fn into_rust_primitive(primitive: &ast::Primitive) -> &'static str {
//         match primitive {
//             ast::Primitive::Uint8 => "u8",
//             ast::Primitive::Uint16 => "u16",
//             ast::Primitive::Uint32 => "u32",
//             ast::Primitive::Uint64 => "u64",
//             ast::Primitive::Int8 => "i8",
//             ast::Primitive::Int16 => "i16",
//             ast::Primitive::Int32 => "i32",
//             ast::Primitive::Int64 => "i64",
//             ast::Primitive::Float32 => "f32",
//             ast::Primitive::Float64 => "f64",
//         }
//     }
// }

// impl Visitor for RustCodegen {
//     fn visit_struct_field(&mut self, field: &ast::StructField) -> String {
//         let (r#type, count) = field.r#type();
//         let rust_type = RustCodegen::into_rust_type(r#type);

//         if count.get() > 1 {
//             format!("{}: [{rust_type}; {count}]", field.ident)
//         } else {
//             format!("{}: {rust_type}", field.ident)
//         }
//     }

//     fn visit_struct_prefix(&mut self, ident: &str) -> String {
//         format!(
//             r#"#[derive(Debug, Clone, Copy, PartialEq)]
// struct {ident} {{
// "#
//         )
//     }

//     fn visit_struct_suffix(&mut self, ident: &str) -> String {
//         "}".to_string()
//     }

//     fn struct_field_seperator(&self) -> &'static str {
//         ",\n"
//     }

//     fn visit_include(&mut self, include: &str) -> String {
//         let prefix = include.len() - 4;
//         format!(
//             "use crate::interfaces::{};",
//             include[..prefix].to_lowercase()
//         )
//     }

//     fn visit_caller(&mut self, interface: &ast::Interface) -> String {
//         let error = format!(
//             r#"#[repr(transparent)]
// #[derive(Clone, Copy, PartialEq, Eq)]
// pub struct Error(crate::object::Error);

// impl From<Error> for crate::object::Error {{
//     fn from(e: Error) -> Self {{
//         e.0
//     }}
// }}

// impl From<crate::object::Error> for Error {{
//     fn from(e: crate::object::Error) -> Self {{
//         Self(e)
//     }}
// }}
// "#
//         );
//         let mut consts = String::new();
//         let mut functions = String::new();
//         for node in &interface.nodes {
//             match node {
//                 ast::InterfaceNode::Const(c) => consts.push_str(&format!(
//                     "pub const {}: {} = {};",
//                     c.ident,
//                     RustCodegen::into_rust_primitive(c.r#type()),
//                     c.value
//                 )),
//                 ast::InterfaceNode::Function { doc, ident, params } => {}
//                 ast::InterfaceNode::Error(e) => {}
//             }
//         }

//         (error + &consts)
//     }

//     fn visit_callee(&mut self, interface: &ast::Interface) -> String {
//         todo!()
//     }
// }

// fn visit_all(codegen: &mut impl Visitor, ast: ast::Node) {
//     let ast::Node::CompilationUnit(name, nodes) = ast else { unreachable!() };
//     for node in &nodes {
//         //dbg!(&node);
//         match &node {
//             ast::Node::Include(r#include) => println!("{}\n", codegen.visit_include(r#include)),
//             ast::Node::Const(_) => {}
//             ast::Node::Struct(s) => {
//                 println!("{}\n", codegen.visit_struct(s));
//             }
//             ast::Node::Interface(interface) => println!("{}\n", codegen.visit_caller(interface)),
//             _ => unreachable!(),
//         }
//     }
// }

fn main() {
    use clap::Parser as ClapParser;
    let args = Args::parse();
    if let Some(dump) = args.dump {
        match dump {
            Dumpable::Pst => ast::dump_pst(&args.path),
            Dumpable::Ast => ast::dump(&args.path),
        }
        std::process::exit(0);
    }

    let ast_store = ASTStore::new();
    let ast = ast_store.get_or_insert(&args.path).unwrap();

    println!("Checking for duplicate symbols...");
    check(duplicate::DuplicateDetector::new().run_pass(&ast));

    println!("Checking for unresolved includes...");
    check(includes::Includes::new(&ast_store).run_pass(&ast));
}
