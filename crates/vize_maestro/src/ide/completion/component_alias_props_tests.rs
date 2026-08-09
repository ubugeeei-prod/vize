use std::fs;

use tower_lsp::lsp_types::{CompletionResponse, Url};

use super::CompletionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn template_component_prop_completion_resolves_nuxt_reference_aliases() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let nuxt = workspace.path().join(".nuxt");
    let pages = workspace.path().join("app/pages");
    let components = workspace.path().join("app/components");
    fs::create_dir_all(&nuxt).expect("Nuxt directory");
    fs::create_dir_all(&pages).expect("pages directory");
    fs::create_dir_all(&components).expect("components directory");
    fs::write(
        workspace.path().join("tsconfig.json"),
        r#"{"references":[{"path":"./.nuxt/tsconfig.app.json"}],"files":[]}"#,
    )
    .expect("solution config");
    fs::write(
        nuxt.join("tsconfig.app.json"),
        r#"{"compilerOptions":{"paths":{"~/*":["../app/*"]}}}"#,
    )
    .expect("Nuxt app config");
    fs::write(
        components.join("AccountSearchResult.vue"),
        r#"<script setup lang="ts">
defineProps<{ result: unknown; active: boolean }>()
</script>
"#,
    )
    .expect("component");

    let source = r#"<script setup lang="ts">
import AccountSearchResult from '~/components/AccountSearchResult.vue'
</script>
<template><AccountSearchResult  /></template>
"#;
    let importer_path = pages.join("accounts.vue");
    fs::write(&importer_path, source).expect("importer");
    let uri = Url::from_file_path(&importer_path).expect("importer URI");
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&uri, source);

    let offset = source.find("<AccountSearchResult  />").unwrap() + "<AccountSearchResult ".len();
    let ctx = IdeContext::new(&state, &uri, offset).expect("IDE context");
    let labels = match CompletionService::complete(&ctx).expect("completion") {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
    .into_iter()
    .map(|item| item.label)
    .collect::<Vec<_>>();

    assert!(labels.iter().any(|label| label == "result"), "{labels:?}");
    assert!(labels.iter().any(|label| label == "active"), "{labels:?}");
}
