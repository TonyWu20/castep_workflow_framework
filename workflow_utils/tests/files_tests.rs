use tempfile::tempdir;
use workflow_utils::{copy_file, create_dir, exists, read_file, remove_dir, write_file};

#[test]
fn test_read_write_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.txt");
    write_file(&path, "hello").unwrap();
    assert_eq!(read_file(&path).unwrap(), "hello");
}

#[test]
fn test_write_creates_parent_dirs() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("a/b/c/test.txt");
    write_file(&path, "data").unwrap();
    assert_eq!(read_file(&path).unwrap(), "data");
}

#[test]
fn test_copy_file() {
    let dir = tempdir().unwrap();
    let src = dir.path().join("src.txt");
    let dst = dir.path().join("sub/dst.txt");
    write_file(&src, "content").unwrap();
    copy_file(&src, &dst).unwrap();
    assert_eq!(read_file(&dst).unwrap(), "content");
}

#[test]
fn test_create_remove_dir() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("mydir");
    create_dir(&path).unwrap();
    assert!(exists(&path));
    remove_dir(&path).unwrap();
    assert!(!exists(&path));
}

#[test]
fn test_file_not_found() {
    assert!(read_file("/nonexistent/path/file.txt").is_err());
}
