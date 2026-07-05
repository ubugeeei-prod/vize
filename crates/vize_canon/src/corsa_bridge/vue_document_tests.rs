use super::vue_document::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};
use crate::file_uri::path_to_file_uri;

#[test]
fn vue_virtual_project_syncs_relative_vue_and_ts_dependencies() {
    let project = tempfile::TempDir::new().expect("temp project");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src dir");

    let host_path = src.join("Host.vue");
    let child_path = src.join("Child.vue");
    let grand_child_path = src.join("GrandChild.vue");
    let util_path = src.join("util.ts");
    let types_path = src.join("types.ts");
    let helper_path = src.join("helper.ts");
    let schema_path = src.join("schema.d.ts");
    let child_util_path = src.join("childUtil.ts");
    std::fs::write(
        &host_path,
        r#"<script setup lang="ts">
import Child from "./Child.vue";
import { value } from "./util";
import type { ChildModule } from "./types";
const current = value;
type _ChildModule = ChildModule;
</script>
<template><Child :value="current" /></template>
"#,
    )
    .expect("host");
    std::fs::write(
        &child_path,
        r#"<script setup lang="ts">
import GrandChild from "./GrandChild.vue";
import { childValue } from "./childUtil";
defineProps<{ value: number }>();
const _grandChild = GrandChild;
const _childValue = childValue;
</script>
<template><GrandChild /></template>
"#,
    )
    .expect("child");
    std::fs::write(
        &grand_child_path,
        r#"<script setup lang="ts">
defineProps<{ label?: string }>();
</script>
<template><span /></template>
"#,
    )
    .expect("grand child");
    std::fs::write(&util_path, "export const value = 1;\n").expect("util");
    std::fs::write(
        &types_path,
        r#"export type ChildModule = typeof import("./Child.vue");
export type HelperModule = import("./helper").Helper;
export type SchemaModule = import("./schema").Schema;
export { default as ReexportedChild } from "./Child.vue";
"#,
    )
    .expect("types");
    std::fs::write(&helper_path, "export type Helper = { ok: true };\n").expect("helper");
    std::fs::write(&schema_path, "export type Schema = { id: string };\n").expect("schema");
    std::fs::write(&child_util_path, "export const childValue = 2;\n").expect("child util");

    let host = std::fs::read_to_string(&host_path).expect("host source");
    let virtual_project =
        build_vue_virtual_project(&host_path, &host, CorsaVueVirtualDocumentOptions::default())
            .expect("virtual project");
    let uris: Vec<&str> = virtual_project
        .documents
        .iter()
        .map(|(uri, _)| uri.as_str())
        .collect();

    assert!(virtual_project.host.code.contains("\"./Child.vue.ts\""));
    assert!(uris.contains(&path_to_file_uri(&src.join("Host.vue.ts")).as_str()));
    assert!(uris.contains(&path_to_file_uri(&src.join("Child.vue.ts")).as_str()));
    assert!(uris.contains(&path_to_file_uri(&src.join("GrandChild.vue.ts")).as_str()));
    assert!(
        uris.contains(&path_to_file_uri(&util_path).as_str()),
        "uris: {uris:?}\n{}",
        virtual_project.host.pre_rewrite_code,
    );
    let types_document = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| uri == path_to_file_uri(&types_path).as_str())
        .map(|(_, content)| content.as_str())
        .expect("TS dependency document should be synced");
    assert!(
        types_document.contains(r#"import("./Child.vue.ts")"#)
            && types_document.contains(r#"from "./Child.vue.ts""#),
        "TS dependency Vue specifiers must target virtual Vue modules:\n{types_document}",
    );
    assert!(
        uris.contains(&path_to_file_uri(&helper_path).as_str()),
        "TS import-type dependencies must be synced too: {uris:?}",
    );
    assert!(
        uris.contains(&path_to_file_uri(&schema_path).as_str()),
        "extensionless TS import-type dependencies must resolve generated d.ts files too: {uris:?}",
    );
    assert!(
        uris.contains(&path_to_file_uri(&child_util_path).as_str()),
        "nested dependency imports must be synced too: {uris:?}",
    );
    assert_eq!(
        uris.iter()
            .filter(|uri| **uri == path_to_file_uri(&src.join("Child.vue.ts")).as_str())
            .count(),
        1,
        "Vue dependency documents must be de-duplicated: {uris:?}",
    );
}

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
    let child_shim_uri = path_to_file_uri(&src.join("Child.vue.ts"));
    let child_tsx_uri = path_to_file_uri(&src.join("Child.vue.tsx"));
    let uris = virtual_project
        .documents
        .iter()
        .map(|(uri, _)| uri.as_str())
        .collect::<Vec<_>>();

    assert!(
        virtual_project.host.code.contains("\"./Child.vue.ts\""),
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
fn vue_virtual_project_stubs_existing_unparseable_vue_dependencies() {
    let project = tempfile::TempDir::new().expect("temp project");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src dir");

    let host_path = src.join("Host.vue");
    let broken_path = src.join("Broken.vue");
    std::fs::write(
        &host_path,
        r#"<script setup lang="ts">
import Broken from "./Broken.vue";
const _broken = Broken;
</script>
<template><Broken /></template>
"#,
    )
    .expect("host");
    std::fs::write(&broken_path, "<template><div></div>").expect("broken dependency");

    let host = std::fs::read_to_string(&host_path).expect("host source");
    let virtual_project =
        build_vue_virtual_project(&host_path, &host, CorsaVueVirtualDocumentOptions::default())
            .expect("host virtual project");
    let broken_virtual_uri = path_to_file_uri(&src.join("Broken.vue.ts"));
    let broken_document = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| uri == broken_virtual_uri.as_str())
        .map(|(_, content)| content.as_str())
        .expect("existing malformed Vue dependency still needs a virtual module");

    assert!(
        virtual_project.host.code.contains("\"./Broken.vue.ts\""),
        "host import must target the virtual Vue mirror:\n{}",
        virtual_project.host.code,
    );
    assert_eq!(
        broken_document,
        "const component: any = undefined;\nexport default component;\n"
    );
    assert!(
        !src.join("Broken.vue.ts").exists(),
        "fallback dependency must be synced in-memory, not written next to the source file"
    );
}
