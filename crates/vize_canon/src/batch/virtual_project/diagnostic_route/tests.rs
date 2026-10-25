use super::super::VirtualProject;
use std::{fs, path::Path};
use vize_carton::{String, cstr};

const VALID: &str = r#"<script setup lang="ts" generic="T extends string = 'a,b'">
defineProps<{ value: T }>();
</script>
<template><div /></template>
"#;

fn invalid() -> String {
    VALID.replace("= 'a,b'", "= ").into()
}

fn register(project: &mut VirtualProject, path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    project.register_path(path).unwrap();
}

#[test]
fn planned_syntax_route_follows_repair_cycles_and_owner_removal() {
    for prefix in ["", "<!-- 🧩 日本語 -->\n"] {
        for newline in ["\n", "\r\n"] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let source = root.join("Generic.vue");
            let mut project = VirtualProject::new(&root).unwrap();
            for (content, required) in [
                (String::from(VALID), false),
                (invalid(), true),
                (String::from(VALID), false),
                (invalid(), true),
            ] {
                let content = cstr!("{prefix}{content}").replace('\n', newline);
                register(&mut project, &source, &content);
                assert_eq!(project.requires_project_session(), required);
            }
            assert!(!project.remove_registered_source(&source).is_empty());
            assert!(!project.requires_project_session());
            register(&mut project, &source, VALID);
            assert!(!project.requires_project_session());
        }
    }
}

#[test]
fn required_route_is_owned_per_source_and_survives_other_owner_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let first = root.join("First.vue");
    let second = root.join("Second.vue");
    let renamed = root.join("Renamed.vue");
    let mut project = VirtualProject::new(&root).unwrap();
    register(&mut project, &first, &invalid());
    register(&mut project, &second, &invalid());
    register(&mut project, &first, VALID);
    assert!(project.requires_project_session());
    project.remove_registered_source(&first);
    assert!(project.requires_project_session());

    fs::rename(&second, &renamed).unwrap();
    project.remove_registered_source(&second);
    assert!(!project.requires_project_session());
    project.register_path(&renamed).unwrap();
    assert!(project.requires_project_session());
    register(&mut project, &renamed, VALID);
    assert!(!project.requires_project_session());

    // Unrelated failed registration cannot drop a live required owner.
    register(&mut project, &renamed, &invalid());
    assert!(
        project
            .register_path_with_content(&root.join("unknown.extension"), "unregistered")
            .is_err()
    );
    assert!(project.requires_project_session());
}

#[test]
fn route_tracks_configuration_membership_and_canonical_removal_without_global_state() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let source = root.join("Generic.vue");
    let plain = root.join("plain.ts");
    let config = root.join("tsconfig.json");
    fs::write(&config, r#"{"files":["Generic.vue"]}"#).unwrap();
    let mut project = VirtualProject::new(&root).unwrap();
    project.set_tsconfig_path(Some(config.clone()));
    register(&mut project, &source, &invalid());
    assert!(project.requires_project_session());

    let independent = VirtualProject::new(&root).unwrap();
    assert!(!independent.requires_project_session());
    fs::write(&config, r#"{"files":["plain.ts"]}"#).unwrap();
    project.refresh_compiler_configuration();
    // The graph owner retires removed members; configuration refresh must not
    // manufacture or erase source-owned metadata independently of that graph.
    assert!(project.requires_project_session());
    let alias = root.join(".").join("Generic.vue");
    project.remove_registered_source(&alias);
    assert!(!project.requires_project_session());
    register(&mut project, &plain, "export const value = 1;\n");
    assert!(!project.requires_project_session());

    fs::write(&config, r#"{"files":["Generic.vue","plain.ts"]}"#).unwrap();
    project.refresh_compiler_configuration();
    project.register_path(&source).unwrap();
    assert!(project.requires_project_session());
}
