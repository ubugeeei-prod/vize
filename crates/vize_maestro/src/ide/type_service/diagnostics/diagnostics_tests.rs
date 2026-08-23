use super::{TypeService, offset_to_line_col};
use crate::server::ServerState;
use tower_lsp::lsp_types::{DiagnosticSeverity, NumberOrString, Position, Url};

#[test]
fn offset_to_line_col_is_zero_indexed_for_lsp() {
    assert_eq!(offset_to_line_col("one\ntwo", 0), (0, 0));
    assert_eq!(offset_to_line_col("one\ntwo", 4), (1, 0));
    assert_eq!(offset_to_line_col("one\ntwo", 6), (1, 2));
}

#[test]
fn offset_to_line_col_counts_utf16_code_units() {
    let source = "const icon = \"😀\"; missing";
    let offset = source.find("missing").unwrap();

    assert_eq!(offset_to_line_col(source, offset), (0, 19));
}

#[test]
fn collect_diagnostics_uses_zero_indexed_lsp_lines() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Component.vue").unwrap();
    state.documents.open(
        uri.clone(),
        "<script setup>\nconst props = defineProps(['count'])\n</script>\n<template>{{ props.count }}</template>".to_string(),
        1,
        "vue".to_string(),
    );

    let diagnostics = TypeService::collect_diagnostics(&state, &uri);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_ref().is_some_and(|code| {
            matches!(code, tower_lsp::lsp_types::NumberOrString::String(value) if value == "untyped-prop")
        }))
        .expect("untyped prop diagnostic should be present");

    assert_eq!(diagnostic.range.start.line, 1);
    assert_eq!(diagnostic.severity, Some(DiagnosticSeverity::WARNING));
}

#[test]
fn collect_diagnostics_anchors_fallthrough_attrs_to_template_root() {
    let state = ServerState::new();
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

    let diagnostics = TypeService::collect_diagnostics(&state, &uri);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code == Some(NumberOrString::String("fallthrough-attrs".to_string()))
        })
        .expect("fallthrough attrs diagnostic should be present");

    assert_eq!(
        diagnostic.range.start,
        Position {
            line: 5,
            character: 2
        }
    );
}

#[test]
fn collect_diagnostics_keeps_plain_fragments_diagnostic_free() {
    let state = ServerState::new();
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

    let diagnostics = TypeService::collect_diagnostics(&state, &uri);

    assert!(
        diagnostics.iter().all(|diagnostic| {
            diagnostic.code != Some(NumberOrString::String("fallthrough-attrs".to_string()))
        }),
        "plain Vue 3 fragments should not warn until attrs are observed: {diagnostics:#?}"
    );
}

#[test]
fn collect_diagnostics_does_not_report_regex_literals_as_undefined_bindings() {
    let state = ServerState::new();
    let uri = Url::parse("file:///RegexLiteral.vue").unwrap();
    state.documents.open(
        uri.clone(),
        r#"<script setup lang="ts">
const message = 'hello'
const bar = 'baz'
</script>
<template>
  {{ message.match(/foo/) }}
  {{ /foo/.test(message) }}
  {{ message.replace(/foo/g, bar) }}
</template>"#
            .to_string(),
        1,
        "vue".to_string(),
    );

    let diagnostics = TypeService::collect_diagnostics(&state, &uri);

    assert!(
        diagnostics
            .iter()
            .filter(|diagnostic| matches!(
                diagnostic.code,
                Some(NumberOrString::String(ref code)) if code == "undefined-binding"
            ))
            .collect::<Vec<_>>()
            .is_empty(),
        "regex literals should not produce undefined-binding diagnostics: {diagnostics:#?}"
    );
}
