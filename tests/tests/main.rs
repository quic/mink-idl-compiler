use idlc_test::{
    implementation::{self, ITest1},
    interfaces::itest2::ITest2,
};

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
