use idlc_test::{
    implementation,
    interfaces::{
        itest::{F1, SUCCESS_FLAG},
        itest1::ITest1,
        itest2::ITest2,
        itest3::ITest3,
    },
};

#[test]
fn rust_calls_rust_itest1() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input = implementation::ITest1::default().into();
    assert_eq!(rust_wrapper.test_obj_in(Some(&input)), Ok(SUCCESS_FLAG));
}

#[test]
fn rust_calls_rust_itest1_array() {
    let rust_wrapper: ITest2 = implementation::ITest2::new().into();
    let input_array = [
        Some(implementation::ITest1::default().into()),
        Some(implementation::ITest1::default().into()),
        Some(implementation::ITest1::default().into()),
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

#[test]
fn test_bundle_inout() {
    let mut _lenout = 0;
    let test2: ITest2 = implementation::ITest2::new().into();
    let result = test2
        .test_bundle(&[], &mut [], &mut _lenout, 15, &[], 25, 35)
        .unwrap();
    assert_eq!(result.0, 25);
    assert_eq!(result.1, 45);
    assert_eq!(result.2, 65);
}

#[test]
fn test_array() {
    let test2: ITest2 = implementation::ITest2::new().into();
    let input = vec![
        F1 { a: 1 },
        F1 { a: 2 },
        F1 { a: 3 },
        F1 { a: 4 },
        F1 { a: 5 },
    ];
    let mut output = [F1 { a: 0 }; 16];
    let mut lenout: usize = 0;
    let mut _lenout: usize = 0;
    let result = test2
        .test_array(
            &input,
            &mut output,
            &mut lenout,
            &input[0],
            &[],
            &mut [],
            &mut _lenout,
            13,
        )
        .unwrap();
    assert_eq!(result.1, 13);
    assert_eq!(result.0.a as usize, input.len());
    assert_eq!(lenout, input.len());
    for i in 0..lenout {
        assert_eq!(output[i].a, input[i].a + 5);
    }
}

#[test]
fn test_pass_inheriting_obj() {
    let test2: ITest2 = implementation::ITest2::new().into();
    let test3: ITest3 = implementation::ITest3::default().into();
    assert_eq!(test2.test_obj_in(Some(&test3)), Ok(SUCCESS_FLAG));
}

#[test]
fn test_custom_error() {
    let f = F1 { a: 0 };
    let test2: ITest2 = implementation::ITest2::new().into();
    assert_eq!(
        test2.test_f2(&f),
        Err(idlc_test::interfaces::itest2::MY_CUSTOM_ERROR)
    );
}

#[test]
fn test_get_content_from_ref() {
    use idlc_test::interfaces::itest1::downcast_concrete;

    let test1 = ITest1::from(implementation::ITest1::new(0xdeadcafe));
    let test2: ITest2 = implementation::ITest2::new().into();
    assert_eq!(
        downcast_concrete::<implementation::ITest1>(&test1)
            .unwrap()
            .value,
        0xdeadcafe
    );
    assert!(downcast_concrete::<implementation::ITest1>(&test2).is_none());
}
