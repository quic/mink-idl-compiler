pub use log::{debug, error, info, trace, warn};

#[inline]
pub fn init() {
    env_logger::init();
}

#[macro_export]
macro_rules! unrecoverable {
    ($($arg:tt)+) => {
        panic!("{}", format_args!($($arg)+));
    }
}
