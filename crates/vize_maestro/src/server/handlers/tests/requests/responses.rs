use super::*;
use tower_lsp::lsp_types::CompletionList;

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
    assert!(names.contains(&"template".to_string()));
    assert!(names.contains(&"script setup".to_string()));
    assert!(names.contains(&"style scoped".to_string()));
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

    assert!(labels.contains(&"template".to_string()));
    assert!(labels.contains(&"script setup".to_string()));
    assert!(labels.contains(&"style".to_string()));
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
