use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let mut builder = cc::Build::new();
    let idlc = std::env::var("IDLC").expect("`IDLC` environment variable should've been set.");
    let build_out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let c_generated = build_out.join("c_headers");
    let rust_generated = build_out.join("rust");

    println!("cargo:rerun-if-changed={idlc}");
    println!("cargo:rerun-if-changed=idl/");
    _ = std::fs::create_dir(&c_generated);
    _ = std::fs::create_dir(&rust_generated);

    let interfaces = [Path::new("idl/ITest.idl"), Path::new("idl/ITest3.idl")];

    for interface in interfaces {
        let stem = interface.file_stem().unwrap().to_str().unwrap();
        let mut command = Command::new(&idlc)
            .args([
                interface.to_str().unwrap(),
                "-o",
                &(c_generated.to_string_lossy() + "/" + stem + ".h"),
            ])
            .spawn()
            .expect("`IDLC` couldn't be spawned");
        assert!(command.wait().unwrap().success());
        let mut command = Command::new(&idlc)
            .args([
                interface.to_str().unwrap(),
                "--skel",
                "-o",
                &(c_generated.to_string_lossy() + "/" + stem + "_invoke.h"),
            ])
            .spawn()
            .expect("`IDLC` couldn't be spawned");
        assert!(command.wait().unwrap().success());
        let mut command = Command::new(&idlc)
            .args([
                interface.to_str().unwrap(),
                "--rust",
                "-o",
                rust_generated.to_str().unwrap(),
            ])
            .spawn()
            .expect("`IDLC` couldn't be spawned");
        assert!(command.wait().unwrap().success());
    }

    println!("cargo:rerun-if-changed=c/");
    builder.file("c/invoke.c");
    builder.include("c");
    builder.include(c_generated);
    builder.flag("-Wno-unused-parameter");
    builder.flag("-Werror");
    builder.compile("ffi");
}
