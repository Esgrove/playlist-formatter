use std::path::PathBuf;

/// Returns the path to the `tests/files` directory.
fn test_files_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("files")
}

#[test]
fn test_dir() {
    assert!(test_files_dir().is_dir());
}
