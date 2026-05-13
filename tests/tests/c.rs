// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

#![cfg(not(miri))]

use idlc_test::{
    c, cpp,
    implementation,
    interfaces::itest2::ITest2,
    interfaces::itest3::IDLVersion,
};

// The Rust test harness requires that we directly interface with Rust object
// only. However, underneath each of these, we can instantiate stub/proxy
// objects from each of the supported languages.

// The following tests all start from a C proxy and invoke to a different
// server/skeleton backend. That way we ensure that we are covering all of the
// possible combinations. Of course there is redundancy with other tests but
// this way we get better code coverage.
#[test]
fn to_c() {
    // Rust wrapper around ITest2 implemented in invoke.c
    let c_itest2 = unsafe { c::create_itest2().unwrap() };
    // ITest1 Rust object, implemented in test1.rs
    let input = implementation::ITest1::default().into();
    // ITest2 object tests ITest1 interface
    assert_eq!(c_itest2.entrypoint(Some(&input)), Ok(()));
    let c_itest3 = unsafe { c::create_itest3().unwrap() };
    assert_eq!(c_itest3.single_in(0xdead), Ok(()));
    {
        // Test that an IDL with no method attributes defaults to 1.0
        let expected = IDLVersion::new(1,0,0);
        let expected_val: u32 = expected.into();
        assert_eq!(c_itest3.api_version(), Ok(expected_val));
        assert_eq!(1, expected.major());
        assert_eq!(0, expected.minor());
        assert_eq!(0, expected.patch());
    }
}

#[test]
fn to_cpp() {
    // Rust wrapper around ITest2 implemented in invoke.c
    let c_itest2 = unsafe { c::create_itest2().unwrap() };
    let cpp_itest1 = unsafe { cpp::create_itest1(0).unwrap() };
    assert_eq!(c_itest2.entrypoint(Some(&cpp_itest1)), Ok(()));
}

#[test]
fn to_rust() {
    // Rust wrapper around ITest2 implemented in invoke.c
    let c_itest2 = unsafe { c::create_itest2().unwrap() };
    let rust_itest1: idlc_test::interfaces::itest1::ITest1 = implementation::ITest1::default().into();
    assert_eq!(c_itest2.entrypoint(Some(&rust_itest1)), Ok(()));
}
