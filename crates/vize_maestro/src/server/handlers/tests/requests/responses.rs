use super::*;
use tower_lsp::lsp_types::CompletionList;

#[cfg(feature = "native")]
#[test]
fn exact_navigation_cache_hits_do_not_wait_for_the_native_transaction() {
    crate::runtime::block_on(async {
        let service = service_with_options(serde_json::json!({
            "definition": true,
            "references": true,
            "typecheck": true,
            "crossFile": true,
            "lint": false,
            "ecosystem": false,
        }));
        let server = service.inner();
        let uri = uri("CachedNavigation.vue");
        open_vue(server, &uri, SAMPLE);
        server.state.set_corsa_disk_watcher_registered();

        let position = Position::new(0, 0);
        let location = Location::new(uri.clone(), range());
        let definition = Some(DefResponse::Scalar(location.clone()));
        let references = Some(vec![location]);
        let stamp = server.state.corsa_request_stamp();
        server
            .state
            .cache_native_definition(&uri, position, stamp, definition.clone());
        server
            .state
            .cache_native_references(&uri, position, true, stamp, references.clone());

        let _native = server.state.corsa_request_scope().await;
        let returned_definition = crate::runtime::timeout(
            std::time::Duration::from_secs(3),
            server.goto_definition(definition_params(&uri)),
        )
        .await
        .expect("an exact definition cache hit must not wait for the held transaction")
        .unwrap();
        assert_eq!(returned_definition, definition);

        let returned_references = crate::runtime::timeout(
            std::time::Duration::from_secs(3),
            server.references(references_params(&uri)),
        )
        .await
        .expect("an exact references cache hit must not wait for the held transaction")
        .unwrap();
        assert_eq!(returned_references, references);
    });
}

