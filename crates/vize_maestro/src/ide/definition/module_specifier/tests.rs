use std::{fs, path::Path};

use tempfile::tempdir;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use super::{resolve_specifier, specifier_at_offset};
use crate::{
    ide::{DefinitionService, IdeContext},
    server::ServerState,
};

#[test]
fn finds_only_module_specifier_strings_at_the_cursor() {
    for (source, marker, expected) in [
        ("import { ref } from 'vue'", "vue", "vue"),
        ("import 'theme.css'", "theme", "theme.css"),
        ("export * from \"pkg/subpath\"", "subpath", "pkg/subpath"),
        ("const lazy = import('./lazy')", "lazy')", "./lazy"),
        ("const value = require('@scope/pkg')", "scope", "@scope/pkg"),
    ] {
        let offset = source.find(marker).expect("marker");
        assert_eq!(
            specifier_at_offset(source, offset),
            Some(expected),
            "{source}"
        );
    }

    let source = "const ordinary = 'vue'";
    assert_eq!(
        specifier_at_offset(source, source.find("vue").unwrap()),
        None
    );
}

#[test]
fn package_definition_without_native_fails_closed_for_conditional_candidates() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("packages/app/src/App.vue");
    write(&source, "<script setup lang=\"ts\"></script>");
    let package = workspace.path().join("node_modules/vue-router");
    write(
        &package.join("package.json"),
        r#"{"exports":{".":{"import":"./dist/index.js","types":"./dist/index.d.ts"}}}"#,
    );
    write(&package.join("dist/index.js"), "export {};");
    write(
        &package.join("dist/index.d.ts"),
        "export interface Route {}\n",
    );

    assert_eq!(resolve_specifier(&file_url(&source), "vue-router"), None);
}

#[test]
fn package_definition_without_native_keeps_an_unambiguous_types_export() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("packages/app/src/App.vue");
    write(&source, "<script setup lang=\"ts\"></script>");
    let package = workspace.path().join("node_modules/vue-router");
    write(
        &package.join("package.json"),
        r#"{"exports":{".":{"types":"./dist/index.d.ts"}}}"#,
    );
    write(
        &package.join("dist/index.d.ts"),
        "export interface Route {}\n",
    );

    let resolved = resolve_specifier(&file_url(&source), "vue-router").unwrap();
    assert_eq!(
        resolved,
        package.join("dist/index.d.ts").canonicalize().unwrap()
    );
}

#[test]
fn definition_service_navigates_module_specifier_to_types_export() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("src/App.vue");
    let content =
        "<script setup lang=\"ts\">\nimport type { Route } from 'vue-router'\n</script>\n";
    write(&source, content);
    let package = workspace.path().join("node_modules/vue-router");
    write(
        &package.join("package.json"),
        r#"{"exports":{".":{"types":"./dist/index.d.ts"}}}"#,
    );
    let declaration = package.join("dist/index.d.ts");
    write(&declaration, "export interface Route {}\n");

    let uri = file_url(&source);
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);
    let offset = content.find("vue-router").unwrap() + 1;
    let context = IdeContext::new(&state, &uri, offset).unwrap();

    let definition = DefinitionService::definition(&context).unwrap();
    let GotoDefinitionResponse::Scalar(location) = definition else {
        panic!("module specifier definition must be a scalar location");
    };
    assert_eq!(location.uri, file_url(&declaration.canonicalize().unwrap()));
}

#[test]
fn definition_session_detects_a_same_mtime_package_manifest_retarget() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("src/App.vue");
    let content = "<script setup lang=\"ts\">\nimport type { Route } from 'router'\n</script>\n";
    write(&source, content);
    let package = workspace.path().join("node_modules/router");
    let manifest = package.join("package.json");
    let first_manifest = r#"{"exports":{".":{"types":"./dist/Alpha.d.ts"}}}"#;
    let second_manifest = r#"{"exports":{".":{"types":"./dist/Bravo.d.ts"}}}"#;
    assert_eq!(first_manifest.len(), second_manifest.len());
    write(&manifest, first_manifest);
    let alpha = package.join("dist/Alpha.d.ts");
    let bravo = package.join("dist/Bravo.d.ts");
    write(&alpha, "export interface Route {}\n");
    write(&bravo, "export interface Route {}\n");

    let uri = file_url(&source);
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);
    let offset = content.find("router").unwrap() + 1;
    let first = IdeContext::new(&state, &uri, offset).unwrap();
    let GotoDefinitionResponse::Scalar(first) =
        DefinitionService::definition(&first).expect("first definition")
    else {
        panic!("module specifier definition must be scalar");
    };
    assert_eq!(first.uri, file_url(&alpha.canonicalize().unwrap()));

    let modified = fs::metadata(&manifest).unwrap().modified().unwrap();
    fs::write(&manifest, second_manifest).unwrap();
    fs::File::options()
        .write(true)
        .open(&manifest)
        .unwrap()
        .set_modified(modified)
        .unwrap();
    let second = IdeContext::new(&state, &uri, offset).unwrap();
    let GotoDefinitionResponse::Scalar(second) =
        DefinitionService::definition(&second).expect("retargeted definition")
    else {
        panic!("module specifier definition must be scalar");
    };
    assert_eq!(second.uri, file_url(&bravo.canonicalize().unwrap()));
}

