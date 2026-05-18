// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

#![cfg(not(miri))]

use idlc_test::{
    c, cpp,
    implementation,
    interfaces::itest3::IDLVersion,
};

// The following tests start from a C ITest2 implementation (invoke.c) and
// invoke every combination of ITest1 backend to verify cross-language FFI.

#[test]
fn to_c() {
    // ITest2 implemented in C (invoke.c)
    let c_itest2 = unsafe { c::create_itest2().unwrap() };
    // ITest1 implemented in C (invoke.c)
    let c_itest1 = unsafe { c::create_itest1(0).unwrap() };
    assert_eq!(c_itest2.entrypoint(Some(&c_itest1)), Ok(()));

    // ITest3 (extends ITest1) implemented in C — verify version defaults to 1.0
    // when no method attributes are specified in the IDL
    let c_itest3 = unsafe { c::create_itest3().unwrap() };
    assert_eq!(c_itest3.single_in(0xdead), Ok(()));
    let expected = IDLVersion::new(1, 0, 0);
    let expected_val: u32 = expected.into();
    assert_eq!(c_itest3.api_version(), Ok(expected_val));
    assert_eq!(1, expected.major());
    assert_eq!(0, expected.minor());
    assert_eq!(0, expected.patch());
}

#[test]
fn to_cpp() {
    // ITest2 implemented in C (invoke.c)
    let c_itest2 = unsafe { c::create_itest2().unwrap() };
    // ITest1 implemented in C++ (main.cpp)
    let cpp_itest1 = unsafe { cpp::create_itest1(0).unwrap() };
    assert_eq!(c_itest2.entrypoint(Some(&cpp_itest1)), Ok(()));
}

#[test]
fn to_rust() {
    // ITest2 implemented in C (invoke.c)
    let c_itest2 = unsafe { c::create_itest2().unwrap() };
    // ITest1 implemented in Rust (implementation/test1.rs)
    let rust_itest1: idlc_test::interfaces::itest1::ITest1 =
        implementation::ITest1::default().into();
    assert_eq!(c_itest2.entrypoint(Some(&rust_itest1)), Ok(()));
}
