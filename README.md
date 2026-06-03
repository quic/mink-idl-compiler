# The Mink IDL Compiler
Mink Interface Description Language (IDL) describes programming interfaces that can be used to communicate across security domain boundaries. It defines its own type system, independent of any particular target language. See [Mink IPC](https://github.com/qualcomm/minkipc) for more detail.

The Mink IDL _compiler_ generates target language header files which include bindings for Mink interfaces and their associates structures. The generated header files introduce proxy functions that facilitate method invocation using Mink's `Object_invoke` IPC mechanism. This abstraction shields both client-side proxy, called `stubs`, and implementation-side proxy, called `skeletons`, from the details of direct invocation.

It compiles `.idl` files into language bindings for:
- C
- C++
- Java
- Rust

## Branches

**main**: Primary development branch. Contributors should develop submissions based on this branch, and submit pull requests to this branch.

## Requirements

- Rust stable toolchain (`cargo`, `rustc`) - minimum supported Rust version = 1.81.0
- `clang` and `clang++` (integration tests compile C/C++ shims)
- Optional: nightly Rust for sanitizer and miri runs

Follow [these instructions](https://rust-lang.org/tools/install/) to install Rust.

If you've installed `rustup` in the past, you can update your installation by running `rustup update`.

## Usage

Run the compiler (file output):
```sh
cargo run -- tests/idl/ITest.idl -o /tmp/ITest.h
```
The target/output language is C, by default.


Generate Rust output (directory output):
```sh
mkdir -p /tmp/rust_out
cargo run -- tests/idl/ITest.idl --rust -o /tmp/rust_out
```

Run `cargo run -- --help` to see all available options.

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

## Development

For details on how to contribute, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Getting in Contact

How to contact maintainers. E.g. GitHub Issues, GitHub Discussions could be indicated for many cases. However a mail list or list of Maintainer e-mails could be shared for other types of discussions. E.g.

* [Report an Issue on GitHub](../../issues)
* [Open a Discussion on GitHub](../../discussions)
* [E-mail us](mailto:mink-idl-compiler@qti.qualcomm.com) for general questions

## License

_mink-idl-compiler_ is licensed under the [BSD-3-clause License](https://spdx.org/licenses/BSD-3-Clause.html). See [LICENSE.txt](LICENSE.txt) for the full license text.
