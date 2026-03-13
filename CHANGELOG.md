# Changelog

All notable changes to this project will be documented in this file.

## [0.2.4](https://github.com/quic/mink-idl-compiler/compare/v0.2.3...v0.2.4) (2026-03-13)


### Bug Fixes

* Add rust-version to top-level Cargo.toml ([#19](https://github.com/quic/mink-idl-compiler/issues/19)) ([9ea8ae3](https://github.com/quic/mink-idl-compiler/commit/9ea8ae3678e5e314d37f6f4a7d26c97d4443df45))

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