#[test]
fn references_preserve_structural_results_without_starting_a_disabled_checker() {
    for cross_file in [false, true] {
        let service =
            service_with_options(options(&[("references", true), ("crossFile", cross_file)]));
        let server = service.inner();
        let uri = uri("References.vue");
        open_vue(server, &uri, SAMPLE);
        for include_declaration in [true, false] {
            let mut params = references_params(&uri);
            params.text_document_position.position = Position::new(1, 10);
            params.context.include_declaration = include_declaration;
            let mut expected = vec![Location::new(
                uri.clone(),
                Range::new(Position::new(1, 10), Position::new(1, 17)),
            )];
            if include_declaration {
                expected.push(Location::new(
                    uri.clone(),
                    Range::new(Position::new(4, 6), Position::new(4, 13)),
                ));
            }
            assert_eq!(
                futures::executor::block_on(server.references(params)).unwrap(),
                Some(expected)
            );
        }
        #[cfg(feature = "native")]
        {
            // Unmask the bridge-presence accessor without initializing the bridge.
            server
                .state
                .apply_lsp_initialization_options(Some(&serde_json::json!({"typecheck": true})));
            assert!(!server.state.has_corsa_bridge());
            assert_eq!(server.state.corsa_init_failure(), None);
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn disabled_references_do_not_start_an_enabled_checker() {
    let service = service_with_options(serde_json::json!({
        "references": false, "typecheck": true, "crossFile": false
    }));
    let server = service.inner();
    let uri = uri("DisabledReferences.vue");
    open_vue(server, &uri, SAMPLE);
    let mut params = references_params(&uri);
    params.text_document_position.position = Position::new(1, 10);
    assert_eq!(
        futures::executor::block_on(server.references(params)).unwrap(),
        None
    );
    assert!(!server.state.has_corsa_bridge());
    assert_eq!(server.state.corsa_init_failure(), None);
}

#[cfg(feature = "native")]
#[test]
fn private_setup_references_keep_exact_local_ranges_without_initializing_a_checker() {
    for (name, initializer) in [
        ("item", "computed(() => 1)"),
        ("route", "computed(() => 1)"),
        ("itemId", "computed(() => 1)"),
        ("item", "1"),
    ] {
        for newline in ["\n", "\r\n"] {
            for cross_file in [false, true] {
                let service = service_with_options(serde_json::json!({
                    "references": true, "typecheck": true, "crossFile": cross_file
                }));
                let server = service.inner();
                let uri = uri("PrivateReferences.vue");
                let source = "<!-- 日本 🧭 -->\n<script setup lang=\"ts\">\nimport { computed } from 'vue';\nconst PRIVATE = INIT;\nfunction shadow(PRIVATE: number) { return PRIVATE; }\n</script>\n<template><p>🧭{{ PRIVATE }}</p></template>\n<style>.c{color:v-bind(PRIVATE)}</style>\n"
                    .replace("INIT", initializer)
                    .replace("PRIVATE", name)
                    .replace('\n', newline);
                open_vue(server, &uri, &source);
                for include_declaration in [false, true] {
                    let mut params = references_params(&uri);
                    params.text_document_position.position = Position::new(3, 6);
                    params.context.include_declaration = include_declaration;
                    let length = name.len() as u32;
                    let mut expected = vec![
                        Location::new(
                            uri.clone(),
                            Range::new(Position::new(6, 18), Position::new(6, 18 + length)),
                        ),
                        Location::new(
                            uri.clone(),
                            Range::new(Position::new(7, 23), Position::new(7, 23 + length)),
                        ),
                    ];
                    if include_declaration {
                        expected.insert(
                            0,
                            Location::new(
                                uri.clone(),
                                Range::new(Position::new(3, 6), Position::new(3, 6 + length)),
                            ),
                        );
                    }
                    assert_eq!(
                        futures::executor::block_on(server.references(params)).unwrap(),
                        Some(expected)
                    );
                    assert!(!server.state.has_corsa_bridge());
                    assert_eq!(server.state.corsa_init_failure(), None);
                }
            }
        }
    }
}

#[test]
fn root_completion_returns_block_snippet_labels() {
    let service = service_with_options(options(&[("completion", true)]));
    let server = service.inner();
    let uri = uri("Empty.vue");
    open_vue(server, &uri, "\n");

    let response = futures::executor::block_on(server.completion(completion_params(&uri)))
        .unwrap()
        .expect("root completion should return block snippets");
    let labels: Vec<_> = match response {
        CompletionResponse::Array(items) => items.into_iter().map(|item| item.label).collect(),
        CompletionResponse::List(CompletionList { items, .. }) => {
            items.into_iter().map(|item| item.label).collect()
        }
    };

    for label in [
        "template",
        "script setup",
        "script",
        "style scoped",
        "style",
    ] {
        assert!(
            labels.iter().any(|item| item == label),
            "missing {label}: {labels:?}"
        );
    }
}

#[cfg(feature = "native")]
#[test]
fn component_prop_colon_completion_does_not_wait_for_a_type_project() {
    let root = tempfile::tempdir().unwrap();
    let dialog = root.path().join("dialog");
    let text_field = root.path().join("text-field");
    std::fs::create_dir_all(&dialog).unwrap();
    std::fs::create_dir_all(&text_field).unwrap();
    std::fs::write(
        text_field.join("afs-text-field.vue"),
        r#"<script setup lang="ts">
defineProps<{
  modelValue: string
  placeholder?: string
  type?: string
  appendIcon?: string
  rules?: unknown[]
}>()
</script>
"#,
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
import AfsTextField from "../text-field/afs-text-field.vue";
</script>
<template>
  <label>Password <span>*</span></label
  >
  <AfsTextField
    : />
</template>
"#;
    let parent = dialog.join("reported.vue");
    std::fs::write(&parent, source).unwrap();
    let uri = Url::from_file_path(parent).unwrap();
    let service = service_with_options(serde_json::json!({
        "completion": true,
        "typecheck": true,
        "crossFile": true,
        "lint": false,
        "ecosystem": false,
    }));
    let server = service.inner();
    server.state.set_workspace_root(root.path().to_path_buf());
    open_vue(server, &uri, source);

    let offset = source.find("    : />").unwrap() + "    :".len();
    let (line, character) = crate::ide::offset_to_position(source, offset);
    let mut params = completion_params(&uri);
    params.text_document_position.position = Position::new(line, character);
    params.context = Some(tower_lsp::lsp_types::CompletionContext {
        trigger_kind: tower_lsp::lsp_types::CompletionTriggerKind::TRIGGER_CHARACTER,
        trigger_character: Some(":".into()),
    });
    let response = futures::executor::block_on(server.completion(params))
        .unwrap()
        .expect("component prop completion after colon");
    let labels: Vec<_> = match response {
        CompletionResponse::Array(items) => items.into_iter().map(|item| item.label).collect(),
        CompletionResponse::List(CompletionList { items, .. }) => {
            items.into_iter().map(|item| item.label).collect()
        }
    };

    for prop in ["modelValue", "placeholder", "type", "appendIcon", "rules"] {
        assert!(
            labels.iter().any(|label| label == prop),
            "missing {prop}: {labels:?}"
        );
    }
    assert_eq!(server.state.canonical_bridge_request_count(), 0);
    assert!(!server.state.has_corsa_bridge());
}

#[test]
fn script_symbol_hover_survives_the_reported_multiline_template_shape() {
    let source = r#"<script setup lang="ts">
import { ref } from "vue";
const inputPassword = ref("");
</script>
<template>
  <label>Password <span>*</span></label
  >
  <AfsTextField v-model="inputPassword" />
</template>
"#;
    let service = service_with_options(options(&[("hover", true)]));
    let server = service.inner();
    let uri = uri("ReportedHover.vue");
    open_vue(server, &uri, source);
    let offset = source.find("inputPassword").unwrap() + 2;
    let (line, character) = crate::ide::offset_to_position(source, offset);
    let mut params = hover_params(&uri);
    params.text_document_position_params.position = Position::new(line, character);

    let hover = futures::executor::block_on(server.hover(params))
        .unwrap()
        .expect("script binding hover");
    assert_eq!(hover.range, None);
    let tower_lsp::lsp_types::HoverContents::Markup(markup) = hover.contents else {
        panic!("expected markdown script binding hover");
    };
    assert_eq!(markup.kind, tower_lsp::lsp_types::MarkupKind::Markdown);
    assert_eq!(
        markup.value,
        "**inputPassword**\n\n_Script binding_\n\n```typescript\ninputPassword: Ref<string>\n```\n\nReactive reference created with `ref()`. Access `.value` in script, auto-unwrapped in template.\n\n**Tip**\n\nUse `inputPassword.value` to read or write this ref in script."
    );
}

#[test]
fn document_symbols_list_sfc_blocks_with_editor_labels() {
    let service = service_with_options(options(&[("documentSymbols", true)]));
    let server = service.inner();
    let uri = uri("Symbols.vue");
    open_vue(server, &uri, SAMPLE);

    let response =
        futures::executor::block_on(server.document_symbol(document_symbol_params(&uri)))
            .unwrap()
            .expect("document symbols should be available");
    let DocumentSymbolResponse::Nested(symbols) = response else {
        panic!("expected nested document symbols");
    };
    let names: Vec<_> = symbols.into_iter().map(|symbol| symbol.name).collect();
    assert_eq!(names, ["template", "script setup", "style scoped"]);
}

#[test]
fn document_symbol_ranges_still_cover_multiline_block_tags() {
    let service = service_with_options(options(&[("documentSymbols", true)]));
    let server = service.inner();
    let uri = uri("MultilineSymbol.vue");
    open_vue(
        server,
        &uri,
        "<template\n  lang=\"html\"\n>\nx\n</template>\n",
    );

    let response =
        futures::executor::block_on(server.document_symbol(document_symbol_params(&uri)))
            .unwrap()
            .expect("document symbols should be available");
    let DocumentSymbolResponse::Nested(symbols) = response else {
        panic!("expected nested document symbols");
    };

    assert_eq!(symbols.len(), 1);
    assert_eq!(
        (symbols[0].range, symbols[0].selection_range),
        (
            Range::new(Position::new(0, 0), Position::new(4, 11)),
            Range::new(Position::new(0, 1), Position::new(0, 9)),
        )
    );
}

#[test]
fn folding_ranges_cover_multiline_sfc_blocks() {
    let service = service_with_options(options(&[("foldingRanges", true)]));
    let server = service.inner();
    let uri = uri("Folds.vue");
    open_vue(server, &uri, SAMPLE);

    let ranges = futures::executor::block_on(server.folding_range(folding_range_params(&uri)))
        .unwrap()
        .expect("folding ranges should be available");
    let labels: Vec<_> = ranges
        .into_iter()
        .filter_map(|range| range.collapsed_text)
        .collect();

    assert_eq!(labels, ["template", "script setup", "style"]);
}

fn diagnostic_hover_fixture() -> serde_json::Value {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/lint-hover-severity/responses.authored.json"
    )))
    .expect("authored complete diagnostic-hover responses");
    assert_eq!(
        fixture["fixCommit"],
        "ef8c3e059ed645e319154d475ef6d994dc489ee6"
    );
    assert_eq!(fixture["documentVersion"], 1);
    fixture
}

fn diagnostic_hover_service(lint: bool) -> tower_lsp::LspService<MaestroServer> {
    let service = service_with_options(serde_json::json!({
        "hover": true, "lint": lint, "typecheck": false, "ecosystem": false
    }));
    open_vue(
        service.inner(),
        &uri("DiagnosticHover.vue"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/lint-hover-severity/App.vue.txt"
        )),
    );
    service
}

