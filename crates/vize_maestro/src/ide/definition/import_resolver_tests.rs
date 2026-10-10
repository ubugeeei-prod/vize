use std::path::Path;

use tempfile::tempdir;
use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use super::{resolve_import_specifier, resolve_import_specifier_with_documents};
use crate::{
    document::DocumentStore,
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

#[test]
fn project_alias_open_targets_do_not_change_filesystem_only_resolution() {
    let workspace = tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let source = root.join("Parent.vue");
    let target = root.join("components/子 [赤] #.vue");
    write(
        &root.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@ui/*":["./components/*"]}}}"#,
    );
    let uri = Url::from_file_path(source).unwrap();
    let target_uri = Url::from_file_path(&target).unwrap();
    let documents = DocumentStore::new();
    documents.open(target_uri.clone(), "<template />\n".into(), 1, "vue".into());

    assert!(!target.exists());
    assert_eq!(resolve_import_specifier(&uri, "@ui/子 [赤] #.vue"), None);
    assert_eq!(
        resolve_import_specifier_with_documents(&uri, "@ui/子 [赤] #.vue", &documents),
        Some(target.clone()),
    );
    documents.close(&target_uri);
    assert_eq!(
        resolve_import_specifier_with_documents(&uri, "@ui/子 [赤] #.vue", &documents),
        None,
    );
    assert!(!target.exists());
}

#[test]
fn open_alias_targets_keep_target_array_order_and_saved_fallbacks() {
    let workspace = tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let first = root.join("preferred/Child.vue");
    let fallback = root.join("components/Child.vue");
    write(
        &root.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@ui/*":["./preferred/*","./components/*"]}}}"#,
    );
    write(&fallback, "<template />\n");
    let uri = Url::from_file_path(root.join("Parent.vue")).unwrap();
    let first_uri = Url::from_file_path(&first).unwrap();
    let documents = DocumentStore::new();
    documents.open(first_uri.clone(), "<template />\n".into(), 1, "vue".into());

    assert_eq!(
        resolve_import_specifier(&uri, "@ui/Child.vue"),
        Some(fallback.clone())
    );
    assert_eq!(
        resolve_import_specifier_with_documents(&uri, "@ui/Child.vue", &documents),
        Some(first.clone()),
    );
    documents.close(&first_uri);
    assert_eq!(
        resolve_import_specifier_with_documents(&uri, "@ui/Child.vue", &documents),
        Some(fallback),
    );
    assert!(!first.exists());
}

#[test]
fn alias_candidate_extensions_keep_the_existing_priority() {
    let workspace = tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    write(
        &root.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@ui/*":["./components/*"]}}}"#,
    );
    let ts = root.join("components/Child.ts");
    let vue = root.join("components/Child.vue");
    write(&ts, "export default {};\n");
    let uri = Url::from_file_path(root.join("Parent.vue")).unwrap();
    let vue_uri = Url::from_file_path(&vue).unwrap();
    let documents = DocumentStore::new();
    documents.open(vue_uri, "<template />\n".into(), 1, "vue".into());
    assert_eq!(
        resolve_import_specifier_with_documents(&uri, "@ui/Child", &documents),
        Some(ts.clone()),
    );

    // All saved candidates retain authority before any open-only candidate.
    std::fs::remove_file(&ts).unwrap();
    write(&vue, "<template />\n");
    documents.open(
        Url::from_file_path(&ts).unwrap(),
        "export default {};\n".into(),
        1,
        "typescript".into(),
    );
    assert_eq!(
        resolve_import_specifier_with_documents(&uri, "@ui/Child", &documents),
        Some(vue.clone()),
    );
    std::fs::remove_file(&vue).unwrap();
    assert_eq!(
        resolve_import_specifier_with_documents(&uri, "@ui/Child", &documents),
        Some(ts),
    );
}

#[test]
fn a_saved_later_extension_performs_zero_open_file_lookups() {
    let workspace = tempdir().unwrap();
    let root = workspace.path().canonicalize().unwrap();
    let base = root.join("components/Child");
    let saved = base.with_extension("vue");
    write(&saved, "<template />\n");
    let open_ts = base.with_extension("ts");
    let lookups = std::cell::Cell::new(0);
    let result = super::probe_with_open_files(&base, |candidate| {
        lookups.set(lookups.get() + 1);
        // URI construction belongs entirely to this disk-miss callback.
        let _uri = Url::from_file_path(candidate).unwrap();
        (candidate == open_ts).then(|| open_ts.clone())
    });
    assert_eq!(result, Some(saved));
    assert_eq!(lookups.get(), 0);
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
