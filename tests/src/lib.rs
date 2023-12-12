use qcom_core::object;

#[allow(
    unused,
    nonstandard_style,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]
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

#[allow(improper_ctypes)]
pub mod c {
    extern "C" {
        #[link_name = "create_c_itest1"]
        pub fn create_itest1(value: u32) -> Option<crate::interfaces::itest1::ITest1>;

        #[link_name = "create_c_itest2"]
        pub fn create_itest2() -> Option<crate::interfaces::itest2::ITest2>;
    }
}

#[allow(improper_ctypes)]
pub mod cpp {
    extern "C" {
        #[link_name = "create_cpp_itest1"]
        pub fn create_itest1(value: u32) -> Option<crate::interfaces::itest1::ITest1>;

        #[link_name = "create_cpp_itest2"]
        pub fn create_itest2() -> Option<crate::interfaces::itest2::ITest2>;

    }
}