fn diagnostic_hover_response(server: &MaestroServer, position: Position) -> serde_json::Value {
    let mut params = hover_params(&uri("DiagnosticHover.vue"));
    params.text_document_position_params.position = position;
    serde_json::to_value(futures::executor::block_on(server.hover(params)).unwrap())
        .expect("complete public Hover serialization")
}

#[test]
fn diagnostic_hover_history_preserves_all_five_whole_severity_responses() {
    let fixture = diagnostic_hover_fixture();
    let rows = fixture["severities"]
        .as_array()
        .expect("five severity rows");
    assert_eq!(rows.len(), 5);
    let position: Position =
        serde_json::from_value(fixture["position"].clone()).expect("authored query");
    for (row, severity) in rows.iter().zip([
        serde_json::json!(1),
        serde_json::json!(2),
        serde_json::json!(3),
        serde_json::json!(4),
        serde_json::Value::Null,
    ]) {
        assert_eq!(row["severity"], severity);
        let service = diagnostic_hover_service(true);
        let server = service.inner();
        let mut input = fixture["diagnostic"].clone();
        input["severity"] = row["severity"].clone();
        let diagnostic = serde_json::from_value(input).expect("authored typed diagnostic");
        server
            .state
            .cache_lint_hover_diagnostics(&uri("DiagnosticHover.vue"), 1, &[diagnostic]);
        assert_eq!(diagnostic_hover_response(server, position), row["result"]);
    }
}

