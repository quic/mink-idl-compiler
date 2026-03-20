// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

mod cli;

use clap::Parser;
use idlc::Language;

fn main() {
    let args = cli::Cli::parse();

    // Init vars for serialization
    idlc_codegen::serialization::init(args.bundle_params_by_size);

    let lang = match (args.c, args.cpp, args.java, args.rust) {
        (true, false, false, false) => Language::C,
        (true, true, false, false) => Language::CPP,
        (true, false, true, false) => Language::Java,
        (true, false, false, true) => Language::Rust,
        _ => unreachable!(),
    };

    // Change current dir based on the location of the input file.
    let input_file = args.file.canonicalize().expect("Invalid input file.");
    let dir_path = input_file
        .parent()
        .expect("Failed to find the location of the input file");

    let mut include_paths = args.include_paths.clone().unwrap_or_default();
    include_paths.push(dir_path.to_path_buf());

    let output = args
        .output
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let marking = match args.marking {
        Some(m_file) => std::fs::read_to_string(m_file).expect("Failed to read marking file"),
        _ => "".to_string(),
    };

    let compiler = idlc::Compiler::new(
        input_file,
        output,
        include_paths,
        lang,
        args.allow_undefined_behavior,
    );

    match args.dump {
        Some(cli::Dumpable::Pst) => compiler.dump_pst(),
        Some(cli::Dumpable::Ast) => compiler.dump_ast(),
        Some(cli::Dumpable::Mir) => compiler.dump_mir(),
        _ => compiler.generate(marking, args.skel, args.no_typed_objects),
    }
}
