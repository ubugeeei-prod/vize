use tower_lsp::lsp_types::{NumberOrString, Url};

use super::{DiagnosticService, sources};
use crate::server::ServerState;

fn state_with_lsp_diagnostics(lint: bool, typecheck: bool) -> ServerState {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "lint": lint,
        "typecheck": typecheck
    })));
    state
}

#[test]
fn collect_surfaces_fallthrough_attrs_on_authored_template_root() {
    let state = state_with_lsp_diagnostics(false, true);
    let uri = Url::parse("file:///MultiRoot.vue").unwrap();
    state.documents.open(
        uri.clone(),
        r#"<script setup lang="ts">
const marker = 1;
</script>

<template>
  <header :class="$attrs.class">top</header>
  <main>body</main>
</template>
"#
        .to_string(),
        1,
        "vue".to_string(),
    );

    let diagnostics = DiagnosticService::collect(&state, &uri);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code == Some(NumberOrString::String("fallthrough-attrs".to_string()))
        })
        .expect("fallthrough attrs diagnostic should be present");

    assert_eq!(diagnostic.source.as_deref(), Some(sources::TYPE_CHECKER));
    assert_eq!(diagnostic.range.start.line, 5);
    assert_eq!(diagnostic.range.start.character, 2);
    assert!(
        diagnostic.message.contains("Multi-root component"),
        "{diagnostic:#?}"
    );
}

#[test]
fn collect_keeps_plain_fragments_diagnostic_free() {
    let state = state_with_lsp_diagnostics(false, true);
    let uri = Url::parse("file:///PlainFragment.vue").unwrap();
    state.documents.open(
        uri.clone(),
        r#"<script setup lang="ts">
const marker = 1;
</script>

<template>
  <header>top</header>
  <main>body</main>
</template>
"#
        .to_string(),
        1,
        "vue".to_string(),
    );

    let diagnostics = DiagnosticService::collect(&state, &uri);

    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.code != Some(NumberOrString::String("fallthrough-attrs".to_string()))
        }),
        "plain Vue 3 fragments should not warn until attrs are observed: {diagnostics:#?}"
    );
}

#[test]
fn collect_suppresses_fallthrough_attrs_when_inherit_attrs_is_false() {
    let state = state_with_lsp_diagnostics(false, true);
    let uri = Url::parse("file:///IntentionalFragment.vue").unwrap();
    state.documents.open(
        uri.clone(),
        r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>

<template>
  <header :class="$attrs.class">top</header>
  <main>body</main>
</template>
"#
        .to_string(),
        1,
        "vue".to_string(),
    );

    let diagnostics = DiagnosticService::collect(&state, &uri);

    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.code != Some(NumberOrString::String("fallthrough-attrs".to_string()))
        }),
        "inheritAttrs: false should keep intentional fragments diagnostic-free: {diagnostics:#?}"
    );
}
