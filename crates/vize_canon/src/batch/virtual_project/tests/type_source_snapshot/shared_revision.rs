//! One loaded revision registers its sources through one dependency snapshot.

use std::{fs, path::PathBuf};

use super::{VirtualProject, assert_equivalent_projects, uncached_project};

#[test]
fn loaded_revision_shares_one_frozen_dependency_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    let api = dir.path().join("api.ts");
    fs::write(&api, "export interface Shared { label: string }").unwrap();
    let sfc = |name: &str| {
        let path = src.join(name);
        fs::write(
            &path,
            r#"<script setup lang="ts">
import type { Shared } from '../../api'
defineProps<Shared>()
</script>
<template><div>{{ label }}</div></template>"#,
        )
        .unwrap();
        path
    };
    let paths = [sfc("First.vue"), sfc("Second.vue")];
    let sources: Vec<(PathBuf, std::string::String)> = paths
        .iter()
        .map(|path| (path.clone(), fs::read_to_string(path).unwrap()))
        .collect();

    let mut shared = VirtualProject::new(&root).unwrap();
    shared
        .register_paths_with_content(
            sources
                .iter()
                .map(|(path, source)| (path.as_path(), source.as_str())),
        )
        .unwrap();
    assert_equivalent_projects(&shared, &uncached_project(&root, &paths));

    // The dependency changes while the revision is being registered. Later
    // importers must still read the text the first importer captured; a
    // per-file snapshot would observe the rewritten module instead.
    let mut frozen = VirtualProject::new(&root).unwrap();
    frozen
        .register_paths_with_content(sources.iter().enumerate().map(|(index, (path, source))| {
            if index == 1 {
                fs::write(&api, "export interface Shared { renamed: string }").unwrap();
            }
            (path.as_path(), source.as_str())
        }))
        .unwrap();
    for path in &paths {
        let code = &frozen.find_by_original(path).unwrap().content;
        assert!(
            code.contains("const label = props[\"label\"]"),
            "{} lost the captured dependency text",
            path.display()
        );
        assert!(!code.contains("renamed"), "{}", path.display());
    }
}
