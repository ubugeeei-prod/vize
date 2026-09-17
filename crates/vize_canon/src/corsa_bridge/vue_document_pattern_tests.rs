use super::vue_document::{
    CorsaVueVirtualDocumentOptions, build_vue_virtual_project_with_overlays,
};

#[test]
fn patterned_editor_alias_snapshots_preserve_dependencies_and_switch_modes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{
        "compilerOptions": { "paths": { "@/*": ["./*"] } }, "include": ["*.vue"]
    }"#,
    )
    .unwrap();
    let host = root.path().join("Host.vue");
    let child = root.path().join("Child.vue");
    let source = r#"<script setup lang="ts">
import Child from '@/Child.vue';
const value = 'a' as 'a' | 'b';
</script><template><Child/><div v-match="value"><p v-when="'a'"/><p v-when="'b'"/></div></template>"#;
    let child_source = r#"<script setup lang="ts">const item = 'a' as 'a' | 'b'</script>
<template v-match="item"><p v-when="'a'"/><p v-when="'b'"/></template>"#;
    std::fs::write(&host, source).unwrap();
    std::fs::write(&child, "<template/>").unwrap();
    let mut roots = Vec::new();
    for enabled in [true, false, true] {
        let project = build_vue_virtual_project_with_overlays(
            &host,
            source,
            CorsaVueVirtualDocumentOptions {
                experimental_patterned_template: enabled,
                ..Default::default()
            },
            &[(child.clone(), child_source)],
        )
        .unwrap();
        assert_eq!(project.host.code.contains("__VizePatterns.Match<"), enabled);
        assert_eq!(
            project
                .host
                .code
                .contains("declare namespace __VizePatterns"),
            enabled
        );
        let dependency = project
            .host
            .dependencies
            .iter()
            .find(|dependency| dependency.source_path.ends_with("Child.vue"))
            .unwrap();
        assert_eq!(dependency.code.contains("__VizePatterns.Match<"), enabled);
        assert_eq!(
            dependency.code.contains("declare namespace __VizePatterns"),
            enabled
        );
        let mirrored_child = project.host.materialized_sources.iter().find(|file| {
            file.source_path.ends_with("Child.vue") && file.code.contains("Exhaustiveness<")
        });
        assert_eq!(mirrored_child.is_some(), enabled);
        roots.push(project.session_project_root.unwrap());
    }
    assert_ne!(
        roots[0], roots[1],
        "different modes must never share a materialized namespace"
    );
    assert_eq!(roots[0], roots[2]);
}
