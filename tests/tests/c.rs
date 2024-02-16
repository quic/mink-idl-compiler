#![cfg(not(miri))]

use idlc_test::{
    c::{create_itest1, create_itest2},
    implementation,
    interfaces::{itest::SUCCESS_FLAG, itest2::ITest2},
};

#[test]
fn rust_itest1() {
    let c_wrapper = unsafe { create_itest2().unwrap() };
    let input = implementation::ITest1::default().into();
    assert_eq!(c_wrapper.test_obj_in(Some(&input)), Ok(SUCCESS_FLAG));
}

#[test]
fn c_itest1() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input = unsafe { create_itest1(Default::default()).unwrap() };
    assert_eq!(rust_wrapper.test_obj_in(Some(&input)), Ok(SUCCESS_FLAG));
}

#[test]
fn rust_itest1_array() {
    let c_wrapper = unsafe { create_itest2().unwrap() };
    let input_array = [
        Some(implementation::ITest1::default().into()),
        Some(implementation::ITest1::default().into()),
        Some(implementation::ITest1::default().into()),
    ];
    assert_eq!(c_wrapper.test_obj_array_in(&input_array), Ok(SUCCESS_FLAG));

    let (objs, value) = c_wrapper.test_obj_array_out().unwrap();
    assert!(value == SUCCESS_FLAG);
    for obj in objs {
        assert_eq!(c_wrapper.test_obj_in(obj.as_ref()), Ok(SUCCESS_FLAG));
    }
}

#[test]
fn c_itest1_array() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input_array = [
        Some(unsafe { create_itest1(Default::default()).unwrap() }),
        Some(unsafe { create_itest1(Default::default()).unwrap() }),
        Some(unsafe { create_itest1(Default::default()).unwrap() }),
    ];
    assert_eq!(
        rust_wrapper.test_obj_array_in(&input_array),
        Ok(SUCCESS_FLAG)
    );

    let (objs, value) = rust_wrapper.test_obj_array_out().unwrap();
    assert!(value == SUCCESS_FLAG);
    for obj in objs {
        assert_eq!(rust_wrapper.test_obj_in(obj.as_ref()), Ok(SUCCESS_FLAG));
    }
}
