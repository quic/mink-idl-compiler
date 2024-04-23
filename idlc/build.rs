// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::process::Command;

pub fn main() {
    let output = Command::new("git")
        .args(["describe", "--always", "--abbrev=40", "--dirty"])
        .output()
        .unwrap();
    let git_hash = std::str::from_utf8(&output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
