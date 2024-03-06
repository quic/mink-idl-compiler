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
