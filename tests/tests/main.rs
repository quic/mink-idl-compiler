// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

#[cfg(not(miri))]
use idlc_test::{c, cpp};
use idlc_test::{
    implementation::{self, ITest1},
    interfaces::itest2::ITest2,
};

// The following tests start from a Rust ITest2 implementation (test2.rs) and
// invoke every combination of ITest1 backend to verify cross-language FFI.

#[cfg(not(miri))]
#[test]
fn to_c() {
    // ITest2 implemented in Rust (implementation/test2.rs)
    let rust_itest2: ITest2 = implementation::ITest2::new().into();
    // ITest1 implemented in C (invoke.c)
    let c_itest1 = unsafe { c::create_itest1(0).unwrap() };
    assert_eq!(rust_itest2.entrypoint(Some(&c_itest1)), Ok(()));

    // ITest3 (extends ITest1) implemented in Rust — verify version defaults to
    // 1.0 when no method attributes are specified in the IDL
    let rust_itest3: idlc_test::interfaces::itest3::ITest3 =
        implementation::ITest3::default().into();
    assert_eq!(rust_itest3.single_in(0xdead), Ok(()));
    let expected = idlc_test::interfaces::itest1::IDLVersion::new(1, 0, 0);
    let expected_val: u32 = expected.into();
    assert_eq!(rust_itest3.api_version(), Ok(expected_val));
    assert_eq!(1, expected.major());
    assert_eq!(0, expected.minor());
    assert_eq!(0, expected.patch());
}

#[cfg(not(miri))]
#[test]
fn to_cpp() {
    // ITest2 implemented in Rust (implementation/test2.rs)
    let rust_itest2: ITest2 = implementation::ITest2::new().into();
    // ITest1 implemented in C++ (main.cpp)
    let cpp_itest1 = unsafe { cpp::create_itest1(0).unwrap() };
    assert_eq!(rust_itest2.entrypoint(Some(&cpp_itest1)), Ok(()));
}

#[test]
fn to_rust() {
    // ITest2 implemented in Rust (implementation/test2.rs)
    let rust_itest2: ITest2 = implementation::ITest2::new().into();
    // ITest1 implemented in Rust (implementation/test1.rs)
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
