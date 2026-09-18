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
