// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#[test]
fn out_of_bound_allow_undefined_behavior() {
    let name = std::path::PathBuf::from("struct-verifier.idl");
    let idl = "const int32 FOO = 0x80000000;";
    idlc_ast::from_string(name, idl, true).unwrap();
}

#[test]
#[should_panic(expected = "isn't in range for type 'int32'")]
fn out_of_bound_disallow_undefined_behavior() {
    let name = std::path::PathBuf::from("struct-verifier.idl");
    let idl = "const int32 FOO = 0x80000000;";
    idlc_ast::from_string(name, idl, false).unwrap_err();
}
