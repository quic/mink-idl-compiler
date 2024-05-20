# Changelog

All notable changes to this project will be documented in this file.

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
