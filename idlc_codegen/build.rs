// Copyright (c) Qualcomm Technologies, Inc. and/or its subsidiaries.
// SPDX-License-Identifier: BSD-3-Clause

pub fn main() {
    // Capture the Cargo manifest version for idlc/ - rather than idlc_codgeen/
    // - for printing to each generated file. This the version which gets
    // emitted when `--version` is passed.
    let manifest = include_str!("../idlc/Cargo.toml");
    let version = manifest
        .lines()
        .find_map(|line| {
            let line = line.trim();
            line.strip_prefix("version = \"")
                .and_then(|s| s.strip_suffix('"'))
        })
        .expect("version field not found in idlc/Cargo.toml");

    println!("cargo:rustc-env=IDLC_VERSION={version}");
    println!("cargo:rerun-if-changed=../idlc/Cargo.toml");
}
