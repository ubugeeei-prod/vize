//! Directory specifiers keep the same authored barrel graph as the native checker.

use super::{canonical_key, may_resolve_a_dependency, resolve_dependency};
use crate::batch::virtual_project::VirtualProject;
use std::fs;

#[test]
fn exact_directory_imports_register_both_barrels_and_their_vue_dependency() {
    let root = tempfile::tempdir().unwrap();
    let authored_root = root.path().canonicalize().unwrap();
    let source = authored_root.join("src");
    let nested = source.join("story");
    fs::create_dir_all(&nested).unwrap();
    let component = source.join("Input.vue");
    let parent = source.join("index.ts");
    let sibling = nested.join("index.ts");
    let story = nested.join("Story.vue");
    fs::write(
        &component,
        "<script setup lang=\"ts\">defineProps<{ value: number }>()</script>\n",
    )
    .unwrap();
    fs::write(&parent, "export { default as Input } from './Input.vue';\n").unwrap();
    fs::write(&sibling, "export { Input } from '..';\n").unwrap();
    fs::write(&story, "<script setup lang=\"ts\">import { Input } from '.';</script>\n<template><Input value=\"wrong\" /></template>\n").unwrap();

    assert_eq!(
        resolve_dependency(".", &nested, root.path(), &[]),
        Some(sibling.clone())
    );
    assert_eq!(
        resolve_dependency("..", &nested, root.path(), &[]),
        Some(nested.join("../index.ts"))
    );
    assert_eq!(
        resolve_dependency("..", &nested, root.path(), &[]).and_then(|path| canonical_key(&path)),
        Some(parent.clone())
    );
    let mut project = VirtualProject::new(root.path()).unwrap();
    project.register_path(&story).unwrap();
    project.register_reachable_dependencies().unwrap();
    let registered = project.registered_original_paths_sorted();
    let mut expected = vec![story, sibling, parent, component];
    expected.sort();
    assert_eq!(registered, expected);
}

#[test]
fn directory_prefilter_preserves_the_bare_package_fast_path() {
    for source in [
        "import { Input } from '.'",
        "export { Input } from \".\"",
        "import { Input } from '..'",
        "export { Input } from \"..\"",
    ] {
        assert!(may_resolve_a_dependency(source, &[], &[]), "{source}");
    }
    for source in [
        "import { ref } from 'vue'",
        "import { Input } from '.hidden'",
        "import { Input } from '..hidden'",
        "export const path = '..hidden'",
    ] {
        assert!(!may_resolve_a_dependency(source, &[], &[]), "{source}");
    }
}
