//! Tests to validate cyclical imports in includes.
/// This module containes tests that require serial execution.
///
/// Without serial execution, writing the contents in files could cause race conditions.
use idlc_ast_passes::idl_store::IDLStore;
use idlc_ast_passes::*;

use serial_test::serial;

fn verify(
    test_dir: &'static str,
    file_name_a: &'static str,
    include_paths: &'static str,
) -> Result<Vec<String>, idlc_ast_passes::Error> {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let full_path = format!(r#"{}/{}"#, "tests/test_data", test_dir);
    dir.push(full_path);

    std::env::set_current_dir(&dir).expect("Failed to change directory");

    let mut include_paths_vec: Vec<std::path::PathBuf> = vec![std::env::current_dir().unwrap()];

    if !include_paths.is_empty() {
        include_paths_vec.push(std::path::Path::new(include_paths).to_path_buf());
    }
    let mut store = IDLStore::with_includes(&include_paths_vec);
    let input_file = std::path::Path::new(file_name_a)
        .canonicalize()
        .expect("Invalid input file.");

    let ast = store.get_or_insert(&input_file);
    store.run_pass(&ast)
}

fn verify_memory(
    file_name_a: &'static str,
    a_idl: &'static str,
    file_name_b: &'static str,
    b_idl: &'static str,
    file_name_c: &'static str,
    c_idl: &'static str,
) -> Result<Vec<String>, idlc_ast_passes::Error> {
    let mut store = IDLStore::new();
    let name = std::path::PathBuf::from(file_name_a);
    let node = idlc_ast::from_string(name.clone(), a_idl).unwrap();
    store.insert_canonical(&name, &node);

    let name2 = std::path::PathBuf::from(file_name_b);
    let node2 = idlc_ast::from_string(name2.clone(), b_idl).unwrap();
    store.insert_canonical(&name2, &node2);

    let name3 = std::path::PathBuf::from(file_name_c);
    let node3 = idlc_ast::from_string(name3.clone(), c_idl).unwrap();
    store.insert_canonical(&name3, &node3);

    let ast = store.get_ast(&name).unwrap();
    store.run_pass(&ast)
}

#[test]
#[serial]
fn cycle_without_include_paths() {
    assert!(matches!(
        verify("cycle_without_include_paths", "a.idl", ""),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
#[serial]
fn no_cycle_without_include_paths() {
    assert_eq!(
        verify("no_cycle_without_include_paths", "a.idl", "").unwrap(),
        [
            std::path::Path::new("c.idl")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            std::path::Path::new("b.idl")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap(),
            std::path::Path::new("a.idl")
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
        ]
    );
}

#[test]
#[serial]
fn cycle_with_include_paths() {
    assert!(matches!(
        verify("cycle_with_include_paths", "a.idl", "./test_inner"),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
fn cycle_in_memory() {
    assert!(matches!(
        verify_memory(
            "test_1.idl",
            r#"
        include "test_2.idl""#,
            "test_2.idl",
            r#"
        include "test_3.idl""#,
            "test_3.idl",
            r#"
        include "test_1.idl""#,
        ),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
#[serial]
fn cycle_with_duplicate_file_with_include_paths() {
    assert!(matches!(
        verify(
            "cycle_with_duplicate_file_with_include_paths",
            "a.idl",
            "./test_inner",
        ),
        Err(Error::CyclicalInclude(_))
    ));
}

#[test]
#[serial]
#[should_panic(expected = "File not found")]
fn invalid_file() {
    let _ = verify("", "invalid_file.idl", "");
}

#[test]
#[serial]
#[should_panic(expected = "cannot be found")]
fn invalid_trailing_path() {
    let _ = verify("", "invalid_trailing_path.idl", "");
}
