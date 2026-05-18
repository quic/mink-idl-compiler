# The Mink IDL Compiler

## Background
Mink IDL describes programming interfaces that can be used to communicate across security domain boundaries. It defines its own type system, independent of any particular target language. See [Mink IPC](https://github.com/qualcomm/minkipc) for more detail.

The Mink IDL compiler generates target language header files which include bindings for Mink interfaces and their associates structures. The generated header files introduce proxy functions that facilitate method invocation using Mink's `Object_invoke` IPC mechanism. This abstraction shields both client-side proxy, called `stubs`, and implementation-side proxy, called `skeletons`, from the details of direct invocation.

## Getting Started
By default, the output language is C.

Run the compiler (file output):
```sh
cargo run -- tests/idl/ITest.idl -o /tmp/ITest.h
```

Generate Rust output (directory output):
```sh
mkdir -p /tmp/rust_out
cargo run -- tests/idl/ITest.idl --rust -o /tmp/rust_out
```

Run `cargo run -- --help` to see all available options.

See [CONTRIBUTING.md](CONTRIBUTING.md) for more detail.

## Notable Features
- Arrays of Objects
  - **bounded** arrays of Objects are allowed (e.g. `in IFoo[3] arr`)
  - Only 1 Object array allowed per direction per method (i.e. one array per `in` or `out`)
  - At most 16 Objects can be moved in either direction
- Objects in struct
  - struct fields can include Objects
  - e.g.
    ```C
    struct ObjInStruct {
      uint32[4] p1;
      ITest1 first_obj;
    };

    ```
  - Structs with Objects inside cannot be used in an array
  - Struct fields cannot be an array of Objects

## Restrictions
- No cyclic includes.
- Argument names within a method must be unique.
- Every struct is aligned to the size of the largest member, this rule holds for recursive structs as well.
- Interface consts must be unique.
  - Error definitions are considered as consts.
- Interface function name must be unique.
- Structs with Object in them directly or transitively cannot be used as an array in a function.
- Cannot have Object array and standalone Object parameters _with the same directionality_ in a method.
- Cannot have multiple Object arrays _with the same directionality_ in a method.
- New methods must be appended at the bottom since method op_codes are positional.

Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
SPDX-License-Identifier: BSD-3-Clause
