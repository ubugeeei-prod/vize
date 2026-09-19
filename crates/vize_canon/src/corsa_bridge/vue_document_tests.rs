use super::vue_document::{
    CorsaVueVirtualDocumentOptions, build_vue_virtual_project,
    build_vue_virtual_project_with_overlays,
};
use crate::CorsaBridge;
use crate::file_uri::path_to_file_uri;
use vize_carton::cstr;

#[test]
fn resolved_dependencies_include_closed_barrels_but_exclude_unrelated_overlays() {
    let project = tempfile::tempdir().unwrap();
    let host_path = project.path().join("Host.vue");
    let barrel = project.path().join("barrel.ts");
    let leaf = project.path().join("leaf.ts");
    let unrelated = project.path().join("Unrelated.vue");
    let source = "<script setup lang=\"ts\">import { value } from './barrel';</script><template>{{ value }}</template>";
    std::fs::write(&host_path, source).unwrap();
    std::fs::write(&barrel, "export { value } from './leaf';").unwrap();
    std::fs::write(&leaf, "export const value = 1;").unwrap();
    let virtual_project = build_vue_virtual_project_with_overlays(
        &host_path,
        source,
        CorsaVueVirtualDocumentOptions::default(),
        &[(unrelated, "<template />")],
    )
    .unwrap();
    let mut expected = vec![
        vize_carton::path::canonicalize_non_verbatim(&barrel),
        vize_carton::path::canonicalize_non_verbatim(&leaf),
    ];
    expected.sort();
    assert_eq!(virtual_project.host.resolved_dependencies, expected);
}

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
    let mirror = virtual_project
        .session_project_root
        .as_ref()
        .expect("relative dependencies must be materialized");
    let uris: Vec<&str> = virtual_project
        .documents
        .iter()
        .map(|(uri, _)| uri.as_str())
        .collect();

    assert!(
        virtual_project
            .host
            .code
            .contains(cstr!("\"{}\"", mirror.join("Child.vue.ts").display()).as_str())
    );
    assert!(uris.contains(&path_to_file_uri(&mirror.join("Host.vue.ts")).as_str()));
    assert!(uris.contains(&path_to_file_uri(&mirror.join("Child.vue.ts")).as_str()));
    assert!(uris.contains(&path_to_file_uri(&mirror.join("GrandChild.vue.ts")).as_str()));
    assert!(
        uris.contains(&path_to_file_uri(&mirror.join(util_path.file_name().unwrap())).as_str()),
        "uris: {uris:?}\n{}",
        virtual_project.host.pre_rewrite_code,
    );
    let types_document = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| {
            uri == path_to_file_uri(&mirror.join(types_path.file_name().unwrap())).as_str()
        })
        .map(|(_, content)| content.as_str())
        .expect("TS dependency document should be synced");
    assert!(
        types_document
            .contains(cstr!("import(\"{}\")", mirror.join("Child.vue.ts").display()).as_str())
            && types_document
                .contains(cstr!("from \"{}\"", mirror.join("Child.vue.ts").display()).as_str()),
        "TS dependency Vue specifiers must target virtual Vue modules:\n{types_document}",
    );
    assert!(
        uris.contains(&path_to_file_uri(&mirror.join(helper_path.file_name().unwrap())).as_str()),
        "TS import-type dependencies must be synced too: {uris:?}",
    );
    assert!(
        uris.contains(&path_to_file_uri(&mirror.join(schema_path.file_name().unwrap())).as_str()),
        "extensionless TS import-type dependencies must resolve generated d.ts files too: {uris:?}",
    );
    assert!(
        uris.contains(
            &path_to_file_uri(&mirror.join(child_util_path.file_name().unwrap())).as_str()
        ),
        "nested dependency imports must be synced too: {uris:?}",
    );
    assert_eq!(
        uris.iter()
            .filter(|uri| **uri == path_to_file_uri(&mirror.join("Child.vue.ts")).as_str())
            .count(),
        1,
        "Vue dependency documents must be de-duplicated: {uris:?}",
    );
}

