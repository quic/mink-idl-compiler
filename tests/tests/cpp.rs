#![cfg(not(miri))]

use idlc_test::{
    cpp::{create_itest1, create_itest2},
    implementation,
    interfaces::itest2::ITest2,
};

#[test]
fn implementation() {
    let cpp_wrapper = unsafe { create_itest2().unwrap() };
    let input = implementation::ITest1::default().into();
    assert_eq!(cpp_wrapper.entrypoint(Some(&input)), Ok(()));
}

#[test]
fn invoke() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input = unsafe { create_itest1(Default::default()).unwrap() };
    assert_eq!(rust_wrapper.entrypoint(Some(&input)), Ok(()));
}
