use std::fs;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

use crate::ide::IdeContext;
use crate::server::ServerState;

#[test]
fn component_definition_resolves_pug_kebab_import_tag() {
    let workspace = tempfile::tempdir().expect("temporary workspace");
    let target_path = workspace.path().join("HighlightMessage.vue");
    fs::write(&target_path, "<template><p /></template>\n").expect("component");

    let importer_path = workspace.path().join("App.vue");
    let source = r#"<script setup lang="ts">
import HighlightMessage from "./HighlightMessage.vue";
</script>
<template lang="pug">
  highlight-message(type="success")
</template>
"#;
    fs::write(&importer_path, source).expect("importer");

    let importer_uri = Url::from_file_path(&importer_path).expect("importer URI");
    let state = ServerState::new();
    state
        .documents
        .open(importer_uri.clone(), source.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&importer_uri, source);
    let offset = source.rfind("highlight-message").expect("component tag");
    let ctx = IdeContext::new(&state, &importer_uri, offset).expect("IDE context");
    let definition = super::component_tag_definition(&ctx).expect("component definition");
    let GotoDefinitionResponse::Scalar(location) = definition else {
        panic!("component definition must be scalar");
    };

    assert_eq!(
        location
            .uri
            .to_file_path()
            .expect("definition path")
            .canonicalize()
            .expect("canonical definition path"),
        target_path.canonicalize().expect("canonical target path")
    );
}
