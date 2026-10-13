//! The reach sweep reads every original logical alias or fails with its full IO error.

#![cfg(unix)]

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;

use vize_l0::{CompactString, cstr};

use crate::{PROFILER_TEST_LOCK, sweep};

use super::parity::parity_failures;

fn refused_sweep(root: &Path, path: &Path) -> CompactString {
    let payload = catch_unwind(AssertUnwindSafe(|| sweep(root, &[path.to_path_buf()])))
        .err()
        .expect("unreadable selected input must fail the complete sweep");
    payload
        .downcast_ref::<String>()
        .expect("complete corpus IO failure payload")
        .as_str()
        .into()
}

fn complete_failure(path: &Path, error: &std::io::Error) -> CompactString {
    cstr!(
        "corpus IO failed: read source {}: {error} (kind={:?}, raw_os_error={:?})",
        path.display(),
        error.kind(),
        error.raw_os_error(),
    )
}

#[test]
fn original_alias_beyond_single_lookup_limit_is_measured_by_every_reach_shape() {
    let _guard = PROFILER_TEST_LOCK.lock().unwrap();
    let scratch = tempfile::tempdir().expect("original corpus root");
    let root = scratch.path();
    std::os::unix::fs::symlink(".", root.join("alias")).expect("original ancestor alias");
    let source = "\u{feff}<template><section>日本語 😀</section></template>\r\n";
    let physical = root.join("selected 日本語.vue");
    fs::write(&physical, source.as_bytes()).expect("complete original source");
    let mut logical = root.to_path_buf();
    for _ in 0..80 {
        logical.push("alias");
    }
    logical.push("selected 日本語.vue");
    let original_error = fs::read_to_string(&logical).expect_err("single lookup exceeds limit");
    assert_eq!(cstr!("{:?}", original_error.kind()), "FilesystemLoop");
    assert!(original_error.raw_os_error().is_some());
    assert_eq!(
        fs::read(&physical).expect("original bytes"),
        source.as_bytes()
    );

    let measured = sweep(root, &[logical]);
    assert_eq!(
        (
            measured.files,
            measured.unreadable,
            measured.parse_errors,
            measured.without_template,
        ),
        (1, 0, 0, 0)
    );
    assert_eq!(
        measured
            .tallies
            .iter()
            .map(|(name, (_, tally))| (*name, tally.templates))
            .collect::<Vec<_>>(),
        [
            ("dom_inline", 1),
            ("dom_module", 1),
            ("ssr", 1),
            ("vapor", 1)
        ]
    );
    assert_eq!(parity_failures(&measured), Vec::<String>::new());
}

#[test]
fn genuine_selected_source_cycle_refuses_with_complete_logical_path_and_os_error() {
    let scratch = tempfile::tempdir().expect("original corpus root");
    let path = scratch.path().join("cycle 日本語.vue");
    std::os::unix::fs::symlink(path.file_name().expect("source name"), &path)
        .expect("genuine source cycle");
    let error = fs::read_to_string(&path).expect_err("self-referential link is refused");
    assert_eq!(cstr!("{:?}", error.kind()), "FilesystemLoop");
    assert!(error.raw_os_error().is_some());
    assert_eq!(
        refused_sweep(scratch.path(), &path),
        complete_failure(&path, &error)
    );
}

#[test]
fn invalid_utf8_selected_source_refuses_with_complete_original_error() {
    let scratch = tempfile::tempdir().expect("original corpus root");
    let path = scratch.path().join("invalid 日本語.vue");
    fs::write(&path, b"<template>\xff</template>").expect("original invalid UTF-8 source");
    let error = fs::read_to_string(&path).expect_err("invalid UTF-8 is refused");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.raw_os_error(), None);
    assert_eq!(
        refused_sweep(scratch.path(), &path),
        complete_failure(&path, &error)
    );
}
