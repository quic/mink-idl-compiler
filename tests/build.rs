use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

static IDLC: OnceLock<PathBuf> = OnceLock::new();
fn idlc() -> &'static Path {
    IDLC.get_or_init(|| {
        PathBuf::from(
            std::env::var("IDLC").expect("`IDLC` enviropnment variable should've been set"),
        )
    })
}
static OUT_DIR: OnceLock<PathBuf> = OnceLock::new();
fn out_dir() -> &'static Path {
    OUT_DIR.get_or_init(|| PathBuf::from(std::env::var("OUT_DIR").unwrap()))
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Language {
    Rust,
    C { is_skel: bool },
}

fn build_interface(interface: &Path, output: &Path, lang: Language) {
    let mut args = vec![interface.to_str().unwrap()];
    match lang {
        Language::Rust => {
            args.push("--rust");
        }
        Language::C { is_skel } => {
            if is_skel {
                args.push("--skel");
            }
        }
    };
    args.extend_from_slice(&["-o", output.to_str().unwrap()]);
    let mut command = Command::new(idlc())
        .args(args)
        .spawn()
        .expect("`IDLC` should've been able to spawn");
    assert!(
        command.wait().unwrap().success(),
        "`IDLC` didn't complete successfully"
    );
}

fn main() {
    let mut builder = cc::Build::new();
    let idlc = std::env::var("IDLC").expect("`IDLC` environment variable should've been set.");
    let build_directory = |path: Option<&Path>, directory| {
        let mut base = out_dir().to_owned();
        base.push(directory);

        _ = std::fs::create_dir(&base);
        if let Some(path) = path {
            base.push(path);
        }

        base
    };
    let rust_generated = || build_directory(None, "rust");
    let c_generated = |path: Option<&Path>| build_directory(path, "c");

    println!("cargo:rerun-if-changed={idlc}");
    println!("cargo:rerun-if-changed=idl/");

    let interfaces = [Path::new("idl/ITest.idl"), Path::new("idl/ITest3.idl")];

    for interface in interfaces {
        let stem = interface.file_stem().unwrap().to_str().unwrap();

        build_interface(
            interface,
            &c_generated(Some(&PathBuf::from(format!("{stem}.h")))),
            Language::C { is_skel: false },
        );
        build_interface(
            interface,
            &c_generated(Some(&PathBuf::from(format!("{stem}_skel.h")))),
            Language::C { is_skel: true },
        );
        build_interface(interface, &rust_generated(), Language::Rust);
    }

    println!("cargo:rerun-if-changed=c/");
    builder.file("c/invoke.c");
    builder.include("c");
    builder.include(c_generated(None));
    builder.flag("-Wno-unused-parameter");
    builder.flag("-Werror");
    builder.compile("ffi");
}
