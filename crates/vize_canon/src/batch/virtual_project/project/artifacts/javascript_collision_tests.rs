//! A refused reserved-path collision must preserve its complete existing owner.

use super::*;

#[test]
fn denied_javascript_projection_collision_preserves_both_registration_orders() {
    for (language, extension) in [("", "ts"), (" lang=\"jsx\"", "tsx")] {
        for javascript_first in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
            let javascript = root.join("Child.vue");
            let authored =
                root.join(vize_carton::cstr!("Child.vue.__vize_javascript__.{extension}").as_str());
            let sfc = vize_carton::cstr!(
                "<script setup{language}>const index = 1;</script><template><slot name=\"actions\" :index=\"index\" /></template>"
            );
            std::fs::write(&javascript, sfc.as_str()).unwrap();
            std::fs::write(&authored, "export const owned = 1;\n").unwrap();
            let (first, second) = if javascript_first {
                (&javascript, &authored)
            } else {
                (&authored, &javascript)
            };
            let mut project = VirtualProject::new(&root).unwrap();
            project.register_path(first).unwrap();
            let before = vize_carton::cstr!("{:?}", project.virtual_files_sorted());
            let index = project.original_index.clone();
            let originals = project.original_contents.clone();
            let rebuilt = project.incremental_source_nodes_rebuilt;
            let candidates = project.incremental_materialized_candidates.clone();
            let error = project.register_path(second).unwrap_err();
            assert!(matches!(error, CorsaError::PathError { path } if path == *second));
            assert_eq!(
                vize_carton::cstr!("{:?}", project.virtual_files_sorted()),
                before
            );
            assert_eq!(project.original_index, index);
            assert_eq!(project.original_contents, originals);
            assert_eq!(project.incremental_source_nodes_rebuilt, rebuilt);
            assert_eq!(project.incremental_materialized_candidates, candidates);
            assert!(project.find_by_original(first).is_some());
            assert!(project.find_by_original(second).is_none());

            // A same-owner refresh is still admitted and cannot confer ownership
            // on the other authored path. Removing it releases only that owner.
            let changed = if javascript_first {
                sfc.replace("index = 1", "index = 2")
            } else {
                "export const owned = 2;\n".into()
            };
            project
                .register_path_with_content(first, changed.as_str())
                .unwrap();
            assert!(project.find_by_original(first).is_some());
            assert!(project.find_by_original(second).is_none());
            let removed = project.remove_registered_source(first);
            assert!(!removed.is_empty());
            assert!(project.find_by_original(first).is_none());
            project.register_path(second).unwrap();
            assert!(project.find_by_original(second).is_some());
            assert!(project.find_by_original(first).is_none());
        }
    }
}

#[test]
fn javascript_sfc_language_and_configuration_refresh_retire_only_the_old_projection() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let path = root.join("Child.vue");
    let config = root.join("tsconfig.json");
    let javascript = "<script setup>const index = 1;</script><template><slot name=\"actions\" :index=\"index\" /></template>";
    let typescript = javascript.replace("<script setup>", "<script setup lang=\"ts\">");
    std::fs::write(&config, r#"{"compilerOptions":{"allowJs":false}}"#).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project
        .register_path_with_content(&path, javascript)
        .unwrap();
    let denied = project
        .find_by_original(&path)
        .unwrap()
        .virtual_path
        .clone();
    assert!(super::super::super::module_projection::is_javascript_projection_path(&denied));

    project
        .register_path_with_content(&path, &typescript)
        .unwrap();
    let typed = project
        .find_by_original(&path)
        .unwrap()
        .virtual_path
        .clone();
    assert_ne!(typed, denied);
    assert!(!project.virtual_files.contains_key(&denied));
    assert!(!project.original_contents.contains_key(&denied));
    assert!(!project.pre_rewrite_code.contains_key(&denied));
    assert!(!project.unchecked_javascript_files.contains(&typed));

    project
        .register_path_with_content(&path, javascript)
        .unwrap();
    assert_eq!(
        project.find_by_original(&path).unwrap().virtual_path,
        denied
    );
    assert!(!project.virtual_files.contains_key(&typed));
    std::fs::write(
        &config,
        r#"{"compilerOptions":{"allowJs":true,"checkJs":false}}"#,
    )
    .unwrap();
    project.refresh_compiler_configuration();
    project
        .register_path_with_content(&path, javascript)
        .unwrap();
    assert_eq!(project.find_by_original(&path).unwrap().virtual_path, typed);
    assert!(!project.virtual_files.contains_key(&denied));
    assert!(project.unchecked_javascript_files.contains(&typed));
    assert_eq!(project.original_index.len(), 1);
    assert_eq!(project.source_artifacts.len(), 1);
}
