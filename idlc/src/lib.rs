// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_ast_passes::{cycles, functions, idl_store::IDLStore, struct_verifier, CompilerPass};
use idlc_codegen::{Descriptor, Generator};
use idlc_errors::trace;
use idlc_mir::mir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
}

impl Language {
    pub fn generate(
        self,
        include_paths: &[std::path::PathBuf],
        input_file: &std::path::Path,
    ) -> Result<Descriptor, Box<dyn std::error::Error>> {
        let mut idl_store = IDLStore::with_includes(include_paths, false);
        let ast = idl_store.get_or_insert(input_file);

        trace!("Running `IncludeChecker` pass");
        idl_store.run_pass(&ast)?;

        trace!("Running `FunctionDuplicateParam` pass");
        functions::Functions::new().run_pass(&ast)?;

        trace!("Running `CycleChecking` pass");
        let struct_ordering = cycles::Cycles::new(&idl_store).run_pass(&ast)?;

        trace!("Running `StructVerifier` pass");
        struct_verifier::StructVerifier::run_pass(&idl_store, &struct_ordering)?;

        let mir = mir::parse_to_mir(&ast, &mut idl_store);

        match self {
            Self::Rust => Ok(idlc_codegen_rust::Generator::generate(&mir)),
        }
    }
}
