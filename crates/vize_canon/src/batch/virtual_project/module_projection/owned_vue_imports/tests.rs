//! Native Actions exercise the same owned graph, complete packets and lifecycle.

use std::fs;
use std::path::Path;

use vize_carton::cstr;

use super::super::super::VirtualProject;

const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/owned-vue-import-projection/Child.vue.txt"
));
const PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/owned-vue-import-projection/Parent.vue.txt"
));

fn project(root: &Path, options: &str) -> VirtualProject {
    fs::write(root.join("tsconfig.json"), options).unwrap();
    let mut project = VirtualProject::new(root).unwrap();
    project.scope_batch_namespace(&root.join("cache"));
    project
}

fn register(project: &mut VirtualProject, name: &str, source: &str) {
    let path = project.project_root().join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, source).unwrap();
    project.register_path(&path).unwrap();
}

fn packet(project: &VirtualProject, name: &str) -> serde_json::Value {
    let file = project
        .find_by_original(&project.project_root().join(name))
        .unwrap();
    serde_json::json!({"code": file.content,
        "map": cstr!("{:?}", file.source_map),
        "original": file.original_path, "virtual": file.virtual_path,
        "authored": project.original_contents[&file.virtual_path],
        "pristine": project.pre_rewrite_code.get(&file.virtual_path)})
}

#[test]
fn owned_relative_edge_changes_only_extension_and_preserves_every_map_offset() {
    let root = tempfile::tempdir().unwrap();
    let mut project = project(
        root.path(),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler"}}"#,
    );
    register(&mut project, "Child.vue", CHILD);
    register(&mut project, "Parent.vue", PARENT);
    let before = packet(&project, "Parent.vue");

    project.finalize_module_imports();
    let after = packet(&project, "Parent.vue");
    let mut expected = before.clone();
    expected["code"] = before["code"]
        .as_str()
        .unwrap()
        .replace("'./Child.vue.ts'", "'./Child.vue.js'")
        .into();
    assert_eq!(after, expected);
    assert_ne!(after["code"], before["code"]);
    project.finalize_module_imports();
    assert_eq!(packet(&project, "Parent.vue"), after);
    project.materialize().unwrap();
    let path = project
        .find_by_original(&project.project_root().join("Parent.vue"))
        .unwrap()
        .virtual_path
        .clone();
    assert_eq!(
        fs::read_to_string(path).unwrap(),
        after["code"].as_str().unwrap()
    );
    assert_eq!(
        fs::read_to_string(root.path().join("Parent.vue")).unwrap(),
        PARENT
    );
}

#[test]
fn authored_ts_alias_package_and_bare_star_packets_keep_existing_authority() {
    let root = tempfile::tempdir().unwrap();
    let mut project = project(
        root.path(),
        r#"{"compilerOptions":{"moduleResolution":"bundler","paths":{"@/*":["./*"]}}}"#,
    );
    register(&mut project, "Child.vue", CHILD);
    for (name, source) in [
        (
            "Authored.vue",
            "<script setup lang=\"ts\">import C from './Child.vue.ts';</script><template />",
        ),
        (
            "Alias.vue",
            "<script setup lang=\"ts\">import C from '@/Child.vue';</script><template />",
        ),
        (
            "Package.vue",
            "<script setup lang=\"ts\">import C from 'package/Child.vue';</script><template />",
        ),
        (
            "Star.vue",
            "<script lang=\"ts\">export * from './Child.vue';</script><template />",
        ),
    ] {
        register(&mut project, name, source);
        project.finalize_declaration_imports();
        let before = packet(&project, name);
        project.finalize_module_imports();
        assert_eq!(packet(&project, name), before, "{name}");
    }
}

#[test]
fn type_only_and_named_export_edges_use_the_same_registered_target() {
    let root = tempfile::tempdir().unwrap();
    let mut project = project(
        root.path(),
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    register(&mut project, "Child.vue", CHILD);
    register(
        &mut project,
        "Types.vue",
        "<script lang=\"ts\">import type C from './Child.vue'; export { default as Child } from './Child.vue';</script><template />",
    );
    let before = packet(&project, "Types.vue");
    project.finalize_module_imports();
    let mut expected = before.clone();
    expected["code"] = before["code"]
        .as_str()
        .unwrap()
        .replace("'./Child.vue.ts'", "'./Child.vue.js'")
        .into();
    assert_eq!(packet(&project, "Types.vue"), expected);
    assert_eq!(
        expected["code"]
            .as_str()
            .unwrap()
            .matches("'./Child.vue.js'")
            .count(),
        2
    );
}

#[test]
fn missing_foreign_tsx_and_suffix_targets_are_refused_without_packet_mutation() {
    for options in [
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
        r#"{"compilerOptions":{"moduleResolution":"bundler","moduleSuffixes":[".native",""]}}"#,
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut project = project(root.path(), options);
        register(&mut project, "Parent.vue", PARENT);
        let missing = packet(&project, "Parent.vue");
        project.finalize_module_imports();
        assert_eq!(packet(&project, "Parent.vue"), missing);
        register(&mut project, "Child.vue", CHILD);
        register(&mut project, "Parent.vue", PARENT);
        let baseline = packet(&project, "Parent.vue");
        let target = project
            .find_by_original(&root.path().join("Child.vue"))
            .unwrap()
            .virtual_path
            .clone();
        let original = project.virtual_files[&target].original_path.clone();
        project
            .virtual_files
            .get_mut(&target)
            .unwrap()
            .original_path = root.path().join("Foreign.vue");
        project.finalize_module_imports();
        assert_eq!(packet(&project, "Parent.vue"), baseline);
        project
            .virtual_files
            .get_mut(&target)
            .unwrap()
            .original_path = original;
        register(
            &mut project,
            "Child.vue",
            "<script setup lang=\"tsx\">const node = <div />;</script><template />",
        );
        project.finalize_module_imports();
        assert_eq!(packet(&project, "Parent.vue"), baseline);
        if options.contains("moduleSuffixes") {
            register(&mut project, "Child.vue", CHILD);
            project.finalize_module_imports();
            assert_eq!(packet(&project, "Parent.vue"), baseline);
        }
    }
}

#[test]
fn target_shape_remove_readd_and_second_project_never_reuse_previous_resolution() {
    let root = tempfile::tempdir().unwrap();
    let mut project = project(
        root.path(),
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    register(&mut project, "Child.vue", CHILD);
    register(&mut project, "Parent.vue", PARENT);
    project.finalize_module_imports();
    let admitted = packet(&project, "Parent.vue");
    let target = project.project_root().join("Child.vue");
    project.remove_registered_source(&target);
    fs::remove_file(&target).unwrap();
    project.finalize_module_imports();
    let removed = packet(&project, "Parent.vue");
    let mut missing_reference = self::project(
        root.path(),
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    register(&mut missing_reference, "Parent.vue", PARENT);
    missing_reference.finalize_module_imports();
    assert_eq!(removed, packet(&missing_reference, "Parent.vue"));
    register(&mut project, "Child.vue", CHILD);
    project.finalize_module_imports();
    assert_eq!(packet(&project, "Parent.vue"), admitted);
    let other = tempfile::tempdir().unwrap();
    let mut second = self::project(
        other.path(),
        r#"{"compilerOptions":{"moduleResolution":"bundler"}}"#,
    );
    register(&mut second, "Parent.vue", PARENT);
    let second_before = packet(&second, "Parent.vue");
    second.finalize_module_imports();
    assert_eq!(packet(&second, "Parent.vue"), second_before);
    assert_eq!(packet(&project, "Parent.vue"), admitted);
}

mod native;
