// Binary targets might not use all the functions.
#![allow(unused)]

use std::collections::HashMap;

use idlc_ast::{visitor::Visitor, Node, Struct};

use idlc_ast_passes::{includes, struct_verifier, ASTStore, CompilerPass, Error};

#[derive(clap::Parser)]
#[command(author, version, about = None, long_about)]
/// Parse .idl files into AST.
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

fn main() {
    use clap::Parser as ClapParser;
    let args = Args::parse();
    if let Some(dump) = args.dump {
        match dump {
            Dumpable::Pst => idlc_ast::dump_pst(&args.path),
            Dumpable::Ast => idlc_ast::dump(&args.path),
        }
        std::process::exit(0);
    }

    let ast_store = ASTStore::new();
    let ast = ast_store.get_or_insert(&args.path).unwrap();

    println!("Checking for unresolved includes...");
    let ordering = check(includes::Includes::new(&ast_store).run_pass(&ast));

    println!("Checking for struct sizes");
    check(struct_verifier::StructVerifier::new(&ast_store).run_pass(&ast));
}
