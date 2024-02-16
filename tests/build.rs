use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

static IDLC: OnceLock<PathBuf> = OnceLock::new();
fn idlc() -> &'static Path {
    IDLC.get_or_init(|| {
        let p = PathBuf::from(
            std::env::var("IDLC").unwrap_or_else(|_| "../target/debug/idlc".to_string()),
        );
        assert!(
            p.exists(),
            "`idlc` not found @ ../target/debug/idlc nor was it set using `IDLC`
        environment variable"
        );
        p
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
    Cpp { is_skel: bool },
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
        Language::Cpp { is_skel } => {
            args.push("--cpp");
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
    let cpp_generated = |path: Option<&Path>| build_directory(path, "cpp");

    println!("cargo:rerun-if-changed={}", idlc().display());
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
            &c_generated(Some(&PathBuf::from(format!("{stem}_invoke.h")))),
            Language::C { is_skel: true },
        );
        build_interface(
            interface,
            &cpp_generated(Some(&PathBuf::from(format!("{stem}.hpp")))),
            Language::Cpp { is_skel: false },
        );
        build_interface(
            interface,
            &cpp_generated(Some(&PathBuf::from(format!("{stem}_invoke.hpp")))),
            Language::Cpp { is_skel: true },
        );
        build_interface(interface, &rust_generated(), Language::Rust);
    }

    println!("cargo:rerun-if-changed=c/");
    let mut c_ffi = cc::Build::new();
    c_ffi.file("c/invoke.c");
    c_ffi.include("c");
    c_ffi.include(c_generated(None));
    c_ffi.flag("-Wno-unused-parameter");
    c_ffi.flag("-Werror");
    c_ffi.compile("c-ffi");

    println!("cargo:rerun-if-changed=cpp/");
    let mut cpp_ffi = cc::Build::new();
    cpp_ffi.file("cpp/main.cpp");
    cpp_ffi.cpp(true);
    cpp_ffi.include("c");
    cpp_ffi.include("cpp");
    cpp_ffi.include(c_generated(None));
    cpp_ffi.include(cpp_generated(None));
    cpp_ffi.flag("-Wno-unused-parameter");
    cpp_ffi.flag("-Wno-missing-field-initializers");
    cpp_ffi.flag("-Werror");
    cpp_ffi.compile("cpp-ffi");
}
