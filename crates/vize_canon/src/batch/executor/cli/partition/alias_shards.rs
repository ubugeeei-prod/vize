//! Partition plans for tsconfig `paths` aliases (#10360). No checker process.

use std::{fs, path::PathBuf};

use super::super::{partition_virtual_files, shard_count};
use crate::batch::VirtualProject;

const REPRO_FILES: usize = 192;
const OPTIONS: &str = r#""target":"ES2022","module":"ESNext","moduleResolution":"Bundler","strict":true,"noEmit":true,"skipLibCheck":true,"jsx":"preserve""#;

struct Built {
    _keep: tempfile::TempDir,
    project: VirtualProject,
    vue_paths: Vec<PathBuf>,
    local_paths: Vec<PathBuf>,
}

fn write_tsconfig(root: &std::path::Path, extra: &str) {
    fs::write(
        root.join("tsconfig.json"),
        format!(r#"{{"compilerOptions":{{{OPTIONS}{extra}}},"include":["src"]}}"#),
    )
    .unwrap();
}

fn vue_source(specifier: &str, import_vue: bool) -> String {
    let vue = if import_vue {
        "import { ref } from \"vue\";\n"
    } else {
        ""
    };
    format!(
        "<script setup lang=\"ts\">\nimport {{ n }} from \"{specifier}\";\n{vue}const v = n;\n</script>\n<template><p>{{{{ v }}}}</p></template>\n"
    )
}

fn build_groups(specifier: fn(usize) -> String, extra: &str, import_vue: bool) -> Built {
    let keep = tempfile::tempdir().unwrap();
    let root = keep.path().canonicalize().unwrap();
    let mut vue_paths = Vec::with_capacity(REPRO_FILES);
    let mut local_paths = Vec::with_capacity(REPRO_FILES);
    for index in 0..REPRO_FILES {
        let dir = root.join(format!("src/g{index:03}"));
        fs::create_dir_all(&dir).unwrap();
        let local = dir.join("local.ts");
        fs::write(&local, format!("export const n = {index};\n")).unwrap();
        let vue = dir.join("A.vue");
        fs::write(&vue, vue_source(&specifier(index), import_vue)).unwrap();
        vue_paths.push(vue);
        local_paths.push(local);
    }
    write_tsconfig(&root, extra);
    let mut paths = vue_paths.clone();
    paths.extend(local_paths.iter().cloned());
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_paths(&paths).unwrap();
    Built {
        _keep: keep,
        project,
        vue_paths,
        local_paths,
    }
}

fn alias_specifier(index: usize) -> String {
    format!("@/g{index:03}/local")
}

fn bare_specifier(index: usize) -> String {
    format!("g{index:03}/local")
}

fn relative_specifier(_index: usize) -> String {
    "./local".to_owned()
}

fn twelve_core_servers() -> usize {
    let servers = shard_count(12, 1, REPRO_FILES);
    assert_eq!(servers, 3, "192 SFCs on 12 cores request 3 shards");
    servers
}

fn assert_repro_splits(built: &Built) {
    let plan = partition_virtual_files(&built.project, twelve_core_servers());
    assert!(
        plan.shards.len() > 1,
        "distinct modules collapsed onto one checker (shards={})",
        plan.shards.len()
    );
    assert_eq!(plan.shards.len(), 3);
    let vue = plan.owners.get(&built.vue_paths[0]).copied();
    let local = plan.owners.get(&built.local_paths[0]).copied();
    assert!(vue.is_some(), "vue file was not partitioned");
    assert_eq!(vue, local, "an alias target must stay with its importer");
    assert!(
        built
            .vue_paths
            .iter()
            .any(|path| plan.owners.get(path).copied() != vue),
        "every Vue file landed in the same shard"
    );
}

#[test]
fn distinct_alias_modules_shard_on_twelve_cores() {
    let built = build_groups(alias_specifier, r#","paths":{"@/*":["./src/*"]}"#, false);
    let content = &built
        .project
        .find_by_original(&built.vue_paths[0])
        .unwrap()
        .content;
    assert!(
        content.contains("@/g000/local"),
        "generated module lost the alias specifier:\n{content}"
    );
    assert_repro_splits(&built);
}

#[test]
fn relative_imports_still_shard_on_twelve_cores() {
    let built = build_groups(relative_specifier, "", false);
    assert_repro_splits(&built);
}

#[test]
fn base_url_without_paths_still_shards_on_twelve_cores() {
    let relative = build_groups(relative_specifier, r#","baseUrl":"."#, false);
    assert_repro_splits(&relative);
    // A synthesized `"*"` for baseUrl must not couple every bare specifier.
    let bare = build_groups(bare_specifier, r#","baseUrl":"."#, true);
    let plan = partition_virtual_files(&bare.project, twelve_core_servers());
    assert!(
        plan.shards.len() > 1,
        "baseUrl collapsed bare imports (shards={})",
        plan.shards.len()
    );
}

#[test]
fn empty_star_prefix_does_not_couple_every_importer() {
    let built = build_groups(bare_specifier, r#","paths":{"*":["./src/*"]}"#, true);
    assert_repro_splits(&built);
}

fn dominant_sources(paths: &[(String, String)], extra: &str) {
    let keep = tempfile::tempdir().unwrap();
    let root = keep.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    let mut files = Vec::new();
    for (name, source) in paths {
        let path = root.join("src").join(name);
        fs::write(&path, source).unwrap();
        files.push(path);
    }
    write_tsconfig(&root, extra);
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_paths(&files).unwrap();
    assert!(
        partition_virtual_files(&project, 2).shards.is_empty(),
        "a real alias component must stay on one checker"
    );
}

#[test]
fn one_alias_module_stays_a_dominant_component() {
    let mut files = Vec::new();
    for index in 0..8 {
        files.push((
            format!("A{index}.vue"),
            "<script setup lang=\"ts\">import { n } from '@/shared'; const v = n;</script><template><p>{{ v }}</p></template>".to_owned(),
        ));
    }
    files.push(("shared.ts".to_owned(), "export const n = 1;\n".to_owned()));
    files.push((
        "Isolated.vue".to_owned(),
        "<script setup lang=\"ts\">const n = 1</script><template>{{ n }}</template>".to_owned(),
    ));
    dominant_sources(&files, r#","paths":{"@/*":["./src/*"]}"#);
}

#[test]
fn alias_cycle_stays_a_dominant_component() {
    let mut files = Vec::new();
    for index in 0..8 {
        let next = (index + 1) % 8;
        files.push((
            format!("C{index}.vue"),
            format!(
                "<script setup lang=\"ts\">import Next from '@/C{next}.vue'\nvoid Next</script><template><p /></template>"
            ),
        ));
    }
    files.push((
        "Isolated.vue".to_owned(),
        "<script setup lang=\"ts\">const n = 1</script><template>{{ n }}</template>".to_owned(),
    ));
    dominant_sources(&files, r#","paths":{"@/*":["./src/*"]}"#);
}

#[test]
fn same_missing_alias_stays_a_dominant_component() {
    let mut files = Vec::new();
    for index in 0..8 {
        files.push((
            format!("M{index}.vue"),
            "<script setup lang=\"ts\">import { n } from '@/missing'; const v = n;</script><template><p>{{ v }}</p></template>".to_owned(),
        ));
    }
    files.push((
        "Isolated.vue".to_owned(),
        "<script setup lang=\"ts\">const n = 1</script><template>{{ n }}</template>".to_owned(),
    ));
    dominant_sources(&files, r#","paths":{"@/*":["./src/*"]}"#);
}

#[test]
fn node_modules_alias_does_not_couple_importers() {
    let keep = tempfile::tempdir().unwrap();
    let root = keep.path().canonicalize().unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    let mut files = Vec::new();
    for index in 0..4 {
        let path = root.join(format!("src/N{index}.vue"));
        fs::write(
            &path,
            "<script setup lang=\"ts\">import { button } from '@lib/button'; const v = button;</script><template><p>{{ v }}</p></template>",
        )
        .unwrap();
        files.push(path);
    }
    write_tsconfig(&root, r#","paths":{"@lib/*":["./node_modules/@lib/*"]}"#);
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_paths(&files).unwrap();
    let plan = partition_virtual_files(&project, 2);
    assert!(
        plan.shards.len() > 1,
        "a node_modules paths target coupled importers (shards={})",
        plan.shards.len()
    );
}
