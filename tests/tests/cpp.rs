#![cfg(not(miri))]

use idlc_test::{
    cpp::{create_itest1, create_itest2},
    implementation,
    interfaces::{itest::SUCCESS_FLAG, itest2::ITest2},
};

#[test]
fn rust_itest1() {
    let cpp_wrapper = unsafe { create_itest2().unwrap() };
    let input = implementation::ITest1::default().into();
    assert_eq!(cpp_wrapper.test_obj_in(Some(&input)), Ok(SUCCESS_FLAG));
}

#[test]
fn cpp_itest1() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input = unsafe { create_itest1(Default::default()).unwrap() };
    assert_eq!(rust_wrapper.test_obj_in(Some(&input)), Ok(SUCCESS_FLAG));
}
