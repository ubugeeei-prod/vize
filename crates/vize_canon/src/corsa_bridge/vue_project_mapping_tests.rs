use super::vue_document::{
    CorsaVueVirtualDocumentOptions, build_vue_virtual_project,
    build_vue_virtual_project_with_overlays,
};
use crate::file_uri::path_to_file_uri;

#[test]
fn retains_exact_mappings_for_reachable_nested_vue_dependencies() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let child_path = project.path().join("Child.vue");
    let grandchild_path = project.path().join("Grandchild.vue");
    let unrelated_path = project.path().join("Unrelated.vue");
    let host = r#"<script setup lang="ts">import Child from "./Child.vue"</script>
<template><Child /></template>"#;
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(
        &child_path,
        r#"<script setup lang="ts">
import Grandchild from "./Grandchild.vue";
const childValue = 1;
</script>
<template><Grandchild>{{ childValue }}</Grandchild></template>"#,
    )
    .expect("child");
    std::fs::write(
        &grandchild_path,
        "<script setup lang=\"ts\">const leaf = true</script>\n<template>{{ leaf }}</template>",
    )
    .expect("grandchild");
    std::fs::write(&unrelated_path, "<template><aside /></template>").expect("unrelated");

    let virtual_project =
        build_vue_virtual_project(&host_path, host, CorsaVueVirtualDocumentOptions::default())
            .expect("virtual project");
    let dependency_paths = virtual_project
        .host
        .dependencies
        .iter()
        .map(|dependency| dependency.source_path.as_path())
        .collect::<Vec<_>>();
    assert_eq!(
        dependency_paths,
        vec![child_path.as_path(), grandchild_path.as_path()]
    );
    assert!(!dependency_paths.contains(&unrelated_path.as_path()));

    let child = &virtual_project.host.dependencies[0];
    let authored_offset = child.source.find("childValue").expect("authored token");
    let mapping = child
        .mappings
        .iter()
        .filter(|mapping| {
            authored_offset >= mapping.src_range.start && authored_offset < mapping.src_range.end
        })
        .min_by_key(|mapping| mapping.src_range.end - mapping.src_range.start)
        .expect("mapping for dependency token");
    let generated_pre_rewrite =
        mapping.gen_range.start + authored_offset.saturating_sub(mapping.src_range.start);
    let generated_post_rewrite = child
        .import_source_map
        .get_virtual_offset(generated_pre_rewrite as u32) as usize;
    assert_eq!(
        &child.code[generated_post_rewrite..generated_post_rewrite + "childValue".len()],
        "childValue",
        "metadata must map through the exact import rewrite synchronized with Corsa",
    );
}

#[test]
fn retained_dependency_source_is_the_open_overlay_snapshot() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let child_path = project.path().join("Child.vue");
    let host = "<script setup lang=\"ts\">import Child from \"./Child.vue\"</script>";
    let disk = "<script setup lang=\"ts\">defineProps<{ count: string }>()</script>";
    let overlay = "<script setup lang=\"ts\">defineProps<{ count: number }>()</script>";
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(&child_path, disk).expect("child");

    let virtual_project = build_vue_virtual_project_with_overlays(
        &host_path,
        host,
        CorsaVueVirtualDocumentOptions::default(),
        &[(child_path.clone(), overlay)],
    )
    .expect("virtual project");
    let child = virtual_project
        .host
        .dependencies
        .first()
        .expect("child dependency metadata");
    assert_eq!(child.source, overlay);
    assert!(child.code.contains("count: number"));
    assert!(!child.code.contains("count: string"));
}

#[test]
fn retains_tsx_dependency_identity_but_not_its_ts_import_shim() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let child_path = project.path().join("Child.vue");
    let host = "<script setup lang=\"ts\">import Child from \"./Child.vue\"</script>";
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(
        &child_path,
        "<script setup lang=\"tsx\">const vnode = <div /></script>",
    )
    .expect("child");

    let virtual_project =
        build_vue_virtual_project(&host_path, host, CorsaVueVirtualDocumentOptions::default())
            .expect("virtual project");
    let dependencies = &virtual_project.host.dependencies;
    assert_eq!(
        dependencies.len(),
        1,
        "the .vue.ts shim is not authored Vue"
    );
    assert_eq!(
        dependencies[0].request_uri,
        path_to_file_uri(&child_path.with_extension("vue.tsx"))
    );
    assert_eq!(dependencies[0].virtual_suffix, ".tsx");
    assert!(dependencies[0].source_type.is_typescript());
    assert!(dependencies[0].source_type.is_jsx());
}

#[test]
fn does_not_claim_authored_mappings_for_unparseable_fallbacks() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let broken_path = project.path().join("Broken.vue");
    let host = "<script setup lang=\"ts\">import Broken from \"./Broken.vue\"</script>";
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(&broken_path, "<template><div></div>").expect("broken");

    let virtual_project =
        build_vue_virtual_project(&host_path, host, CorsaVueVirtualDocumentOptions::default())
            .expect("virtual project");
    assert!(virtual_project.host.dependencies.is_empty());
}
