// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn idlc_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_idlc"))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("idlc crate should have a workspace root")
        .to_path_buf()
}

fn fixture(path: &str) -> PathBuf {
    repo_root().join(path)
}

fn run_idlc(args: &[&str]) -> Output {
    Command::new(idlc_bin())
        .args(args)
        .output()
        .expect("idlc should execute")
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be monotonic since epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "idlc-cli-neg-{label}-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp dir should be creatable");
    dir
}

fn stderr_string(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn rejects_invalid_flag_combo_skel_with_rust() {
    let output_dir = unique_temp_dir("flags");
    let input = fixture("tests/idl/ITest.idl");

    let output = run_idlc(&[
        input.to_str().expect("utf-8 input path"),
        "--skel",
        "--rust",
        "-o",
        output_dir.to_str().expect("utf-8 output path"),
    ]);

    assert!(!output.status.success(), "expected clap to reject args");
    assert!(
        stderr_string(&output).contains("cannot be used with"),
        "expected conflict diagnostics in stderr, got:\n{}",
        stderr_string(&output)
    );
}

#[test]
fn fails_when_include_file_is_missing() {
    let dir = unique_temp_dir("missing-include");
    let input = dir.join("missing_include.idl");
    let output_file = dir.join("out.h");

    std::fs::write(
        &input,
        r#"include "does_not_exist.idl"
interface ITest {
  method ping();
};
"#,
    )
    .expect("input idl should be writable");

    let output = run_idlc(&[
        input.to_str().expect("utf-8 input path"),
        "-o",
        output_file.to_str().expect("utf-8 output path"),
    ]);

    assert!(
        !output.status.success(),
        "expected include resolution failure"
    );
    assert!(
        stderr_string(&output).contains("does_not_exist.idl"),
        "expected missing include name in stderr, got:\n{}",
        stderr_string(&output)
    );
}

#[test]
fn rejects_directory_output_for_c_codegen() {
    let output_dir = unique_temp_dir("c-out-dir");
    let input = fixture("tests/idl/ITest.idl");

    let output = run_idlc(&[
        input.to_str().expect("utf-8 input path"),
        "--c",
        "-o",
        output_dir.to_str().expect("utf-8 output path"),
    ]);

    assert!(!output.status.success(), "expected c output path mismatch");
    assert!(
        stderr_string(&output).contains("expects output file"),
        "expected output file mismatch message in stderr, got:\n{}",
        stderr_string(&output)
    );
}

#[test]
fn rejects_file_output_for_rust_codegen() {
    let dir = unique_temp_dir("rust-out-file");
    let output_file = dir.join("out.rs");
    let input = fixture("tests/idl/ITest.idl");
    std::fs::write(&output_file, "// placeholder").expect("output file should be creatable");

    let output = run_idlc(&[
        input.to_str().expect("utf-8 input path"),
        "--rust",
        "-o",
        output_file.to_str().expect("utf-8 output path"),
    ]);

    assert!(
        !output.status.success(),
        "expected rust output path mismatch"
    );
    assert!(
        stderr_string(&output).contains("expects output directory"),
        "expected output directory mismatch message in stderr, got:\n{}",
        stderr_string(&output)
    );
}
