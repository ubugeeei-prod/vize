use super::super::vue_document::{
    CorsaVueVirtualDocumentOptions, build_prebuilt_vue_virtual_project_with_overlays,
};
use crate::batch::{
    ImportRewriter, VueDocumentVirtualTsOptions, generate_vue_document_virtual_ts_with_options,
};
use crate::file_uri::path_to_file_uri;
use crate::virtual_ts::VirtualTsOptions;
use vize_atlas::Shared;

#[test]
fn prebuilt_project_sync_does_not_reparse_host_or_vue_overlays() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let child_path = project.path().join("Child.vue");
    let host = r#"<script setup lang="ts">import Child from "./Child.vue"</script>
<template><Child /></template>"#;
    let child = r#"<script setup lang="ts">defineProps<{ count: number }>()</script>
<template><span /></template>"#;
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(&child_path, child).expect("child");
    let rewriter = ImportRewriter::new();
    let generate = |path: &std::path::Path, source: &str| {
        generate_vue_document_virtual_ts_with_options(
            path,
            source,
            &VirtualTsOptions::default(),
            &rewriter,
            false,
            VueDocumentVirtualTsOptions::default(),
        )
        .expect("prebuilt Vue document")
    };
    let host_document = generate(&host_path, host);
    let child_document = Shared::new(generate(&child_path, child));
    vize_atelier_sfc::reset_authored_script_parse_invocations();

    let virtual_project = build_prebuilt_vue_virtual_project_with_overlays(
        &host_path,
        host_document,
        CorsaVueVirtualDocumentOptions::default(),
        &[(child_path.clone(), child.into())],
        &[(child_path.clone(), child_document)],
    );

    assert_eq!(vize_atelier_sfc::authored_script_parse_invocations(), 0);
    assert!(virtual_project.documents.iter().any(|(uri, content)| {
        uri == path_to_file_uri(&child_path.with_extension("vue.ts")).as_str()
            && content.contains("count: number")
    }));
}

#[test]
fn prebuilt_project_never_generates_a_missing_vue_overlay() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let child_path = project.path().join("Child.vue");
    let host = r#"<script setup>import Child from "./Child.vue"</script>
<template><Child /></template>"#;
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(&child_path, "<script setup>const reparsed = true</script>").expect("child");
    let host_document = generate_vue_document_virtual_ts_with_options(
        &host_path,
        host,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
        VueDocumentVirtualTsOptions::default(),
    )
    .expect("prebuilt host");
    vize_atelier_sfc::reset_authored_script_parse_invocations();

    let virtual_project = build_prebuilt_vue_virtual_project_with_overlays(
        &host_path,
        host_document,
        CorsaVueVirtualDocumentOptions::default(),
        &[],
        &[],
    );

    assert_eq!(vize_atelier_sfc::authored_script_parse_invocations(), 0);
    assert!(virtual_project.documents.iter().any(|(uri, content)| {
        uri == path_to_file_uri(&child_path.with_extension("vue.ts")).as_str()
            && content.contains("const component: any")
            && !content.contains("reparsed")
    }));
}
