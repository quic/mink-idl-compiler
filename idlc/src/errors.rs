// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#[inline]
pub fn check<T, E: std::fmt::Display>(r: Result<T, E>) -> T {
    match r {
        Ok(t) => t,
        Err(e) => {
            idlc_errors::unrecoverable!("{e}");
        }
    }
}
