//! Whole Options projections preserve authored imports without admitting setup-only JS.
use super::*;

fn configured_project(
    root: &std::path::Path,
    options: bool,
    legacy: bool,
    dialect: vize_carton::config::VueVersion,
    allowed: bool,
) -> VirtualProject {
    let config =
        vize_carton::cstr!("{{\"compilerOptions\":{{\"allowJs\":{allowed},\"checkJs\":false}}}}");
    std::fs::write(root.join("tsconfig.json"), config.as_str()).unwrap();
    let mut project = VirtualProject::new(root).unwrap();
    project.set_options_api(options);
    project.set_legacy_vue2(legacy);
    project.set_dialect(dialect);
    project
}

#[test]
fn options_projection_preserves_plain_and_mixed_runtime_prop_imports() {
    let child = include_str!(
        "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/OptionsChild.vue.txt"
    );
    let wrong = include_str!(
        "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/OptionsParent.vue.txt"
    );
    let repaired = wrong.replace("'wrong'", "1");
    let mixed = child.replace(
        "<template>",
        "<script setup>const local = 1;</script>\n<template>",
    );
    for (options, legacy, dialect) in [
        (true, false, vize_carton::config::VueVersion::V3),
        (false, true, vize_carton::config::VueVersion::V3),
        (false, false, vize_carton::config::VueVersion::V2),
        (false, false, vize_carton::config::VueVersion::V2_7),
    ] {
        for source in [child, mixed.as_str()] {
            let directory = tempfile::tempdir().unwrap();
            let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
            let child_path = root.join("Child.vue");
            let parent_path = root.join("Parent.vue");
            std::fs::write(&child_path, source).unwrap();
            let mut reference = configured_project(&root, options, legacy, dialect, true);
            reference
                .register_path_with_content(&child_path, source)
                .unwrap();
            let mut expected = Vec::new();
            for parent in [wrong, repaired.as_str()] {
                std::fs::write(&parent_path, parent).unwrap();
                reference
                    .register_path_with_content(&parent_path, parent)
                    .unwrap();
                reference.finalize_module_imports();
                expected.push((
                    vize_carton::cstr!("{:#?}", reference.find_by_original(&child_path).unwrap()),
                    vize_carton::cstr!("{:#?}", reference.find_by_original(&parent_path).unwrap()),
                ));
            }
            let mut project = configured_project(&root, options, legacy, dialect, false);
            project
                .register_path_with_content(&child_path, source)
                .unwrap();
            for (parent, (expected_child, expected_parent)) in
                [wrong, repaired.as_str()].into_iter().zip(expected)
            {
                std::fs::write(&parent_path, parent).unwrap();
                project
                    .register_path_with_content(&parent_path, parent)
                    .unwrap();
                project.finalize_module_imports();
                let actual_child = project.find_by_original(&child_path).unwrap();
                assert_eq!(
                    actual_child.virtual_path,
                    project.virtual_root.join("Child.vue.ts")
                );
                // Debug includes the complete content, SFC/projection/import mappings,
                // authored path and materialized path; nothing is selected or removed.
                assert_eq!(vize_carton::cstr!("{actual_child:#?}"), expected_child);
                assert_eq!(
                    vize_carton::cstr!("{:#?}", project.find_by_original(&parent_path).unwrap()),
                    expected_parent
                );
                assert_eq!(
                    project.unchecked_javascript_files,
                    std::iter::once(actual_child.virtual_path.clone())
                        .collect::<vize_carton::FxHashSet<_>>()
                );
            }
        }
    }
}

#[test]
fn options_mode_keeps_setup_only_javascript_denied() {
    for (options, legacy, dialect) in [
        (true, false, vize_carton::config::VueVersion::V3),
        (false, true, vize_carton::config::VueVersion::V3),
        (false, false, vize_carton::config::VueVersion::V2),
        (false, false, vize_carton::config::VueVersion::V2_7),
        (false, false, vize_carton::config::VueVersion::V3),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
        let mut project = configured_project(&root, options, legacy, dialect, false);
        let path = root.join("Child.vue");
        let mut sources = vec![
            "<script setup>defineProps({ count: Number });</script>",
            "<script setup>// @ts-check\ndefineProps({ count: Number });</script>",
        ];
        if !options && !legacy && dialect == vize_carton::config::VueVersion::V3 {
            sources.push(include_str!(
                "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/OptionsChild.vue.txt"
            ));
        }
        for source in sources {
            std::fs::write(&path, source).unwrap();
            project.register_path_with_content(&path, source).unwrap();
            assert_eq!(
                project.find_by_original(&path).unwrap().virtual_path,
                project
                    .virtual_root
                    .join("Child.vue.__vize_javascript__.ts")
            );
        }
    }
}
