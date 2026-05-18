# `idlc_test` — Integration Test Suite

This crate is the end-to-end integration test suite for `idlc`, the MINK IDL
compiler. It compiles test IDL files through the compiler, exercises the
generated headers in C, C++, and Rust, and runs a single Rust binary that
calls interface methods across every combination of language boundary.

## How it works

### Build pipeline (`build.rs`)

The build script drives the entire code-generation and compilation pipeline:

1. **IDL compilation** — `idlc` is invoked on each `.idl` file in `idl/` to
   produce:
   - `c/ITest.h`, `c/ITest_invoke.h` — C client proxy macros and server
     skeleton dispatcher
   - `cpp/ITest.hpp`, `cpp/ITest_invoke.hpp` — C++ client proxy class and
     server skeleton base class
   - `rust/itest1.rs` etc. — Rust trait, proxy struct, and invoke dispatcher,
     included directly via `include!()` in `src/lib.rs`

2. **C static library** (`c-ffi`) — `c/invoke.c` is compiled with the
   generated C headers. It provides `create_c_itest1`, `create_c_itest2`, and
   `create_c_itest3` as `extern "C"` symbols.

3. **C++ static library** (`cpp-ffi`) — `cpp/main.cpp` is compiled with both
   the generated C++ headers and the C headers (for shared struct definitions).
   It provides `create_cpp_itest1`, `create_cpp_itest2`, and
   `create_cpp_itest3` as `extern "C"` symbols.

4. **Rust library** — `src/lib.rs` links the two static libraries and exposes
   `c::create_itest{1,2,3}` and `cpp::create_itest{1,2,3}` as unsafe FFI
   wrappers, along with native Rust implementations in `src/implementation/`.

### Object model

All three language backends share a common `Object` ABI:

```c
typedef struct {
    int32_t (*invoke)(void *ctx, ObjectOp op, ObjectArg *args, ObjectCounts k);
    void *ctx;
} Object;
```

A `retain`/`release` lifecycle is managed through the `OP_RETAIN`/`OP_RELEASE`
operations dispatched through `invoke`. The generated client-side proxy (C
macros, C++ `ProxyBase`, Rust `ITest1` struct) holds an `Object` and
translates typed method calls into `invoke(op, args, counts)` calls.

### Interfaces under test

#### `ITest.idl` — core test interfaces

- **`ITest1`** — the primary interface under test, with 25+ methods covering:
  - Scalar in/out primitives (`single_in`, `single_out`, `add_1000`)
  - Buffer arguments alongside scalar primitives (`single_primitive_in/out`,
    `multiple_primitive`)
  - Struct arguments by value (`in_struct`, `out_struct`, `in_small_struct`,
    `out_small_struct`, `bundled_with_unbundled`, `primitive_plus_struct_*`)
  - Arrays of structs (`struct_array_in/out`, `primitive_array_in_struct`)
  - Arrays of interface objects (`test_obj_array_in/out`)
  - Structs containing interface objects (`objects_in_struct`)
  - An optional/unimplemented method (`unimplemented`)
  - Versioned methods with `#[version]` attributes (`derive_v*`)

- **`ITest2`** — a single-method orchestrator: `entrypoint(in ITest1 o)`
  receives an `ITest1` object and exhaustively exercises its API.

#### `ITest3.idl` — inheritance test interfaces

- **`ITest3`** — extends `ITest1` with one additional method (`extra_test3`).
  Used to verify that the compiler correctly emits inherited method dispatchers
  and that the default API version is `1.0.0` when no `#[version]` attributes
  are present.

- **`ITest4`** — extends `ITest3`, verifying multi-level inheritance.

### Language implementations

Each language provides three implementations that mirror each other:

| Symbol                     | Language | Source                        |
|----------------------------|----------|-------------------------------|
| `create_c_itest1(value)`   | C        | `c/invoke.c`                  |
| `create_c_itest2()`        | C        | `c/invoke.c`                  |
| `create_c_itest3()`        | C        | `c/invoke.c`                  |
| `create_cpp_itest1(value)` | C++      | `cpp/main.cpp`                |
| `create_cpp_itest2()`      | C++      | `cpp/main.cpp`                |
| `create_cpp_itest3()`      | C++      | `cpp/main.cpp`                |
| `implementation::ITest1`   | Rust     | `src/implementation/test1.rs` |
| `implementation::ITest2`   | Rust     | `src/implementation/test2.rs` |
| `implementation::ITest3`   | Rust     | `src/implementation/test1.rs` |

The C++ implementations in `cpp/main.cpp` delegate their method bodies to the
C implementations in `c/invoke.c` via the shared `c::` namespace, so the
actual logic lives in one place. The Rust implementations in
`src/implementation/` are independent.

All three `ITest1` implementations share the same test fixtures and invariants:
`TRUTH` (a `Collection`) and `TRUTH2` (a `SingleEncapsulated`). The helper
function `test_singular_object` (C/C++ in `c/invoke.c`, Rust in
`src/implementation/mod.rs`) exhaustively exercises all scalar, struct, and
array methods on a single `ITest1` object, including an `api_version` check
that asserts `2.0.0`.

### Test harness (`tests/`)

The Rust test harness in `tests/` exercises all 9 combinations of ITest2
implementor × ITest1 implementor across the three languages:

| Test file                | ITest2 (caller) | ITest1 (callee) |
|--------------------------|-----------------|-----------------|
| `tests/main.rs::to_c`    | Rust            | C               |
| `tests/main.rs::to_cpp`  | Rust            | C++             |
| `tests/main.rs::to_rust` | Rust            | Rust            |
| `tests/c.rs::to_c`       | C               | C               |
| `tests/c.rs::to_cpp`     | C               | C++             |
| `tests/c.rs::to_rust`    | C               | Rust            |
| `tests/cpp.rs::to_c`     | C++             | C               |
| `tests/cpp.rs::to_cpp`   | C++             | C++             |
| `tests/cpp.rs::to_rust`  | C++             | Rust            |

`tests/main.rs` additionally contains thread-safety tests
(`implementation_and_invoke_sync` and `implementation_and_invoke_send`) that
run the Rust→Rust path from 10 concurrent threads.

Each `to_c` test also constructs an `ITest3` object (same language) and
verifies that the API version defaults to `1.0.0` when no `#[version]`
attributes are present in the IDL.

### Running the tests

```sh
# Build idlc first (required by the test build script)
cargo build -p idlc

# Run all integration tests
cargo test -p idlc_test
```

The `IDLC` environment variable can point to an alternate compiler binary:

```sh
IDLC=/path/to/idlc cargo test -p idlc_test
```

### What is NOT tested

- **Java** — Java code generation exists in `idlc` but Java compilation is not
  yet wired into this test suite.
