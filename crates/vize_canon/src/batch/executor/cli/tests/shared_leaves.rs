use std::{fs, path::Path};

use super::super::partition_virtual_files;
use crate::batch::VirtualProject;

fn project(root: &Path, shared: &str, additional: &[(&str, &str)]) -> VirtualProject {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("shared.ts"), shared).unwrap();
    for index in 0..4 {
        fs::write(
            root.join(format!("Comp{index}.vue")),
            "<script setup lang=\"ts\">import { value } from './shared'; const n = value;</script><template><div>{{ n }}</div></template>",
        )
        .unwrap();
    }
    for &(file, content) in additional {
        fs::write(root.join(file), content).unwrap();
    }
    let mut project = VirtualProject::new(root).unwrap();
    let mut paths: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    project.register_paths(&paths).unwrap();
    project
}

#[test]
fn cheap_multiply_imported_script_is_shared_without_owning_diagnostics() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        "export const value: number = 'leaf-error';",
        &[],
    );
    let plan = partition_virtual_files(&project, 2);
    assert_eq!(plan.shards.len(), 2);
    assert_eq!(plan.owners.len(), 4);
    assert!(!plan.owners.contains_key(&dir.path().join("shared.ts")));
    for shard in plan.shards {
        assert_eq!(
            shard
                .iter()
                .filter(|path| path.ends_with("shared.ts"))
                .count(),
            1,
            "shared leaf diagnostics are accepted from every shard and deduplicated"
        );
    }
    assert!(partition_virtual_files(&project, 1).shards.is_empty());
}

#[test]
fn transitive_and_oversized_shared_scripts_keep_their_importers_connected() {
    for (shared, additional) in [
        (
            "export { value } from './other';".to_owned(),
            vec![("other.ts", "export const value = 1;")],
        ),
        (
            format!("export const value = 1; /*{}*/", "x".repeat(1024)),
            vec![],
        ),
        (
            "export const value = require /* gap */ ('./other');".to_owned(),
            vec![("other.ts", "export const value = 1;")],
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let project = project(dir.path(), &shared, &additional);
        assert!(partition_virtual_files(&project, 2).shards.is_empty());
    }
}

#[test]
fn global_namespace_and_implicit_script_roots_disable_new_leaf_sharing() {
    for source in [
        "namespace GlobalModel { export type Value = number; }",
        "/* export const marker = 1; */ const globalValue = 1;",
        "export {}; void import(`missing);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let project = project(
            dir.path(),
            "export const value = 1;",
            &[("globals.ts", source)],
        );
        assert!(
            partition_virtual_files(&project, 2).shards.is_empty(),
            "global script visibility cannot change when a dominant graph is split"
        );
    }
}

#[test]
fn module_global_augmentations_remain_shared_with_cheap_leaves() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(
        dir.path(),
        "export const value = 1;",
        &[(
            "augment.ts",
            "export {}; declare global { interface Window { leaf: number } }",
        )],
    );
    let plan = partition_virtual_files(&project, 2);
    assert_eq!(plan.shards.len(), 2);
    assert!(!plan.owners.contains_key(&dir.path().join("augment.ts")));
    for shard in plan.shards {
        assert!(shard.iter().any(|path| path.ends_with("augment.ts")));
        assert!(shard.iter().any(|path| path.ends_with("shared.ts")));
    }
}

#[test]
fn single_imports_stay_owned_and_vue_edges_stay_connected() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = project(dir.path(), "export const value = 1;", &[]);
    for index in 1..4 {
        fs::write(
            dir.path().join(format!("Comp{index}.vue")),
            "<script setup lang=\"ts\">const n = 1;</script><template>{{ n }}</template>",
        )
        .unwrap();
    }
    project
        .register_paths(&[
            dir.path().join("Comp1.vue"),
            dir.path().join("Comp2.vue"),
            dir.path().join("Comp3.vue"),
        ])
        .unwrap();
    let plan = partition_virtual_files(&project, 2);
    assert_eq!(
        plan.owners.get(&dir.path().join("Comp0.vue")),
        plan.owners.get(&dir.path().join("shared.ts"))
    );
    assert!(plan.owners.contains_key(&dir.path().join("shared.ts")));

    fs::write(
        dir.path().join("Comp1.vue"),
        "<script setup lang=\"ts\">import Comp from './Comp0.vue';</script><template><Comp /></template>",
    ).unwrap();
    project
        .register_path(&dir.path().join("Comp1.vue"))
        .unwrap();
    let plan = partition_virtual_files(&project, 2);
    assert_eq!(
        plan.owners.get(&dir.path().join("Comp0.vue")),
        plan.owners.get(&dir.path().join("Comp1.vue"))
    );
}

#[test]
fn template_and_uncertain_module_operands_keep_the_original_component_plan() {
    for load in [
        "void import(`leaf-types`);",
        "void import((`leaf-types`));",
        "void import(('leaf-types'));",
        "const name = 'types'; void import(`leaf-${name}`);",
        "const name = 'vue'; void import(name);",
        "type P = typeof import(`leaf-types`);",
        "const pkg = require('vue'); void pkg;",
        "const pkg = require?.('leaf-types'); void pkg;",
        "const pkg = require<string>('leaf-types'); void pkg;",
        "void import.source('leaf-types');",
        "import ^ 'leaf-types';",
        "import x 'leaf-types';",
        "void import?.('leaf-types');",
        "export ^ 'leaf-types';",
        "export * 'leaf-types';",
        "export * from ^ 'leaf-types';",
        "void import('vue');",
        "type P = typeof import('vue');",
        "void import(/* trivia */ 'leaf-types', { with: { type: 'json' } });",
        "import pkg = require /* trivia */ ('leaf-types'); void pkg;",
        "declare /* gap */ global { type LeafGlobal = number; }",
        "namespace Local { export const n = 1; }",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source = format!(
            "<script setup lang=\"ts\">import {{ value }} from './shared'; {load} const n = value;</script><template>{{{{ n }}}}</template>"
        );
        let project = project(
            dir.path(),
            "export const value = 1;",
            &[("Comp0.vue", &source)],
        );
        assert!(
            partition_virtual_files(&project, 2).shards.is_empty(),
            "an unproved module operand cannot hide dependencies in sibling programs: {load}"
        );
    }
}

#[test]
fn nested_vue_loads_cannot_reuse_the_root_helpers_package_context() {
    let dir = tempfile::tempdir().unwrap();
    let mut project = project(dir.path(), "export const value = 1;", &[]);
    let nested = dir.path().join("sub/Comp.vue");
    fs::create_dir(nested.parent().unwrap()).unwrap();
    fs::write(
        &nested,
        "<script setup lang=\"ts\">import { value } from '../shared'; import 'vue'; const n = value;</script><template>{{ n }}</template>",
    )
    .unwrap();
    project.register_path(&nested).unwrap();
    assert!(
        partition_virtual_files(&project, 2).shards.is_empty(),
        "a nearest nested Vue package can provide globals absent from the root helper"
    );
}
