//! Logical corpus aliases remain finite and inside the selected root.

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use super::{collect_vue_files, read_corpus_source};

fn write(root: &Path, relative: &str, bytes: &[u8]) {
    let file = root.join(relative);
    fs::create_dir_all(file.parent().expect("source directory")).expect("source directory");
    fs::write(file, bytes).expect("whole source");
}

#[test]
fn packaging_ancestor_alias_retains_source_once_per_finite_route() {
    let scratch = tempfile::tempdir().expect("corpus root");
    let root = scratch.path();
    let source = "\u{feff}<template>日本語 😀</template>\r\n";
    write(root, "components/App.vue", source.as_bytes());
    write(root, "packaging/deb/control", b"packaging metadata\n");
    write(root, "node_modules/ignored.vue", b"dependency source\n");
    symlink("../..", root.join("packaging/deb/root")).expect("Jellyfin ancestor alias");
    let mut files = Vec::new();
    collect_vue_files(root, &mut files);
    assert_eq!(
        files,
        [
            root.join("components/App.vue"),
            root.join("packaging/deb/root/components/App.vue"),
        ]
    );
    for file in &files {
        assert_eq!(read_corpus_source(file).as_bytes(), source.as_bytes());
    }
    assert_eq!(
        fs::read(root.join("components/App.vue")).expect("original source"),
        source.as_bytes()
    );
    assert_eq!(
        fs::read_link(root.join("packaging/deb/root")).expect("original link"),
        Path::new("../..")
    );
}

#[test]
fn separate_in_root_aliases_keep_each_original_logical_file() {
    let scratch = tempfile::tempdir().expect("corpus root");
    let root = scratch.path();
    let source = b"<template>complete source</template>\n";
    write(root, "source/App.vue", source);
    symlink("source", root.join("alias-a")).expect("first source alias");
    symlink("source", root.join("alias-b")).expect("second source alias");
    symlink("source/App.vue", root.join("linked.vue")).expect("source file alias");
    let mut files = Vec::new();
    collect_vue_files(root, &mut files);
    assert_eq!(
        files,
        [
            "alias-a/App.vue",
            "alias-b/App.vue",
            "linked.vue",
            "source/App.vue"
        ]
        .map(|file| root.join(file))
    );
    for file in files {
        assert_eq!(read_corpus_source(&file).as_bytes(), source);
    }
}

#[test]
fn external_directory_and_file_aliases_do_not_import_unpinned_sources() {
    let scratch = tempfile::tempdir().expect("corpus parent");
    let root = scratch.path().join("corpus");
    let outside = scratch.path().join("corpus-neighbor");
    write(&root, "retained.vue", b"<template>retained</template>\n");
    write(
        &outside,
        "external.vue",
        b"<template>outside source</template>\n",
    );
    symlink(&outside, root.join("outside-directory")).expect("external directory alias");
    symlink(outside.join("external.vue"), root.join("outside-file.vue"))
        .expect("external source alias");
    let mut files = Vec::new();
    collect_vue_files(&root, &mut files);
    assert_eq!(files, [root.join("retained.vue")]);
    assert_eq!(
        read_corpus_source(&files[0]).as_bytes(),
        b"<template>retained</template>\n"
    );
    assert_eq!(
        fs::read(outside.join("external.vue")).expect("external source unchanged"),
        b"<template>outside source</template>\n"
    );
}
