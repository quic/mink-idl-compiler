// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

//! IDL Compiler (idlc) for the Mink architecture.
//!
//! # Architecture
//! 1. Input IDL is first converted into a Parse Syntax Tree (PST) using [pest](https://github.com/pest-parser/pest/)
//! 2. PST is converted into an AST in [`idlc_ast`].
//! 3. AST is extended by adding Mink specific functionality in [`idlc_mir`].
//!    Cross interface resolution and hierarchy is also resolved here.
//! 4. MIR is consumed by [`idlc_codegen`] and it's derivatives to create the output file.
mod errors;
mod timer;
use errors::check;

use std::io::Write;

use idlc_errors::trace;

use idlc_ast_passes::{cycles, idl_store::IDLStore, struct_verifier, CompilerPass};

use idlc_mir::mir;
use idlc_mir_passes::{interface_verifier, MirCompilerPass};

use idlc_codegen::{Generator, SplitInvokeGenerator};

fn long_version() -> &'static str {
    format!(
        "{}\nCompiled from (git hash): {}",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH")
    )
    .leak()
}

#[derive(clap::Parser)]
#[command(author, version, long_version=long_version(), about = None, long_about)]
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
    /// Note: Untested but guaranteed to generate same output as the previous versions.
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

    #[arg(long)]
    /// Adding marking on top of the generated file
    marking: Option<std::path::PathBuf>,

    #[arg(long = "no-typed-objects", default_value_t = false)]
    /// Forces C codegen to emit 'Object' as a object type instead of its own type.
    /// This option does NOT affect any other codegen backends.
    no_typed_objects: bool,

    #[arg(long, default_value_t = false)]
    /// `idlc` by default is pedantic about integer widths overflowing.
    ///
    /// To allow undefined behavior to go through codegen enable this flag, outputs aren't
    /// guaranteed!
    allow_undefined_behavior: bool,
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
    idlc_errors::init();
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

    let mut idl_store = IDLStore::with_includes(&include_paths, args.allow_undefined_behavior);

    let ast = idl_store.get_or_insert(input_file);

    timer::time!(check(idl_store.run_pass(&ast)), "`IncludeChecker` pass");

    timer::time!(
        check(idlc_ast_passes::functions::Functions::new().run_pass(&ast)),
        "`FunctionDuplicateParam` pass"
    );

    let struct_ordering = timer::time!(
        check(cycles::Cycles::new(&idl_store).run_pass(&ast)),
        "`CycleCheck` pass"
    );

    timer::time!(
        check(struct_verifier::StructVerifier::run_pass(
            &idl_store,
            &struct_ordering,
        )),
        "`StructVerifier` pass"
    );

    let mir = timer::time!(mir::parse_to_mir(&ast, &mut idl_store), "Mir");
    if dump == Some(Dumpable::Mir) {
        idlc_mir::dump(mir);
        std::process::exit(0);
    }

    trace!("Verifying interfaces");
    interface_verifier::InterfaceVerifier::new(&mir).run_pass();

    let marking = idlc_codegen::documentation::Documentation::add_marking(
        args.marking,
        idlc_codegen::documentation::DocumentationStyle::C,
    );

    let output = args
        .output
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    match (args.c, args.cpp, args.java, args.rust) {
        (true, false, false, false) => {
            let c_gen = idlc_codegen_c::Generator::new(args.no_typed_objects);
            let content = if args.skel {
                timer::time!(c_gen.generate_invoke(&mir), "C invoke codegen")
            } else {
                timer::time!(
                    c_gen.generate_implementation(&mir),
                    "C implementation codegen"
                )
            };
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(output)
                .unwrap();
            file.write_all(marking.as_bytes()).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }
        (true, true, false, false) => {
            let content = if args.skel {
                timer::time!(
                    idlc_codegen_cpp::Generator.generate_invoke(&mir),
                    "C++ invoke codegen"
                )
            } else {
                timer::time!(
                    idlc_codegen_cpp::Generator.generate_implementation(&mir),
                    "C++ implementation codegen"
                )
            };
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(output)
                .unwrap();
            file.write_all(marking.as_bytes()).unwrap();
            file.write_all(content.as_bytes()).unwrap();
        }
        (true, false, true, false) => {
            idlc_errors::warn!(
                "Note: JavaGen is untested but guaranteed to generate same output as the previous versions.",
            );
            for (name, content) in
                timer::time!(idlc_codegen_java::Generator::generate(&mir), "Java codegen")
            {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output.join(name))
                    .unwrap();
                file.write_all(marking.as_bytes()).unwrap();
                file.write_all(content.as_bytes()).unwrap();
            }
        }
        (true, false, false, true) => {
            for (name, content) in
                timer::time!(idlc_codegen_rust::Generator::generate(&mir),"Rust codegen")
            {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output.join(name))
                    .unwrap();
                file.write_all(marking.as_bytes()).unwrap();
                file.write_all(content.as_bytes()).unwrap();
            }
        }
        _ => unreachable!(),
    };
}
