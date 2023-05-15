

The Mink IDL Compiler
====

Background
----
Mink IDL is used to describe programming interfaces that can be used to communicate across security domain boundaries. Once an interface is described in an IDL source file, the Mink IDL compiler can generate target language header files. It defines its own type system, independent of any particular target language. However, it's worth noting that certain aspects of the Java output are specifically tailored for Android development.

Additionally, the generated header files introduce proxy functions that faciliate method invocation using Mink's `Object_invoke` IPC mechanism. This abstraction shields both client-side proxy, called `stubs`, and implementation-side proxy, called `skeletons`, from the details of direct invocation.

### Build Mink IDL Compiler
Run the following command to build the project:
```sh
$ idlc <IDL File> -o <Output file>
```

#### Options
Run `idlc --help` to see the available options.

### Testing

First run `cargo build` from the root directory.
Navigate to the `tests` directory, and run the following command.
```sh
$ RUST_BACKTRACE=1 cargo test
```
`IDLC` defaults to `../target/debug/idlc`

Restriction
----
This is a guideline for the new MinkIDL compiler. This serve as a precaution to maintain backward compatibility.

- No cyclic includes.
- Argument names within a method must be unique.
- Ensures every struct is aligned to the size of the largest member, this rule
  holds for recursive structs as well.
- Interface error name should not be duplicated.
- Interface function name should not be duplicated.
- Interface consts should not be duplicated, error definitions are also considered consts.
- Structs with Object in them directly or transitively cannot be used as an array in a function.
- Cannot have input Object array + any type of input objects. Same for Output.
- Cannot have multiple input Object arrays in a method. Same for Output.
- When adding new method in interface, this should be appened at the bottom. 

Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
SPDX-License-Identifier: BSD-3-Clause
