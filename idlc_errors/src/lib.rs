// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use env_logger::Env;
pub use log::{debug, error, info, trace, warn};

#[inline]
pub fn init() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
}

#[macro_export]
macro_rules! unrecoverable {
    ($($arg:tt)+) => {
        panic!("{}", format_args!($($arg)+));
    }
}
