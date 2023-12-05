const TRUTH: crate::interfaces::itest::Collection = crate::interfaces::itest::Collection {
    a: 0,
    b: 1,
    c: 2,
    d: 3,
};

mod test1;
mod test2;

pub use test1::{ITest1, ITest3};
pub use test2::ITest2;
