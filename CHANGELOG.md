# Changelog

All notable changes to this project will be documented in this file.

## [0.2.4](https://github.com/quic/mink-idl-compiler/compare/v0.2.3...v0.2.4) (2026-03-20)


### Bug Fixes

* Add rust-version to top-level Cargo.toml ([#19](https://github.com/quic/mink-idl-compiler/issues/19)) ([9ea8ae3](https://github.com/quic/mink-idl-compiler/commit/9ea8ae3678e5e314d37f6f4a7d26c97d4443df45))
* Bundled struct field order matches legacy behavior ([#14](https://github.com/quic/mink-idl-compiler/issues/14)) ([cf5cc97](https://github.com/quic/mink-idl-compiler/commit/cf5cc9752f24baa8b19e8bec1766b931c54e42c8))


### Miscellaneous

* **release:** bump crate versions ([9d1b9af](https://github.com/quic/mink-idl-compiler/commit/9d1b9af6163b0bc1f8f69c65e013689385481a8b))


### Integration/Automation

* automate release PRs and assets ([#18](https://github.com/quic/mink-idl-compiler/issues/18)) ([165bbf1](https://github.com/quic/mink-idl-compiler/commit/165bbf1fbe36ef33a1ccd9a623d100bed831ced5))
* Change release-please `release-type` to `rust` ([#22](https://github.com/quic/mink-idl-compiler/issues/22)) ([9567461](https://github.com/quic/mink-idl-compiler/commit/956746180bb99b9afd1c15743ab296eda5f59084))
* Crates have unique versions ([#29](https://github.com/quic/mink-idl-compiler/issues/29)) ([1d54407](https://github.com/quic/mink-idl-compiler/commit/1d54407f5c308d592b7fbf4c07581a500fabe9d3))
* fix workflow files ([#20](https://github.com/quic/mink-idl-compiler/issues/20)) ([34557f2](https://github.com/quic/mink-idl-compiler/commit/34557f2ec51a80c598554d4d872ff75c84cd6f0a))
* Push workspace version to each crate ([#23](https://github.com/quic/mink-idl-compiler/issues/23)) ([e45d50a](https://github.com/quic/mink-idl-compiler/commit/e45d50a84e7bd4eb54a1e46df4e57de7f37454ff))
* Release please no plugin ([#25](https://github.com/quic/mink-idl-compiler/issues/25)) ([e00cd44](https://github.com/quic/mink-idl-compiler/commit/e00cd44f67ac409d29cfcb05f64435cc6a77a11a))
* Simple release please ([#27](https://github.com/quic/mink-idl-compiler/issues/27)) ([93eb3dd](https://github.com/quic/mink-idl-compiler/commit/93eb3dd77f782f3a828e9abd959317f8037b5b7d))
* Simple release please extra file ([#28](https://github.com/quic/mink-idl-compiler/issues/28)) ([c24387e](https://github.com/quic/mink-idl-compiler/commit/c24387e73b279f0a590754b09a862d610389f5bd))
* Switch to tag-based automation ([#26](https://github.com/quic/mink-idl-compiler/issues/26)) ([818fa50](https://github.com/quic/mink-idl-compiler/commit/818fa50290559d0b2a78cad4936db88cf8c35cd2))
* Virtual workspace fixup ([#24](https://github.com/quic/mink-idl-compiler/issues/24)) ([0358798](https://github.com/quic/mink-idl-compiler/commit/035879866e6e0da1160d55b55ea8060575a73fae))


### Documentation

* Update the guidance for releases ([ab84b64](https://github.com/quic/mink-idl-compiler/commit/ab84b64e6ff8d81567580e1c9de361a6bd0664bc))

## [0.2.3] - 2025-11-27

### Fixes
- Update Cargo.lock file

## [0.2.2] - 2025-11-18

### Fixes
- Fix Cargo.lock for Yocto compilation support

## [0.2.1] - 2025-03-12

### Fixes
- changing C++ impl from const to constexpr
- Fix multiple inheritance issues for C++ header

### Enhancements
- Add a new test for interface inheritance

## [0.2.0] - 2024-10-17

### Enhancements
- Add new flag to generate copyright markings on generated files

## [0.1.4] - 2024-10-17

### Fixes
- C/CPP Codegen: Fixed bundled output initialization

## [0.1.3] - 2024-09-11

### Fixes
- C/CPP Codegen: Removed unnecessary whitespace

## [0.1.2] - 2024-05-20

### Fixes
- CPP Codegen: Fixed the type of const value

## [0.1.1] - 2024-05-14

### Fixes
- Initialization for bundled struct

## [0.1.0] - 2024-04-30

### Enhancements
- Object arrays implementation for Rust
- Support off-target testing for C/CPP/Rust
- Add support for attributes
- #[optional] attribute adds logic to return 'Object_ERROR_INVALID' if method is undefined by the service implementor

### Fixes
- Object arrays codegen fix for C/CPP
- Index issues of ObjectArg fix for C/CPP

### Breaking Changes
- Argument names within a method must be unique.
- Object array needs to be bounded.
- Struct name should not be same as any other interface name.
