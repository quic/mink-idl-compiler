use qcom_core::object;

#[allow(unused, nonstandard_style, clippy::all, clippy::pedantic)]
pub mod interfaces {
    pub mod itest {
        include!(concat!(env!("OUT_DIR"), "/rust/itest.rs"));
    }
    pub mod itest1 {
        include!(concat!(env!("OUT_DIR"), "/rust/itest1.rs"));
    }
    pub mod itest2 {
        include!(concat!(env!("OUT_DIR"), "/rust/itest2.rs"));
    }
    pub mod itest3 {
        include!(concat!(env!("OUT_DIR"), "/rust/itest3.rs"));
    }
}

pub mod implementation;

extern "C" {
    #[allow(improper_ctypes)]
    pub fn create_c_itest2() -> interfaces::itest2::ITest2;
    #[allow(improper_ctypes)]
    pub fn create_c_itest1(value: u32) -> Option<interfaces::itest1::ITest1>;
}
