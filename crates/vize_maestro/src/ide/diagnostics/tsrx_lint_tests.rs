//! `.tsrx` documents get the one TSRX lint entry, never the SFC path.

use super::{DiagnosticService, sources};
use crate::server::ServerState;
use tower_lsp::lsp_types::{NumberOrString, Position, Url};

fn state(lint: bool) -> ServerState {
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({ "lint": lint })));
    state
}

fn collect(state: &ServerState, source: &str) -> Vec<(String, Position, Position)> {
    let uri = Url::parse("file:///project/List.tsrx").unwrap();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "tsrx".to_string());
    DiagnosticService::collect(state, &uri)
        .into_iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.source.as_deref(), Some(sources::LINTER));
            let NumberOrString::String(code) = diagnostic.code.unwrap() else {
                panic!("lint codes are rule names");
            };
            (code, diagnostic.range.start, diagnostic.range.end)
        })
        .collect()
}

#[test]
fn a_tsrx_document_reports_markup_rules_at_authored_positions() {
    let source = "export function List({ items }: { items: string[] }) @{\n  <ul>\n    @for (const item of items; key item) {\n      <li  title={item}>{item}</li>\n    }\n  </ul>\n}\n";
    assert_eq!(
        collect(&state(true), source),
        [(
            "vue/no-multi-spaces".to_string(),
            Position::new(3, 9),
            Position::new(3, 11),
        )]
    );
}

#[test]
fn a_tsrx_syntax_error_is_a_parser_diagnostic_not_an_sfc_one() {
    let source = "export function Notice() @{\n  <div>Notice</span>\n}\n";
    let diagnostics = collect(&state(true), source);
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|(code, _, _)| code.as_str())
        .collect();
    assert_eq!(codes, ["parser/tsrx"]);
}

#[test]
fn lint_off_reports_nothing_for_a_tsrx_document() {
    let source = "export function Notice() @{\n  <div>Notice</span>\n}\n";
    assert_eq!(collect(&state(false), source), []);
}