#[test]
fn diagnostic_hover_history_keeps_whole_mixed_source_order_and_authored_text() {
    let fixture = diagnostic_hover_fixture();
    let service = diagnostic_hover_service(true);
    let server = service.inner();
    let first =
        serde_json::from_value(fixture["diagnostic"].clone()).expect("first authored diagnostic");
    let mut second = fixture["diagnostic"].clone();
    second["severity"] = serde_json::json!(4);
    second["code"] = serde_json::json!("musea/second-marker");
    second["source"] = serde_json::json!("vize/musea");
    second["message"] = serde_json::json!("Second problem — keep Unicode.");
    second.as_object_mut().unwrap().remove("codeDescription");
    let second = serde_json::from_value(second).expect("second authored diagnostic");
    server
        .state
        .cache_lint_hover_diagnostics(&uri("DiagnosticHover.vue"), 1, &[first, second]);
    let position = serde_json::from_value(fixture["position"].clone()).unwrap();
    assert_eq!(
        diagnostic_hover_response(server, position),
        fixture["orderedResult"]
    );
}

#[test]
fn diagnostic_hover_history_rejects_disabled_foreign_outside_and_stale_inputs() {
    let fixture = diagnostic_hover_fixture();
    let document = uri("DiagnosticHover.vue");
    let position = serde_json::from_value(fixture["position"].clone()).unwrap();
    for (lint, source, query, cache_version) in [
        (false, "vize/lint", position, 1),
        (true, "typescript", position, 1),
        (true, "vize/lint", Position::new(1, 2), 1),
        (true, "vize/lint", position, 2),
    ] {
        let service = diagnostic_hover_service(lint);
        let server = service.inner();
        let mut input = fixture["diagnostic"].clone();
        input["source"] = serde_json::json!(source);
        server.state.cache_lint_hover_diagnostics(
            &document,
            cache_version,
            &[serde_json::from_value(input).unwrap()],
        );
        assert_eq!(
            diagnostic_hover_response(server, query),
            serde_json::Value::Null
        );
    }

    let service = diagnostic_hover_service(true);
    let server = service.inner();
    let diagnostic = serde_json::from_value(fixture["diagnostic"].clone()).unwrap();
    server
        .state
        .cache_lint_hover_diagnostics(&document, 1, &[diagnostic]);
    let repaired = "<script setup lang=\"ts\">\n// diagnostic repair\n</script>\n";
    server
        .state
        .documents
        .open(document.clone(), repaired.to_string(), 2, "vue".to_string());
    server.state.update_virtual_docs(&document, repaired);
    assert_eq!(
        diagnostic_hover_response(server, position),
        serde_json::Value::Null
    );
    let diagnostic = serde_json::from_value(fixture["diagnostic"].clone()).unwrap();
    server
        .state
        .cache_lint_hover_diagnostics(&document, 2, &[diagnostic]);
    let first = fixture["severities"].as_array().unwrap().first().unwrap();
    assert_eq!(diagnostic_hover_response(server, position), first["result"]);
    #[cfg(feature = "native")]
    {
        // Expose the existing owner without starting it via a request.
        server
            .state
            .apply_lsp_initialization_options(Some(&serde_json::json!({ "typecheck": true })));
        assert!(!server.state.has_corsa_bridge());
        assert_eq!(server.state.corsa_init_failure(), None);
    }
}
