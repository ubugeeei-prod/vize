#![allow(clippy::disallowed_methods)]

use super::{indexed_dependency_paths, open_importers, resolve_import};
use crate::server::ServerState;
use tower_lsp::lsp_types::Url;
use vize_canon::PackageRouteResolver;
use vize_s0::cstr;

#[test]
fn index_tracks_and_removes_open_vue_imports() {
    let dir = tempfile::tempdir().unwrap();
    let components = dir.path().join("components");
    let other_components = dir.path().join("components-old");
    let child = components.join("Child.vue");
    let sibling = components.join("Sibling.vue");
    let outside = other_components.join("Outside.vue");
    let parent = dir.path().join("Parent.vue");
    std::fs::create_dir(&components).unwrap();
    std::fs::create_dir(&other_components).unwrap();
    std::fs::write(&child, "<template />").unwrap();
    std::fs::write(&sibling, "<template />").unwrap();
    std::fs::write(&outside, "<template />").unwrap();
    std::fs::write(&parent, "<template />").unwrap();
    let child_uri = Url::from_file_path(&child).unwrap();
    let components_uri = Url::from_file_path(&components).unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let state = ServerState::new();
    let source = "<script setup lang=\"ts\">import Child from './components/Child'; import Sibling from './components/Sibling.vue'; import Outside from './components-old/Outside.vue'</script>";

    state.update_virtual_docs(&parent_uri, source);
    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri.clone()]);
    assert_eq!(
        open_importers(&state, &components_uri),
        vec![parent_uri.clone()],
        "directory events return an importer once even with several nested imports"
    );
    assert_eq!(
        indexed_dependency_paths(&state, &components),
        vec![
            std::fs::canonicalize(&child).unwrap(),
            std::fs::canonicalize(&sibling).unwrap(),
        ],
        "a sibling whose name only shares the string prefix is excluded"
    );

    state.update_virtual_docs(&parent_uri, "<script setup>const local = 1</script>");
    assert!(open_importers(&state, &child_uri).is_empty());
}

#[test]
fn directory_lookup_keeps_exact_and_nested_dependency_keys() {
    let dir = tempfile::tempdir().unwrap();
    let bundle = dir.path().join("bundle.vue");
    let nested = bundle.join("Nested.vue");
    std::fs::create_dir(&bundle).unwrap();
    std::fs::write(&nested, "<template />").unwrap();
    let direct_uri = Url::from_file_path(dir.path().join("Direct.vue")).unwrap();
    let nested_uri = Url::from_file_path(dir.path().join("NestedImporter.vue")).unwrap();
    let bundle_uri = Url::from_file_path(&bundle).unwrap();
    let state = ServerState::new();

    state.update_virtual_docs(
        &direct_uri,
        "<script setup>import Bundle from './bundle.vue'</script>",
    );
    state.update_virtual_docs(
        &nested_uri,
        "<script setup>import Nested from './bundle.vue/Nested.vue'</script>",
    );

    assert_eq!(
        open_importers(&state, &bundle_uri),
        vec![direct_uri, nested_uri]
    );
}

#[test]
fn index_resolves_explicit_script_dependencies_and_query_suffixes() {
    let dir = tempfile::tempdir().unwrap();
    let child = dir.path().join("types.ts");
    let parent = dir.path().join("Parent.vue");
    std::fs::write(&child, "export type Count = number").unwrap();
    std::fs::write(&parent, "<template />").unwrap();
    let child_uri = Url::from_file_path(&child).unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let state = ServerState::new();
    let source = "<script>import './types.ts?raw'</script>";

    state.update_virtual_docs(&parent_uri, source);
    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri]);
}

