// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#![cfg(not(miri))]

use idlc_test::{
    c::{create_itest1, create_itest2, create_itest3},
    implementation,
    interfaces::itest2::ITest2,
    interfaces::itest3::IDLVersion,
};

#[test]
fn implementation() {
    let c_wrapper = unsafe { create_itest2().unwrap() };
    let input = implementation::ITest1::default().into();
    assert_eq!(c_wrapper.entrypoint(Some(&input)), Ok(()));
    let c_wrapper_itest3 = unsafe { create_itest3().unwrap() };
    assert_eq!(c_wrapper_itest3.single_in(0xdead), Ok(()));
    {
        // Test that an IDL with no method attributes defaults to 1.0
        let expected = IDLVersion::new(1,0,0);
        let expected_val: u32 = expected.into();
        assert_eq!(c_wrapper_itest3.api_version(), Ok(expected_val));
        assert_eq!(1, expected.major());
        assert_eq!(0, expected.minor());
        assert_eq!(0, expected.patch());
    }
}

#[test]
fn invoke() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input = unsafe { create_itest1(Default::default()).unwrap() };
    assert_eq!(rust_wrapper.entrypoint(Some(&input)), Ok(()));
}