#[test]
fn resolves_scoped_wildcard_declaration_subpath() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("src/App.vue");
    write(&source, "<script setup lang=\"ts\"></script>");
    let package = workspace.path().join("node_modules/@scope/router");
    write(
        &package.join("package.json"),
        r#"{"exports":{"./feature/*":{"types":"./types/*.d.mts"}}}"#,
    );
    write(
        &package.join("types/navigation.d.mts"),
        "export type Target = string;\n",
    );

    let resolved =
        resolve_specifier(&file_url(&source), "@scope/router/feature/navigation").unwrap();
    assert_eq!(
        resolved,
        package
            .join("types/navigation.d.mts")
            .canonicalize()
            .unwrap()
    );
}

#[test]
#[cfg(unix)]
fn resolves_symlinked_workspace_package_vue_export_to_authored_source() {
    let workspace = tempdir().unwrap();
    let app = workspace.path().join("app");
    let source = app.join("src/App.vue");
    write(&source, "<script setup lang=\"ts\"></script>");
    let package = workspace.path().join("packages/ui");
    write(
        &package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{"./widget":"./src/Widget.vue"}}"#,
    );
    let component = package.join("src/Widget.vue");
    write(&component, "<template />\n");
    link_package(&package, &app.join("node_modules/@scope/ui"));

    let resolved = resolve_specifier(&file_url(&source), "@scope/ui/widget").unwrap();
    assert_eq!(resolved, component.canonicalize().unwrap());
}

#[test]
fn component_definition_rejects_deleted_target_but_keeps_open_unsaved_target() {
    let dir = tempdir().unwrap();
    let component_path = dir.path().join("Deleted.vue");
    let source_path = dir.path().join("Parent.vue");
    let source = r#"<script setup lang="ts">
import Deleted from './Deleted.vue'
</script>
<template><Deleted /></template>
"#;
    let uri = Url::from_file_path(source_path).unwrap();
    let component_uri = Url::from_file_path(component_path).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);
    let offset = source.find("Deleted />").unwrap();
    let context = IdeContext::new(&state, &uri, offset).unwrap();

    assert!(DefinitionService::definition(&context).is_none());

    state.documents.open(
        component_uri.clone(),
        "<template />\n".to_string(),
        1,
        "vue".to_string(),
    );
    let GotoDefinitionResponse::Scalar(location) =
        DefinitionService::definition(&context).expect("open target definition")
    else {
        panic!("component definition must be a scalar location");
    };
    assert_eq!(location.uri, component_uri);
}

#[test]
fn resolves_relative_declarations_and_rejects_package_escape() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("src/App.vue");
    write(&source, "<script setup lang=\"ts\"></script>");
    write(
        &workspace.path().join("src/model.d.cts"),
        "export type Model = {};\n",
    );
    let package = workspace.path().join("node_modules/unsafe");
    write(
        &package.join("package.json"),
        r#"{"exports":{".":{"types":"../outside.d.ts"}}}"#,
    );
    write(
        &workspace.path().join("node_modules/outside.d.ts"),
        "export {};\n",
    );

    assert_eq!(
        resolve_specifier(&file_url(&source), "./model"),
        Some(
            workspace
                .path()
                .join("src/model.d.cts")
                .canonicalize()
                .unwrap()
        ),
    );
    assert_eq!(resolve_specifier(&file_url(&source), "unsafe"), None);
    assert_eq!(
        resolve_specifier(&file_url(&source), "C:/components/Foo.vue"),
        None
    );
    assert_eq!(
        resolve_specifier(&file_url(&source), r"C:\components\Foo.vue"),
        None
    );
}

#[test]
fn resolves_relative_imports_with_dotted_basenames() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("src/App.vue");
    let target = workspace.path().join("src/x.use.ts");
    write(&source, "<script setup lang=\"ts\"></script>");
    write(&target, "export const useX = () => 1;\n");

    assert_eq!(
        resolve_specifier(&file_url(&source), "./x.use"),
        Some(target.canonicalize().unwrap()),
    );
}

/// Native TypeScript answers a module-specifier definition with the whole
/// target file span. The editor contract is the module identity at its origin,
/// so the mapped answer must collapse back to a zero-width range.
#[cfg(feature = "native")]
#[test]
fn native_module_definitions_collapse_to_the_target_file_origin() {
    use tower_lsp::lsp_types::{Location, Position, Range};

    let uri = Url::parse("file:///pkg/dist/index.d.ts").unwrap();
    let whole_file = Location {
        uri: uri.clone(),
        range: Range::new(Position::new(0, 0), Position::new(53, 0)),
    };
    let pinned = super::pin_to_module_origin(whole_file);
    assert_eq!(pinned.uri, uri);
    assert_eq!(
        pinned.range,
        Range::new(Position::new(0, 0), Position::new(0, 0))
    );
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn file_url(path: &Path) -> Url {
    Url::from_file_path(path).unwrap()
}

#[cfg(unix)]
fn link_package(source: &Path, target: &Path) {
    crate::ide::tests::symlink_dir(source, target);
}