#[test]
fn index_keeps_explicit_missing_vue_imports_for_future_create_events() {
    let dir = tempfile::tempdir().unwrap();
    let child_uri = Url::from_file_path(dir.path().join("FutureChild.vue")).unwrap();
    let parent_uri = Url::from_file_path(dir.path().join("Parent.vue")).unwrap();
    let state = ServerState::new();
    let source = "<script setup lang=\"ts\">import Child from './FutureChild.vue'</script>";

    state.update_virtual_docs(&parent_uri, source);

    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri]);
}

#[test]
fn index_keeps_nuxt_alias_importers_addressable_after_dependency_delete() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let nuxt = root.join(".nuxt");
    let pages = root.join("app/pages/[[server]]/list/[list]/index");
    let components = root.join("app/components/list");
    std::fs::create_dir_all(&nuxt).unwrap();
    std::fs::create_dir_all(&pages).unwrap();
    std::fs::create_dir_all(&components).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"references":[{"path":"./.nuxt/tsconfig.app.json"}],"files":[]}"#,
    )
    .unwrap();
    std::fs::write(
        nuxt.join("tsconfig.app.json"),
        r#"{"compilerOptions":{"paths":{"~/*":["../app/*"]}}}"#,
    )
    .unwrap();

    let child = components.join("SearchResult.vue");
    let parent = pages.join("accounts.vue");
    std::fs::write(&child, "<template />\n").unwrap();
    std::fs::write(&parent, "<template />\n").unwrap();
    let child_uri = Url::from_file_path(&child).unwrap();
    let canonical_parent_uri = Url::from_file_path(&parent).unwrap();
    let parent_uri = Url::parse(
        &canonical_parent_uri
            .as_str()
            .replace('[', "%5B")
            .replace(']', "%5D"),
    )
    .unwrap();
    assert_ne!(parent_uri, canonical_parent_uri);
    let state = ServerState::new();
    let source = r#"<script setup lang="ts">import SearchResult from '~/components/list/SearchResult.vue'</script>"#;

    state.update_virtual_docs(&parent_uri, source);
    std::fs::remove_file(child).unwrap();

    assert_eq!(open_importers(&state, &child_uri), vec![parent_uri]);
}

#[test]
fn index_resolves_package_export_declaration_variants() {
    let dir = tempfile::tempdir().unwrap();
    let package = dir.path().join("node_modules/vue-router");
    let parent = dir.path().join("Parent.vue");
    let module_declaration = package.join("routes.d.mts");
    let common_declaration = package.join("plugin.d.cts");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{
  "exports": {
    "./auto-routes": { "types": "./routes.d.mts" },
    "./volar/plugin": { "types": "./plugin.d.cts" }
  }
}"#,
    )
    .unwrap();
    std::fs::write(
        &module_declaration,
        "export declare const routes: unknown[]",
    )
    .unwrap();
    std::fs::write(&common_declaration, "export declare const plugin: unknown").unwrap();
    std::fs::write(&parent, "<template />").unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let module_uri = Url::from_file_path(&module_declaration).unwrap();
    let common_uri = Url::from_file_path(&common_declaration).unwrap();
    let state = ServerState::new();
    let source = r#"<script setup lang="ts">
import { routes } from 'vue-router/auto-routes'
import { plugin } from 'vue-router/volar/plugin'
void routes
void plugin
</script>"#;

    state.update_virtual_docs(&parent_uri, source);

    assert_eq!(
        open_importers(&state, &module_uri),
        vec![parent_uri.clone()]
    );
    assert_eq!(open_importers(&state, &common_uri), vec![parent_uri]);
}

