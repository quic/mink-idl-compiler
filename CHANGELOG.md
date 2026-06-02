# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0](https://github.com/quic/mink-idl-compiler/compare/v0.3.0...v1.0.0) (2026-06-02)


### ⚠ BREAKING CHANGES

* C++ implementation of IDL versioning with old impl_base.hpp ([#48](https://github.com/quic/mink-idl-compiler/issues/48))
* Remove `#[optional]` function attribute ([#16](https://github.com/quic/mink-idl-compiler/issues/16))

### Bug Fixes

* C++ implementation of IDL versioning with old impl_base.hpp ([#48](https://github.com/quic/mink-idl-compiler/issues/48)) ([69a0fb5](https://github.com/quic/mink-idl-compiler/commit/69a0fb589142e4012789c80def459e6c883f7ab7))
* Disallow array of Mink Object in a struct ([#46](https://github.com/quic/mink-idl-compiler/issues/46)) ([0438b0b](https://github.com/quic/mink-idl-compiler/commit/0438b0bf2d83f257937e192a435b1f673cb322cd))
* Prefix reserved words ([#52](https://github.com/quic/mink-idl-compiler/issues/52)) ([ce31e73](https://github.com/quic/mink-idl-compiler/commit/ce31e7307c5071aca6675aa5a9e11964bf2e9239))
* Refactor tests ([#50](https://github.com/quic/mink-idl-compiler/issues/50)) ([2557353](https://github.com/quic/mink-idl-compiler/commit/2557353638ca93a774a2e339d55a37ddd328eef8))
* Remove `#[optional]` function attribute ([#16](https://github.com/quic/mink-idl-compiler/issues/16)) ([3d11875](https://github.com/quic/mink-idl-compiler/commit/3d1187533eba753acc07300eb8ac8be7ed60df55))
* **tests:** CLI negatives tests work for all ARCH ([#41](https://github.com/quic/mink-idl-compiler/issues/41)) ([73a9a3a](https://github.com/quic/mink-idl-compiler/commit/73a9a3a0a2ccdcb09216289a6c8ab17039838867))


### Miscellaneous

* clarify input/output metavar names ([77b0939](https://github.com/quic/mink-idl-compiler/commit/77b093997c5f5a28ecfe7b10eac3ccf15aefca4a))
* **release:** Build and upload exe for releases ([34c503f](https://github.com/quic/mink-idl-compiler/commit/34c503f3bdf1993897e05bfd6d3c94066b1568bf))
* **release:** bump crate versions ([89f90e3](https://github.com/quic/mink-idl-compiler/commit/89f90e32dc2a270ae032469a18a8ad7b5a76f290))
* Update copyright markings on all files ([d219801](https://github.com/quic/mink-idl-compiler/commit/d219801ff3bf343818f9c804ad94e0e7095811d3))


### Integration/Automation

* Build Windows executable ([#43](https://github.com/quic/mink-idl-compiler/issues/43)) ([1a7ba9e](https://github.com/quic/mink-idl-compiler/commit/1a7ba9ecda7b089eee50b9d068e248a6eea34b69))


### Code Refactoring

* **codegen:** params args ([#49](https://github.com/quic/mink-idl-compiler/issues/49)) ([39f9157](https://github.com/quic/mink-idl-compiler/commit/39f91571fd84da227bafbd9a878b334b6ae5a6cf))

## [0.3.0](https://github.com/quic/mink-idl-compiler/compare/v0.2.4...v0.3.0) (2026-04-28)


### Features

* Interface versioning with method attributes ([03a4aa1](https://github.com/quic/mink-idl-compiler/commit/03a4aa1befd3b7df2c9755fb54fa11c563cd3b12))


### Bug Fixes

* **idlc:** Make GIT_HASH optional for long version ([#33](https://github.com/quic/mink-idl-compiler/issues/33)) ([f3a84fe](https://github.com/quic/mink-idl-compiler/commit/f3a84feb97de8529d4fa8361eaa909696860b8f9))


### Miscellaneous

* **release:** bump crate versions ([f07ca6b](https://github.com/quic/mink-idl-compiler/commit/f07ca6b0acf28282c33a9278ab8f80566a72fd07))
* Remove author names from sub-crates ([339295a](https://github.com/quic/mink-idl-compiler/commit/339295ac7d31625956ebf5a3eabda155ae2880f3))
* Sub-crates versioned independently ([bdc6d79](https://github.com/quic/mink-idl-compiler/commit/bdc6d79ddce8b345b3dd709a1128121682c960bb))
* **test:** update object.h and C++ base headers ([3c82a8c](https://github.com/quic/mink-idl-compiler/commit/3c82a8c97af325e28c4e7362d5091236a33465ce))


### Integration/Automation

* Pre-built binary renamed to idlc-{target} ([cbbef30](https://github.com/quic/mink-idl-compiler/commit/cbbef300f394feb940960d5812f5f98e44c6e1ad))


### Documentation

* Update documentation ([#36](https://github.com/quic/mink-idl-compiler/issues/36)) ([4a87a0c](https://github.com/quic/mink-idl-compiler/commit/4a87a0ca1c3d7da71a338d70b4490f3156c712fb)), closes [#34](https://github.com/quic/mink-idl-compiler/issues/34)


### Code Refactoring

* C/C++ header emission logic ([4028966](https://github.com/quic/mink-idl-compiler/commit/402896623ba07607471914c1b624c58e75f8de02))
* C++ #include ordering ([2bcbf12](https://github.com/quic/mink-idl-compiler/commit/2bcbf127daef6c4de21ebe9d7b73fd930da8649f))
* cargo clippy 1.95.0 changes ([ed91200](https://github.com/quic/mink-idl-compiler/commit/ed91200061f950e8f40b4f67d1b01008c4b58d80))
* **idlc:** Split main and lib ([13637c2](https://github.com/quic/mink-idl-compiler/commit/13637c2d936f16679d275371aa1d54602ac47153))
* Shared closures for logic re-use ([37cdc0c](https://github.com/quic/mink-idl-compiler/commit/37cdc0c74a5986cbaa223e6c748f7defd92d8699))
* struct fields use {INDENT} ([6873e0f](https://github.com/quic/mink-idl-compiler/commit/6873e0f1990460468cc49274242b11c1225dc77f))
* Use programmable, not hard-coded, names ([0e0b4bf](https://github.com/quic/mink-idl-compiler/commit/0e0b4bf8e8b24888ab8cebf32abf711a1de579da))

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
