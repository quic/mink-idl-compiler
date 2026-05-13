// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

use idlc_test::{
    c,
    cpp,
    implementation::{self, ITest1},
    interfaces::itest2::ITest2,
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
    // ITest2 Rust object, implemented in test2.rs
    let rust_itest2: ITest2 = implementation::ITest2::new().into();
    // Rust wrapper around ITest1 implemented in invoke.c
    let c_itest1 = unsafe { c::create_itest1(0).unwrap() };
    // ITest2 object tests ITest1 interface
    assert_eq!(rust_itest2.entrypoint(Some(&c_itest1)), Ok(()));
    let rust_itest3: idlc_test::interfaces::itest3::ITest3 = implementation::ITest3::default().into();
    assert_eq!(rust_itest3.single_in(0xdead), Ok(()));
    {
        // Test that an IDL with no method attributes defaults to 1.0
        let expected = idlc_test::interfaces::itest1::IDLVersion::new(1,0,0);
        let expected_val: u32 = expected.into();
        assert_eq!(rust_itest3.api_version(), Ok(expected_val));
        assert_eq!(1, expected.major());
        assert_eq!(0, expected.minor());
        assert_eq!(0, expected.patch());
    }
}

#[test]
fn to_cpp() {
    let rust_itest2: ITest2 = implementation::ITest2::new().into();
    let cpp_itest1 = unsafe { cpp::create_itest1(0).unwrap() };
    assert_eq!(rust_itest2.entrypoint(Some(&cpp_itest1)), Ok(()));
}

#[test]
fn to_rust() {
    let rust_itest2: ITest2 = implementation::ITest2::new().into();
    let rust_itest1 = ITest1::default().into();
    assert_eq!(rust_itest2.entrypoint(Some(&rust_itest1)), Ok(()));
}

#[test]
fn implementation_and_invoke() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input = ITest1::default().into();
    assert_eq!(rust_wrapper.entrypoint(Some(&input)), Ok(()));
}

#[test]
fn implementation_and_invoke_sync() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input = ITest1::default().into();
    std::thread::scope(|s| {
        let mut threads = vec![];
        (0..10).for_each(|_| {
            threads.push(s.spawn(|| {
                assert_eq!(rust_wrapper.entrypoint(Some(&input)), Ok(()));
            }));
        });
        threads.into_iter().for_each(|t| t.join().unwrap());
    })
}

#[test]
fn implementation_and_invoke_send() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input: idlc_test::interfaces::itest1::ITest1 = ITest1::default().into();
    std::thread::scope(|s| {
        let mut threads = vec![];
        (0..10).for_each(|_| {
            let r_clone = rust_wrapper.clone();
            let input_clone = input.clone();
            threads.push(s.spawn(move || {
                assert_eq!(r_clone.entrypoint(Some(&input_clone)), Ok(()));
            }));
        });
        threads.into_iter().for_each(|t| t.join().unwrap());
    })
}
