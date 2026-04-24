# Mink IDL Compiler Architecture

This document explains how the workspace crates collaborate to compile `.idl` files into target-language output.

## High-Level Overview

The compiler follows a staged pipeline:

1. Parse source text with a parsing expression grammar (PEG) into a Parser Syntx Tree (PST) using [pest](https://pest.rs/).
2. Convert PST into a typed Abstract Syntax Tree (AST).
3. Run AST-level validation and include/symbol resolution passes.
4. Lower AST into Mid-level Intermediate Representation (MIR).
5. Run MIR-level semantic checks.
6. Generate code through language backends (C, C++, Java, Rust).

The orchestration entrypoint is the `idlc` crate.

## Crate Responsibilities

- `idlc`: CLI binary and pipeline orchestration.
  - Parses CLI arguments (`clap`).
  - Chooses target language and output mode.
  - Initializes shared serialization behavior.
  - Runs `AST passes -> MIR passes -> codegen`.
- `idlc_ast`: Parsing and AST model.
  - Contains the PEST grammar in `idlc_ast/src/idl_grammar.pest`.
  - Parses IDL into PST/AST (`pst`, `ast` modules).
  - Exposes AST types and visitors.
- `idlc_ast_passes`: AST validation and dependency processing.
  - Include graph traversal and cycle detection (`IDLStore`, `cycles`).
  - Symbol collection/lookup across include files.
  - Function duplicate-parameter checks.
  - Struct verification (layout/constraints).
- `idlc_mir`: MIR model and AST->MIR lowering.
  - Creates codegen-stable IR decoupled from AST evolution.
  - Resolves Mink-specific details (e.g., function op codes, error code numbering).
- `idlc_mir_passes`: MIR semantic validation.
  - Interface-level collision checks.
  - Object/array argument rules and related Mink restrictions.
- `idlc_codegen`: Shared codegen abstractions/utilities.
  - Common traits (`Generator`, `SplitInvokeGenerator`).
  - Marking/header helpers, function helpers, serialization packing logic.
- `idlc_codegen_c`: C backend.
  - Emits constants, structs, includes, interface stubs/skeletons.
- `idlc_codegen_cpp`: C++ backend.
  - Emits C++ forms of constants/structs/interfaces.
  - Reuses portions of C backend utilities for shared constructs.
- `idlc_codegen_java`: Java backend.
  - Emits one or more `.java` files from MIR.
  - The Java output is tailored for Android development. It depends on files not included in this repository, including `MinkProxy.java` and `JMinkObject.java`.
- `idlc_codegen_rust`: Rust backend.
  - Emits one or more `.rs` files from MIR.
- `idlc_errors`: Logging/error helpers used across crates.

## Stage-by-Stage Data Flow

### 1) IDL Text -> PST (PEST)

`idlc_ast::pst` uses `#[derive(Parser)]` over `idl_grammar.pest`.

Grammar defines core language forms:
- includes
- const declarations
- struct declarations
- interfaces, methods, params, attributes, and errors

`idlc` can dump this stage with `--dump pst`.

### 2) PST -> AST

`idlc_ast` converts parser output into AST nodes (`Ast`, `Node`, `Interface`, `Struct`, etc.).

At this stage the compiler has a language-structured model, but not full cross-file semantic resolution.

`idlc` can dump this stage with `--dump ast`.

### 3) AST Passes and Include/Symbol Resolution

`idlc::Compiler::parse_to_ast` constructs `IDLStore` with include paths and runs passes:

1. `IDLStore` include checker (`run_pass`):
   - walks include graph,
   - canonicalizes include paths,
   - detects include cycles,
   - stores ASTs and symbols for lookup.
2. `Functions` pass:
   - verifies no duplicate parameter names in each interface method.
3. `Cycles` pass:
   - computes ordering used for struct validation.
4. `StructVerifier` pass:
   - validates struct rules and constraints.

### 4) AST -> MIR

`idlc_mir::parse_to_mir` lowers AST into MIR.

Why MIR exists:
- isolates backends from AST churn,
- adds Mink semantics required by codegen.

MIR-specific additions include:
- function opcode assignment and bounds checks (`0 ..= 0x3fff`),
- error code mapping starting at `10`,
- normalized parameter and type forms used by all backends.

`idlc` can dump this stage with `--dump mir`.

### 5) MIR Passes

`idlc_mir_passes::interface_verifier` enforces backend-critical restrictions, including:
- duplicate function/const/error names across interface inheritance,
- invalid combinations of object arrays and non-array object params,
- invalid bounded arrays for primitive/struct params,
- disallowed struct-array cases when nested object/interface fields are present.

### 6) MIR -> Language Codegen

`idlc::Compiler::generate` dispatches by language:

- `C` and `C++` use `SplitInvokeGenerator`:
  - implementation/stub output by default,
  - skeleton/invoke output with `--skel`.
- `Java` and `Rust` use `Generator` and can emit multiple files - one per `interface` definiton.

All generated files can prepend optional legal marking text (`--marking`) with style-specific formatting.

## How Backends Work Together

- All backends depend on `idlc_mir` for a shared semantic contract.
- `idlc_codegen` provides shared traits and helpers so backend crates stay focused on syntax emission.
- C++ backend partially reuses C backend support modules where representations align.
- Serialization packing behavior is globally configured once from CLI (`idlc_codegen::serialization::init(...)`), then consumed by backend emitters.

## CLI Options Reference

Command shape:

```sh
idlc [OPTIONS] <FILE>
```

### Positional

- `<FILE>`: input `.idl` file.

### Output and language selection

- `-o <FILE>`:
  - For C/C++, this must be an output file path.
  - For Java/Rust, this must be an output directory.
  - If omitted, defaults to current working directory (so practical use normally sets `-o`).
- `--c`:
  - Generate C output.
  - This is the default language behavior.
- `--cpp`: generate C++ output.
- `--java`: generate Java output.
- `--rust`: generate Rust output.
- `--skel`:
  - For C/C++, emit skeleton/invoke-side output instead of implementation/stub side.
  - Conflicts with Java/Rust modes.

### Include and stage inspection

- `-I, --include <DIR>`:
  - Add include directory (repeatable).
  - Compiler also automatically adds the input file's parent directory.
- `--dump <pst|ast|mir>`:
  - Print selected compiler stage and exit (no code generation).

### Output customization and compatibility switches

- `--marking <MARKING>`:
  - Reads a file and prepends its contents to generated outputs.
- `--no-typed-objects`:
  - C backend only.
  - Emits generic `Object` instead of typed object aliases.
- `--allow-undefined-behavior`:
  - Relaxes strict overflow/width checks in parsing/validation paths.
- `--bundle-params-by-size`:
  - Changes packed-parameter ordering from alignment-based (default) to size-based.
  - Kept for compatibility with previously generated buggy headers.

### Standard clap flags

- `-h, --help`: show help.
- `-V, --version`: show version.