#[test]
fn vue_virtual_project_prefers_open_dependency_overlays() {
    let project = tempfile::TempDir::new().expect("temp project");
    let host_path = project.path().join("Host.vue");
    let child_path = project.path().join("Child.vue");
    let host = r#"<script setup lang="ts">import Child from "./Child.vue"</script>
<template><Child count="one" /></template>"#;
    std::fs::write(&host_path, host).expect("host");
    std::fs::write(
        &child_path,
        "<script setup lang=\"ts\">defineProps<{ count: string }>()</script>",
    )
    .expect("child");
    let overlays = vec![(
        child_path.clone(),
        "<script setup lang=\"ts\">defineProps<{ count: number }>()</script>",
    )];

    let virtual_project = build_vue_virtual_project_with_overlays(
        &host_path,
        host,
        CorsaVueVirtualDocumentOptions::default(),
        &overlays,
    )
    .expect("virtual project");
    let child = virtual_project
        .documents
        .iter()
        .find(|(uri, _)| {
            uri == path_to_file_uri(
                &virtual_project
                    .session_project_root
                    .as_ref()
                    .unwrap()
                    .join("Child.vue.ts"),
            )
            .as_str()
        })
        .map(|(_, content)| content.as_str())
        .expect("child virtual document");
    assert!(child.contains("count: number"), "{child}");
    assert!(!child.contains("count: string"), "{child}");
}

#[test]
fn owned_overlay_bridge_api_remains_source_compatible() {
    let bridge = CorsaBridge::new();
    let overlays: Vec<(std::path::PathBuf, vize_carton::String)> =
        vec![(std::path::PathBuf::from("Child.vue"), "".into())];
    let _future = bridge.open_vue_virtual_document_with_overlays(
        std::path::Path::new("Host.vue"),
        "",
        CorsaVueVirtualDocumentOptions::default(),
        &overlays,
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

#[test]
fn materialized_and_opened_vue_projections_share_code_and_coordinates() {
    let project = tempfile::tempdir().unwrap();
    let host = project.path().join("Host.vue");
    let child = project.path().join("Child.vue");
    let source = "<script setup lang=\"ts\">import Child from './Child.vue';</script><template><Child :title=\"'hello'\" @save=\"() => {}\" /></template>";
    let child_source = "<script setup lang=\"ts\">defineProps<{ title: string }>(); defineEmits<{ save: [] }>();</script><template>{{ title }}</template>";
    std::fs::write(&host, source).unwrap();
    std::fs::write(&child, child_source).unwrap();
    for preserve_event_navigation in [false, true] {
        let options = CorsaVueVirtualDocumentOptions {
            preserve_event_navigation,
            ..Default::default()
        };
        for content in [
            child_source.to_owned(),
            child_source.replace("title: string", "title: number"),
        ] {
            let virtual_project = build_vue_virtual_project_with_overlays(
                &host,
                source,
                options,
                &[(child.clone(), content.as_str())],
            )
            .unwrap();
            let opened = &virtual_project.host;
            for (uri, code, mappings) in
                std::iter::once((&opened.request_uri, &opened.code, &opened.mappings)).chain(
                    opened.dependencies.iter().map(|dependency| {
                        (
                            &dependency.request_uri,
                            &dependency.code,
                            &dependency.mappings,
                        )
                    }),
                )
            {
                let materialized = opened
                    .materialized_sources
                    .iter()
                    .find(|file| path_to_file_uri(&file.materialized_path) == *uri)
                    .unwrap();
                assert_eq!(
                    *code, materialized.code,
                    "one URI must identify one generated program"
                );
                assert_eq!(
                    *mappings, materialized.mappings,
                    "overlays must query coordinates from that exact program"
                );
                assert_eq!(
                    std::fs::read_to_string(&materialized.materialized_path).unwrap(),
                    code.as_str()
                );
            }
        }
    }
}
