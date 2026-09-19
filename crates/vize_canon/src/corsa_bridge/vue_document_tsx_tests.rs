use super::vue_document::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};
use crate::file_uri::path_to_file_uri;
use vize_carton::cstr;

#[test]
fn vue_virtual_project_syncs_tsx_vue_dependency_shims() {
    let project = tempfile::TempDir::new().expect("temp project");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src dir");

    let host_path = src.join("Host.vue");
    let child_path = src.join("Child.vue");
    std::fs::write(
        &host_path,
        r#"<script setup lang="ts">
import Child from "./Child.vue";
const component = Child;
</script>
<template><component /></template>
"#,
    )
    .expect("host");
    std::fs::write(
        &child_path,
        r#"<script setup lang="tsx">
export interface ChildProps {
  label?: string
}

defineProps<ChildProps>();
const vnode = null as unknown;
</script>
<template><span /></template>
"#,
    )
    .expect("child");

    let host = std::fs::read_to_string(&host_path).expect("host source");
    let virtual_project =
        build_vue_virtual_project(&host_path, &host, CorsaVueVirtualDocumentOptions::default())
            .expect("virtual project");
    let mirror = virtual_project.session_project_root.as_ref().unwrap();
    let child_shim_uri = path_to_file_uri(&mirror.join("Child.vue.ts"));
    let child_tsx_uri = path_to_file_uri(&mirror.join("Child.vue.tsx"));
    let uris = virtual_project
        .documents
        .iter()
        .map(|(uri, _)| uri.as_str())
        .collect::<Vec<_>>();

    assert!(
        virtual_project
            .host
            .code
            .contains(cstr!("\"{}\"", mirror.join("Child.vue").display()).as_str()),
        "host import should target the stable Vue shim:\n{}",
        virtual_project.host.code
    );
    assert!(
        uris.contains(&child_tsx_uri.as_str()),
        "TSX Vue dependency body must be synced: {uris:?}",
    );
    let shim = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| uri == child_shim_uri.as_str())
        .map(|(_, content)| content.as_str())
        .expect("TSX Vue dependency must also sync a .vue.ts import shim");
    assert_eq!(
        shim,
        "export { default } from \"./Child.vue.tsx\";\nexport * from \"./Child.vue.tsx\";\n"
    );
}

#[test]
fn vue_virtual_project_syncs_tsx_host_shim_for_ts_dependencies() {
    let project = tempfile::TempDir::new().expect("temp project");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src dir");

    let host_path = src.join("Host.vue");
    let types_path = src.join("types.ts");
    std::fs::write(
        &host_path,
        r#"<script setup lang="tsx">
import type { HostModule } from "./types";
type _HostModule = HostModule;
const vnode = null as unknown;
</script>
<template><span /></template>
"#,
    )
    .expect("host");
    std::fs::write(
        &types_path,
        r#"export type HostModule = typeof import("./Host.vue");
"#,
    )
    .expect("types");

    let host = std::fs::read_to_string(&host_path).expect("host source");
    let virtual_project =
        build_vue_virtual_project(&host_path, &host, CorsaVueVirtualDocumentOptions::default())
            .expect("virtual project");
    let mirror = virtual_project.session_project_root.as_ref().unwrap();
    let host_shim_uri = path_to_file_uri(&mirror.join("Host.vue.ts"));
    let host_tsx_uri = path_to_file_uri(&mirror.join("Host.vue.tsx"));
    let uris = virtual_project
        .documents
        .iter()
        .map(|(uri, _)| uri.as_str())
        .collect::<Vec<_>>();

    assert_eq!(virtual_project.host.request_uri, host_tsx_uri);
    let types_document = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| uri == path_to_file_uri(&types_path).as_str())
        .map(|(_, content)| content.as_str())
        .expect("TS dependency document should be synced");
    assert!(
        types_document
            .contains(cstr!("import(\"{}\")", mirror.join("Host.vue").display()).as_str()),
        "TS dependencies must target the stable Vue shim:\n{types_document}",
    );
    let shim = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| uri == host_shim_uri.as_str())
        .map(|(_, content)| content.as_str())
        .expect("TSX host must also sync a .vue.ts import shim");
    assert_eq!(
        shim,
        "export { default } from \"./Host.vue.tsx\";\nexport * from \"./Host.vue.tsx\";\n"
    );
    assert!(
        uris.contains(&host_tsx_uri.as_str()),
        "TSX host body must remain the request document: {uris:?}",
    );
}
