use std::collections::HashMap;

use tower_lsp::lsp_types::{Location, TextEdit, Url, WorkspaceEdit};
use vize_canon::{LspLocation, LspPosition, LspRange};
use vize_l0::FxHashMap;

use super::value_expression_ranges;
use crate::ide::{DiagnosticService, IdeContext, corsa_support::CanonicalVirtualDocument};
use crate::server::ServerState;

#[test]
fn native_expression_geometry_excludes_contextual_and_type_keys() {
    let source =
        "type Props = { id: string }; const id = 1; const o = { id: id }; o.id; o['id']; ({ id });";
    let ranges = value_expression_ranges(source, false).expect("valid TypeScript");
    for (needle, expected) in [
        ("id: string", false),
        ("id = 1", false),
        ("id: id", false),
        ("id };", true),
        ("id;", true),
        ("'id'", true),
        ("id });", true),
    ] {
        let offset = source.find(needle).expect("role marker");
        assert_eq!(
            ranges.iter().any(|range| range.contains(&offset)),
            expected,
            "{needle}"
        );
    }
    assert!(value_expression_ranges("type Props = {", false).is_none());
    assert!(value_expression_ranges("const view = <div>{value}</div>;", true).is_some());
}

#[test]
fn only_selected_native_value_geometry_adds_the_second_authored_role() {
    let source = "<script setup lang=\"ts\">defineProps<{ id: string }>();</script>\n<template><Recursive :id /></template>";
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Recursive.vue").expect("URI");
    let ctx = IdeContext::testing(&state, &uri, 0, source.into());
    let document = CanonicalVirtualDocument {
        source_uri: uri.clone(),
        request_uri: "file:///workspace/Recursive.vue.ts".into(),
        virtual_result: DiagnosticService::generate_virtual_ts(&uri, source, false, false)
            .expect("generated document"),
        dependencies: Vec::new(),
        materialized_sources: Vec::new(),
        session_project_roots: Vec::new(),
        source_catalogs: Vec::new(),
    };
    let start = source.find(":id").expect("shorthand") + 1;
    let (line, character) = crate::ide::offset_to_position(source, start);
    let argument = Location {
        uri,
        range: tower_lsp::lsp_types::Range::new(
            tower_lsp::lsp_types::Position::new(line, character),
            tower_lsp::lsp_types::Position::new(line, character + 2),
        ),
    };
    let generated = &document.virtual_result.code;
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let positive = value_expression_ranges(generated, false).expect("generated syntax");
    for (start, _) in generated.match_indices("id") {
        let (line, character) = crate::ide::offset_to_position(generated, start);
        let (end_line, end_character) = crate::ide::offset_to_position(generated, start + 2);
        let native = LspLocation {
            uri: document.request_uri.to_string(),
            range: LspRange {
                start: LspPosition { line, character },
                end: LspPosition {
                    line: end_line,
                    character: end_character,
                },
            },
        };
        if crate::ide::corsa_support::map_canonical_corsa_location(&ctx, &document, &native)
            .as_ref()
            != Some(&argument)
        {
            continue;
        }
        let edit = TextEdit {
            range: tower_lsp::lsp_types::Range::new(
                tower_lsp::lsp_types::Position::new(line, character),
                tower_lsp::lsp_types::Position::new(end_line, end_character),
            ),
            new_text: "field".into(),
        };
        if positive
            .iter()
            .any(|range| range.start <= start && start + 2 <= range.end)
        {
            values.push(edit);
        } else {
            keys.push(edit);
        }
    }
    assert!(
        !keys.is_empty(),
        "public key projection must exist: {generated}"
    );
    assert!(
        !values.is_empty(),
        "native value projection must exist: {generated}"
    );
    let mut cache = FxHashMap::default();
    for (edits, expected) in [(keys, Vec::new()), (values, vec![argument.clone()])] {
        let edit = WorkspaceEdit {
            changes: Some(HashMap::from([(
                Url::parse(&document.request_uri).expect("virtual URI"),
                edits,
            )])),
            ..Default::default()
        };
        let mut selected = document
            .selected_shorthand_value_arguments(
                &ctx,
                &edit,
                std::slice::from_ref(&argument),
                &mut cache,
            )
            .expect("resolved roles");
        selected.dedup();
        assert_eq!(selected, expected);
    }
}
