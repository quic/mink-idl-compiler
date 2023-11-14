// Binary targets might not use all the functions.
#![allow(unused)]

use std::collections::HashMap;

use idlc_ast::{dump, visitor::Visitor};

use idlc_ast_passes::{cycles, idl_store::IDLStore, struct_verifier, CompilerPass, Error};

use idlc_mir::mir;

use idlc_mir_passes::{interface_verifier, MirCompilerPass};

use std::time::Instant;

#[derive(clap::Parser)]
#[command(author, version, about = None, long_about)]
/// Parse .idl files into AST.
///
/// The C-compiler for example returns `nested too deeply`, rust detects cyclic
/// imports nicer.
struct Cli {
    /// Input IDL file
    file: std::path::PathBuf,

    #[arg(short, value_name = "FILE")]
    /// Output file name
    output: Option<std::path::PathBuf>,

    #[arg(long, conflicts_with_all = ["java", "rust"])]
    /// Generate skeleton header (instead of stub header).
    skel: bool,

    #[arg(long, group = "lang", default_value_t = true)]
    /// Generate c header. This is the default language.
    c: bool,

    #[arg(long, group = "lang")]
    /// Generate c++ header
    cpp: bool,

    #[arg(long, group = "lang")]
    /// Generate Java
    java: bool,

    #[arg(long, group = "lang")]
    /// Generate Rust
    rust: bool,

    #[arg(short = 'I', long = "include", value_name = "DIR")]
    /// Add DIR to include path. Can be passed multiple times.
    include_paths: Option<Vec<std::path::PathBuf>>,

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
    /// Mid-level Intermediate Representation
    Mir,
}

fn main() {
    use clap::Parser as ClapParser;
    let args = Cli::parse();

    let dump = args.dump;
    if dump == Some(Dumpable::Pst) {
        idlc_ast::pst::dump(&args.file);
        std::process::exit(0);
    } else if dump == Some(Dumpable::Ast) {
        idlc_ast::dump(&args.file);
        std::process::exit(0);
    }

    // Change current dir based on the location of the input file.
    let input_file = &args.file.canonicalize().expect("Invalid input file.");
    let dir_path = input_file
        .parent()
        .expect("Failed to find the location of the input file");

    let mut include_paths = args.include_paths.clone().unwrap_or_default();
    include_paths.push(dir_path.to_path_buf());

    let mut idl = IDLStore::with_includes(&include_paths);

    let ast = idl.get_or_insert(input_file).unwrap();

    println!("Resolving includes...");
    _ = check(idl.run_pass(&ast));

    println!("Checking for struct cycles");
    let struct_ordering = check(cycles::Cycles::new(&idl).run_pass(&ast));

    println!("Checking for struct sizes");
    check(struct_verifier::StructVerifier::run_pass(
        &idl,
        &struct_ordering,
    ));

    let now = Instant::now();
    let mir = mir::parse_to_mir(&ast, &mut idl);
    let duration = now.elapsed();

    if dump == Some(Dumpable::Mir) {
        idlc_mir::dump(mir);
        eprintln!("`dump_mir` completed in {duration:?}");
        std::process::exit(0);
    }

    println!("Verifying interfaces");
    check(interface_verifier::InterfaceVerifier::new(&mir).run_pass());
}
