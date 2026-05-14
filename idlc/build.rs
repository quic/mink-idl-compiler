// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

use std::process::Command;

pub fn main() {
    let output = Command::new("git")
        .args(["describe", "--always", "--abbrev=40", "--dirty"])
        .output();

    let git_hash = match output {
        Ok(output) if output.status.success() => {
            std::str::from_utf8(&output.stdout).unwrap().to_string()
        }
        _ => "unknown".to_string(),
    };

    println!("cargo:rustc-env=GIT_HASH={git_hash}");
}
