use std::path::Path;

use tempfile::tempdir;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use super::resolve_import_specifier;
use crate::{
    ide::{DefinitionService, IdeContext},
    server::ServerState,
};

#[test]
fn user_paths_win_over_a_same_named_package_for_sync_definition() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("src/App.vue");
    let local = workspace.path().join("src/LocalWidget.vue");
    let installed = workspace
        .path()
        .join("node_modules/@scope/ui/InstalledWidget.vue");
    let content = r#"<script setup lang="ts">
import Widget from '@scope/ui'
</script>
<template><Widget /></template>
"#;
    write(
        &workspace.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@scope/ui":["src/LocalWidget.vue"]}}}"#,
    );
    write(&source, content);
    write(
        &local,
        "<script setup lang=\"ts\">defineProps<{ localOnly: true }>()</script>\n",
    );
    write(
        &workspace.path().join("node_modules/@scope/ui/package.json"),
        r#"{"name":"@scope/ui","exports":{".":"./InstalledWidget.vue"}}"#,
    );
    write(
        &installed,
        "<script setup lang=\"ts\">defineProps<{ installedOnly: true }>()</script>\n",
    );

    let uri = Url::from_file_path(&source).unwrap();
    assert_eq!(
        resolve_import_specifier(&uri, "@scope/ui")
            .unwrap()
            .canonicalize()
            .unwrap(),
        local.canonicalize().unwrap()
    );

    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), content.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, content);
    let context = IdeContext::new(&state, &uri, content.find("Widget />").unwrap()).unwrap();
    let GotoDefinitionResponse::Scalar(location) =
        DefinitionService::definition(&context).expect("component definition")
    else {
        panic!("component definition must be scalar");
    };
    assert_eq!(
        location.uri.to_file_path().unwrap().canonicalize().unwrap(),
        local.canonicalize().unwrap()
    );
}

#[test]
fn nuxt_source_aliases_work_before_generated_tsconfig_exists() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("app/pages/accounts.vue");
    let component = workspace
        .path()
        .join("app/components/AccountSearchResult.vue");
    write(
        &workspace.path().join("tsconfig.json"),
        r#"{"references":[{"path":"./.nuxt/tsconfig.app.json"}],"files":[]}"#,
    );
    write(
        &workspace.path().join("nuxt.config.ts"),
        "export default defineNuxtConfig({})\n",
    );
    write(
        &source,
        r#"<script setup lang="ts">
import AccountSearchResult from '~/components/AccountSearchResult.vue'
</script>
<template><AccountSearchResult /></template>
"#,
    );
    write(&component, "<template />\n");

    let uri = Url::from_file_path(&source).unwrap();
    assert_eq!(
        resolve_import_specifier(&uri, "~/components/AccountSearchResult.vue")
            .unwrap()
            .canonicalize()
            .unwrap(),
        component.canonicalize().unwrap()
    );
}

#[test]
fn project_source_alias_resolves_nearest_src_root() {
    let workspace = tempdir().unwrap();
    let source = workspace.path().join("src/views/Docs.vue");
    let component = workspace.path().join("src/components/HighlightMessage.vue");
    write(
        &workspace.path().join("package.json"),
        r#"{"type":"module"}"#,
    );
    write(
        &source,
        r#"<script setup>
import HighlightMessage from '@/components/HighlightMessage.vue'
</script>
<template><highlight-message /></template>
"#,
    );
    write(&component, "<template />\n");

    let uri = Url::from_file_path(&source).unwrap();
    assert_eq!(
        resolve_import_specifier(&uri, "@/components/HighlightMessage.vue")
            .unwrap()
            .canonicalize()
            .unwrap(),
        component.canonicalize().unwrap()
    );
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
