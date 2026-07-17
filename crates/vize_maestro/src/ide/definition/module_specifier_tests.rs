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
fn resolves_hoisted_package_types_export() {
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
}

fn write(path: &Path, content: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn file_url(path: &Path) -> Url {
    Url::from_file_path(path).unwrap()
}
