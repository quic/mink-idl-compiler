// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

//! IDL Compiler (idlc) for the Mink architecture.
//!
//! # Architecture
//! 1. Input IDL is first converted into a Parse Syntax Tree (PST) using [pest](https://github.com/pest-parser/pest/)
//! 2. PST is converted into an AST in [`idlc_ast`].
//! 3. AST is extended by adding Mink specific functionality in [`idlc_mir`].
//!    Cross interface resolution and hierarchy is also resolved here.
//! 4. MIR is consumed by [`idlc_codegen`] and it's derivatives to create the output file.

use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;

mod errors;
mod timer;
use errors::check;

use idlc_ast::Ast;
use idlc_ast_passes::{cycles, idl_store::IDLStore, struct_verifier, CompilerPass};
use idlc_codegen::{Generator, SplitInvokeGenerator};
use idlc_mir::{Mir, NamedVersion};
use idlc_mir_passes::{interface_verifier, MirCompilerPass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    C,
    CPP,
    Java,
    Rust,
}

pub struct Compiler {
    input: PathBuf,
    output: PathBuf,
    includes: Vec<PathBuf>,
    lang: Language,
    allow_undefined_behavior: bool,
    raw_idl: String,
}

impl Compiler {
    pub fn new(
        file: PathBuf,
        out: PathBuf,
        includes: Vec<PathBuf>,
        lang: Language,
        allow_undefined_behavior: bool,
    ) -> Self {
        idlc_errors::init();
        match lang {
            Language::C | Language::CPP => {
                if out.is_dir() {
                    idlc_errors::unrecoverable!("Codegen language expects output file.")
                }
            }
            Language::Java | Language::Rust => {
                if out.is_file() {
                    idlc_errors::unrecoverable!("Codegen language expects output directory.")
                }
            }
        }
        let content = std::fs::read_to_string(&file).unwrap();
        Self {
            input: file,
            output: out,
            includes,
            lang,
            allow_undefined_behavior,
            raw_idl: content,
        }
    }

    fn parse_to_ast(&self) -> (Rc<Ast>, IDLStore) {
        let mut idl_store = IDLStore::with_includes(&self.includes, self.allow_undefined_behavior);
        let ast = idl_store.get_or_insert(&self.input);

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

        (ast, idl_store)
    }

    fn parse_to_mir(&self) -> Mir {
        let (ast, mut idl_store) = self.parse_to_ast();
        let mir = timer::time!(idlc_mir::parse_to_mir(&ast, &mut idl_store), "Mir");

        idlc_errors::trace!("Verifying interfaces");
        interface_verifier::InterfaceVerifier::new(&mir).run_pass();
        mir
    }

    pub fn generate(
        &self,
        legal_marking: String,
        skeleton: bool,
        no_typed_objects: bool,
        specs: Vec<NamedVersion>,
    ) {
        let mut mir = self.parse_to_mir();

        // Prune the MIR to specs passed through the CLI, if any. Because the same
        // mir tree is parsed in multiple places after this, it is easier to modify
        // the tree itself rather than instruct all code generators to ignore the
        // same methods.
        mir.prune(specs);

        match self.lang {
            Language::C => {
                let c_gen = idlc_codegen_c::Generator::new(no_typed_objects);
                let content = if skeleton {
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
                    .open(&self.output)
                    .unwrap();
                let marking = idlc_codegen::marking::Marking::new(
                    &legal_marking,
                    idlc_codegen::marking::MarkingStyle::C,
                );
                file.write_all(marking.as_bytes()).unwrap();
                file.write_all(content.as_bytes()).unwrap();
            }
            Language::CPP => {
                let content = if skeleton {
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
                    .open(&self.output)
                    .unwrap();
                let marking = idlc_codegen::marking::Marking::new(
                    &legal_marking,
                    idlc_codegen::marking::MarkingStyle::C,
                );
                file.write_all(marking.as_bytes()).unwrap();
                file.write_all(content.as_bytes()).unwrap();
            }
            Language::Java => {
                idlc_errors::warn!(
                        "Note: JavaGen is untested but guaranteed to generate same output as the previous versions.",
                );
                let marking = idlc_codegen::marking::Marking::new(
                    &legal_marking,
                    idlc_codegen::marking::MarkingStyle::Java,
                );
                for (name, content) in
                    timer::time!(idlc_codegen_java::Generator::generate(&mir), "Java codegen")
                {
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(self.output.join(name))
                        .unwrap();
                    file.write_all(marking.as_bytes()).unwrap();
                    file.write_all(content.as_bytes()).unwrap();
                }
            }
            Language::Rust => {
                let marking = idlc_codegen::marking::Marking::new(
                    &legal_marking,
                    idlc_codegen::marking::MarkingStyle::Rust,
                );
                for (name, content) in
                    timer::time!(idlc_codegen_rust::Generator::generate(&mir), "Rust codegen")
                {
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(self.output.join(name))
                        .unwrap();
                    file.write_all(marking.as_bytes()).unwrap();
                    file.write_all(content.as_bytes()).unwrap();
                }
            }
        };
    }

    pub fn dump_pst(&self) {
        use std::time::Instant;
        let now = Instant::now();
        let pst = idlc_ast::pst::parse_to_pst(self.raw_idl.as_ref());
        let duration = now.elapsed();
        match pst {
            Ok(pst) => println!("{pst:#?}"),
            Err(e) => eprintln!("Parsing failed:\n{e}\n"),
        }
        eprintln!("'dump_pst' completed in {duration:?}");
    }

    pub fn dump_ast(&self) {
        use std::time::Instant;
        let now = Instant::now();
        let ast = self.parse_to_ast();
        let duration = now.elapsed();
        println!("{ast:#?}");
        eprintln!("'dump_ast' completed in {duration:?}");
    }

    pub fn dump_mir(&self) {
        let mir = self.parse_to_mir();
        println!("{mir:#?}");
    }
}
