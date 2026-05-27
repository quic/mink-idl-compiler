// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_mir::NamedVersion;

fn long_version() -> &'static str {
    let git_hash = env!("GIT_HASH");

    if git_hash == "unknown" {
        env!("CARGO_PKG_VERSION")
    } else {
        format!(
            "{}\nCompiled from (git hash): {}",
            env!("CARGO_PKG_VERSION"),
            git_hash
        )
        .leak()
    }
}

#[derive(clap::Parser)]
#[command(author, version, long_version=long_version(), about = None, long_about)]
/// Compile Mink IDL files into a header to be used by one of the supported language
pub struct Cli {
    /// Input IDL file
    pub idl_file: std::path::PathBuf,

    #[arg(short, value_name = "FILE or PATH")]
    /// Output file (C/C++) or path (Rust/Java)
    pub output: Option<std::path::PathBuf>,

    #[arg(long, conflicts_with_all = ["java", "rust"])]
    /// Generate skeleton header (instead of stub header).
    pub skel: bool,

    #[arg(long, group = "lang", default_value_t = true)]
    /// Generate c header. This is the default language.
    pub c: bool,

    #[arg(long, group = "lang")]
    /// Generate c++ header
    pub cpp: bool,

    #[arg(long, group = "lang")]
    /// Generate Java
    /// Note: Untested but guaranteed to generate same output as the previous versions.
    pub java: bool,

    #[arg(long, group = "lang")]
    /// Generate Rust
    pub rust: bool,

    #[arg(short = 'I', long = "include", value_name = "DIR")]
    /// Add DIR to include path. Can be passed multiple times.
    pub include_paths: Option<Vec<std::path::PathBuf>>,

    #[arg(long)]
    /// Dump various phases of the compiler and exit.
    pub dump: Option<Dumpable>,

    #[arg(long)]
    /// Adding marking on top of the generated file
    pub marking: Option<std::path::PathBuf>,

    #[arg(long = "no-typed-objects", default_value_t = false)]
    /// Forces C codegen to emit 'Object' as a object type instead of its own type.
    /// This option does NOT affect any other codegen backends.
    pub no_typed_objects: bool,

    #[arg(long, default_value_t = false)]
    /// `idlc` by default is pedantic about integer widths overflowing.
    ///
    /// To allow undefined behavior to go through codegen enable this flag, outputs aren't
    /// guaranteed!
    pub allow_undefined_behavior: bool,

    #[arg(long, default_value_t = false)]
    /// Sort bundled parameters by size, rather than by alignment.
    ///
    /// This is for compatibility with headers that were generated with a buggy
    /// compiler.
    pub bundle_params_by_size: bool,

    #[arg(long, value_name = "NAME{@|:|=}X.Y", value_parser = clap::value_parser!(NamedVersion))]
    /// Repeatable: NAME@X.Y | NAME:X.Y | NAME=X.Y
    ///
    /// Restrict the list of methods to those found up to - and including - the
    /// specified interface version. Specifying a version which exceeds the
    /// interface definition is effectively ignored.
    ///
    /// If a spec does not match any interface in the input file, an error is
    /// thrown. Interface names are case-sensitive.
    pub spec: Vec<NamedVersion>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Dumpable {
    /// Parse Syntax Tree
    Pst,
    /// Abstract Syntax Tree
    Ast,
    /// Mid-level Intermediate Representation
    Mir,
}
