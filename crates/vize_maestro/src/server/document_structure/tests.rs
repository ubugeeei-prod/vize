use tower_lsp::lsp_types::{
    FoldingRangeParams, PartialResultParams, Position, SelectionRangeParams,
    TextDocumentIdentifier, Url, WorkDoneProgressParams,
};

use super::{folding_ranges, selection_ranges};
use crate::server::ServerState;

const SFC: &str = "<script setup lang=\"ts\">\nconst count = 1\n</script>\n\n<template>\n  <div class=\"wrap\">{{ count }}</div>\n</template>\n";

fn state_with(uri: &Url, source: &str) -> ServerState {
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state
}

fn folding_params(uri: Url) -> FoldingRangeParams {
    FoldingRangeParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn selection_params(uri: Url, positions: Vec<Position>) -> SelectionRangeParams {
    SelectionRangeParams {
        text_document: TextDocumentIdentifier { uri },
        positions,
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

#[test]
fn folding_ranges_cover_every_multi_line_block_in_discovery_order() {
    let uri = Url::parse("file:///App.vue").unwrap();
    let state = state_with(&uri, SFC);

    let ranges = folding_ranges(&state, &folding_params(uri)).expect("blocks must fold");
    let summary: Vec<(u32, u32, Option<String>)> = ranges
        .into_iter()
        .map(|range| (range.start_line, range.end_line, range.collapsed_text))
        .collect();

    assert_eq!(
        summary,
        vec![
            (4, 6, Some("template".to_string())),
            (0, 2, Some("script setup".to_string())),
        ]
    );
}

#[test]
fn selection_ranges_return_one_chain_per_position() {
    let uri = Url::parse("file:///App.vue").unwrap();
    let state = state_with(&uri, SFC);

    let chains = selection_ranges(
        &state,
        &selection_params(uri, vec![Position::new(5, 24), Position::new(1, 8)]),
    )
    .expect("both positions resolve");

    assert_eq!(chains.len(), 2);
    assert_eq!(chains[0].range.start, Position::new(5, 23));
    assert_eq!(chains[0].range.end, Position::new(5, 28));
    assert_eq!(chains[1].range.start, Position::new(1, 6));
    assert_eq!(chains[1].range.end, Position::new(1, 11));
}

#[test]
fn an_out_of_bounds_position_collapses_the_whole_response() {
    // The LSP contract pairs results with requested positions by index, so a
    // short array would silently misalign every later position in the request.
    let uri = Url::parse("file:///App.vue").unwrap();
    let state = state_with(&uri, SFC);

    let chains = selection_ranges(
        &state,
        &selection_params(uri, vec![Position::new(5, 24), Position::new(900, 0)]),
    );

    assert!(chains.is_none());
}

#[test]
fn an_unopened_document_yields_no_structure() {
    let state = ServerState::new();
    let uri = Url::parse("file:///Missing.vue").unwrap();

    assert!(folding_ranges(&state, &folding_params(uri.clone())).is_none());
    assert!(selection_ranges(&state, &selection_params(uri, vec![Position::new(0, 0)])).is_none());
}