#[test]
fn index_tracks_workspace_package_vue_source_and_manifest_retarget() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    let package = dir.path().join("packages/ui");
    let parent = app.join("src/Parent.vue");
    let original = package.join("src/Widget.vue");
    let renamed = package.join("src/Renamed.vue");
    std::fs::create_dir_all(parent.parent().unwrap()).unwrap();
    std::fs::create_dir_all(original.parent().unwrap()).unwrap();
    std::fs::write(&parent, "<template />\n").unwrap();
    std::fs::write(&original, "<template />\n").unwrap();
    write_package_manifest(&package, "Widget.vue");
    link_package(&package, &app.join("node_modules/@scope/ui"));
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let original_uri = Url::from_file_path(original.canonicalize().unwrap()).unwrap();
    let manifest_uri =
        Url::from_file_path(package.join("package.json").canonicalize().unwrap()).unwrap();
    let state = ServerState::new();
    let source =
        "<script setup lang=\"ts\">import Widget from '@scope/ui/widget'; void Widget</script>";

    state.update_virtual_docs(&parent_uri, source);
    assert_eq!(
        open_importers(&state, &original_uri),
        std::slice::from_ref(&parent_uri)
    );
    assert_eq!(
        open_importers(&state, &manifest_uri),
        std::slice::from_ref(&parent_uri),
        "package.json changes must refresh the importing Vue document",
    );

    std::fs::rename(&original, &renamed).unwrap();
    write_package_manifest(&package, "Renamed.vue");
    state.update_virtual_docs(&parent_uri, source);
    let renamed_uri = Url::from_file_path(renamed.canonicalize().unwrap()).unwrap();
    assert!(open_importers(&state, &original_uri).is_empty());
    assert_eq!(open_importers(&state, &renamed_uri), [parent_uri]);
}

#[test]
fn index_keeps_missing_package_link_and_manifest_addressable_for_creation() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("app");
    let package = dir.path().join("packages/ui");
    let parent = app.join("src/Parent.vue");
    let source_path = package.join("src/Widget.vue");
    let package_link = app.join("node_modules/@scope/ui");
    let logical_manifest = package_link.join("package.json");
    std::fs::create_dir_all(parent.parent().unwrap()).unwrap();
    std::fs::write(&parent, "<template />\n").unwrap();
    let parent_uri = Url::from_file_path(&parent).unwrap();
    let logical_manifest_uri = Url::from_file_path(&logical_manifest).unwrap();
    let state = ServerState::new();
    let source =
        "<script setup lang=\"ts\">import Widget from '@scope/ui/widget'; void Widget</script>";

    state.update_virtual_docs(&parent_uri, source);
    assert_eq!(
        open_importers(&state, &logical_manifest_uri),
        std::slice::from_ref(&parent_uri),
        "an unresolved lookup must index the logical manifest candidate",
    );

    std::fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    std::fs::write(&source_path, "<template />\n").unwrap();
    write_package_manifest(&package, "Widget.vue");
    link_package(&package, &package_link);
    state.update_virtual_docs(&parent_uri, source);

    let physical_source_uri = Url::from_file_path(source_path.canonicalize().unwrap()).unwrap();
    assert_eq!(
        open_importers(&state, &physical_source_uri),
        [parent_uri],
        "reindexing after creation must replace the negative route with its source",
    );
}

fn write_package_manifest(package: &std::path::Path, target: &str) {
    std::fs::write(
        package.join("package.json"),
        cstr!("{{\"name\":\"@scope/ui\",\"exports\":{{\"./widget\":\"./src/{target}\"}}}}"),
    )
    .unwrap();
}

fn link_package(source: &std::path::Path, target: &std::path::Path) {
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

#[test]
fn exact_directory_specifiers_resolve_index_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_dir = dir.path().join("src");
    let source_index = source_dir.join("index.ts");
    let parent_index = dir.path().join("index.ts");
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(&source_index, "export const source = true").unwrap();
    std::fs::write(&parent_index, "export const parent = true").unwrap();
    std::fs::write(dir.path().join("src.vue"), "<template />").unwrap();

    let mut routes = PackageRouteResolver::default();
    assert_eq!(
        resolve_import(&source_dir, ".?raw", &mut routes).dependencies,
        [std::fs::canonicalize(&source_index).unwrap()]
    );
    assert_eq!(
        resolve_import(&source_dir, "..#parent", &mut routes).dependencies,
        [std::fs::canonicalize(&parent_index).unwrap()]
    );
}
