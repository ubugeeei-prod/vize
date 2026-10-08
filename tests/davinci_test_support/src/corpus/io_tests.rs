//! Corpus IO failures retain their complete path and original OS diagnostic.

use std::fs;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};

use vize_l0::{CompactString, cstr};

use super::{collect_vue_files, corpus_entry_path, read_corpus_source};

#[expect(
    clippy::disallowed_types,
    reason = "the standard panic machinery owns a std String; inspect the complete failure payload"
)]
fn failure_message(action: impl FnOnce()) -> CompactString {
    let payload = catch_unwind(AssertUnwindSafe(action)).expect_err("corpus IO must fail closed");
    payload
        .downcast_ref::<std::string::String>()
        .expect("formatted corpus IO failure")
        .as_str()
        .into()
}

fn expected_failure(operation: &str, path: &std::path::Path, error: &io::Error) -> CompactString {
    cstr!(
        "corpus IO failed: {operation} {}: {error} (kind={:?}, raw_os_error={:?})",
        path.display(),
        error.kind(),
        error.raw_os_error(),
    )
}

#[test]
fn corpus_collection_preserves_sorted_whole_vector_and_exclusions() {
    let scratch = tempfile::tempdir().expect("corpus root");
    let root = scratch.path();
    for directory in ["a", "z", "node_modules", "_git-worktrees", "documentation"] {
        fs::create_dir(root.join(directory)).expect("corpus directory");
    }
    for file in [
        "root.vue",
        "a/first.vue",
        "z/last.vue",
        "node_modules/ignored.vue",
        "_git-worktrees/ignored.vue",
        "documentation/notes.txt",
        "source.ts",
    ] {
        fs::write(root.join(file), b"<template>whole source</template>\r\n")
            .expect("corpus source");
    }
    let mut files = Vec::new();
    collect_vue_files(root, &mut files);
    assert_eq!(
        files,
        ["a/first.vue", "root.vue", "z/last.vue"].map(|path| root.join(path))
    );
}

#[test]
fn missing_corpus_directory_fails_with_whole_path_and_os_error() {
    let scratch = tempfile::tempdir().expect("corpus root");
    let root = scratch.path().join("missing directory 日本語");
    let error = fs::read_dir(&root).expect_err("directory is missing");
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(error.raw_os_error().is_some());
    let mut files = Vec::new();
    let message = failure_message(|| collect_vue_files(&root, &mut files));
    assert_eq!(message, expected_failure("read directory", &root, &error));
    assert!(files.is_empty());
}

#[test]
fn directory_entry_error_fails_with_parent_path_and_original_os_error() {
    let root = std::path::Path::new("corpus directory with spaces/日本語");
    let error = io::Error::from_raw_os_error(13);
    let expected = expected_failure("read directory entry under", root, &error);
    let message = failure_message(|| {
        corpus_entry_path(root, Err(error));
    });
    assert_eq!(message, expected);
}

#[test]
fn removed_selected_source_fails_instead_of_omitting_corpus_evidence() {
    let scratch = tempfile::tempdir().expect("corpus root");
    let path = scratch.path().join("selected 日本語.vue");
    fs::write(&path, b"<template>retained selected source</template>")
        .expect("selected corpus source");
    let mut files = Vec::new();
    collect_vue_files(scratch.path(), &mut files);
    assert_eq!(files, [path.clone()]);
    fs::remove_file(&path).expect("remove selected corpus source");
    let error = fs::read_to_string(&path).expect_err("selected source was removed");
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(error.raw_os_error().is_some());
    let message = failure_message(|| {
        read_corpus_source(&path);
    });
    assert_eq!(message, expected_failure("read source", &path, &error));
}

#[test]
fn invalid_utf8_selected_source_fails_with_original_error_kind() {
    let scratch = tempfile::tempdir().expect("corpus root");
    let path = scratch.path().join("invalid UTF-8.vue");
    fs::write(&path, b"<template>\xff</template>").expect("invalid UTF-8 source");
    let error = fs::read_to_string(&path).expect_err("invalid UTF-8 must be refused");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(error.raw_os_error(), None);
    let message = failure_message(|| {
        read_corpus_source(&path);
    });
    assert_eq!(message, expected_failure("read source", &path, &error));
}

#[test]
fn readable_selected_source_retains_every_utf8_byte() {
    let scratch = tempfile::tempdir().expect("corpus root");
    let path = scratch.path().join("valid 日本語.vue");
    let source =
        "\u{feff}<script setup>const value = '😀';</script>\r\n<template>日本語</template>\r\n";
    fs::write(&path, source.as_bytes()).expect("whole UTF-8 source");
    assert_eq!(read_corpus_source(&path).as_bytes(), source.as_bytes());
    assert_eq!(fs::read(&path).expect("original source"), source.as_bytes());
}
