# Contributing

All types of contributions are encouraged and valued.

Build Instructions
----
Install Rust with Rustup using [Rustup](https://rustup.rs/).

Fork, then clone the repo:
``` sh
$ git clone {github repo}
```
Make your change. Add tests for your change. Make the tests pass:
```sh
$ cargo test
```
For `cargo coverage`, run the following commands:
``` sh
$ rustup default stable
$ rustup component add llvm-tools
$ export PATH=`rustc --print=sysroot`/lib/rustlib/x86_64-unknown-linux-gnu/bin/:${PATH}
```

Push to your fork and submit a pull request.
