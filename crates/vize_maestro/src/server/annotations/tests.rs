use tower_lsp::lsp_types::{
    Color, ColorPresentationParams, DocumentColorParams, PartialResultParams,
    TextDocumentIdentifier, Url, WorkDoneProgressParams,
};

use super::{color_presentation, document_color};
use crate::server::ServerState;

const SFC: &str = "<template>\n  <div style=\"color: #f00\" />\n</template>\n\n<style scoped>\n.a { background: rgba(0, 0, 255, 0.5) }\n.b { color: hsl(120 100% 25%) }\n</style>\n";

fn state_with(uri: &Url, source: &str) -> ServerState {
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state
}

fn color_params(uri: Url) -> DocumentColorParams {
    DocumentColorParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

#[test]
fn document_color_reports_every_css_literal_in_document_order() {
    let uri = Url::parse("file:///App.vue").unwrap();
    let state = state_with(&uri, SFC);

    let colors: Vec<(u32, u32, u32)> = document_color(&state, &color_params(uri))
        .into_iter()
        .map(|info| {
            (
                info.range.start.line,
                info.range.start.character,
                info.range.end.character,
            )
        })
        .collect();

    // The inline attribute on line 1, then rgba() and hsl() in the style block.
    assert_eq!(colors, vec![(1, 21, 25), (5, 17, 37), (6, 12, 29)]);
}

#[test]
fn an_unopened_document_reports_an_empty_array_not_an_error() {
    // `textDocument/documentColor` has no "unknown" answer: an empty array is
    // the correct response for a document the server has never seen.
    let state = ServerState::new();
    let uri = Url::parse("file:///Missing.vue").unwrap();
    assert_eq!(document_color(&state, &color_params(uri)), Vec::new());
}

#[test]
fn color_presentation_does_not_consult_the_document() {
    // The client already sent the range it wants replaced, so the response is a
    // pure function of the colour the picker produced.
    let params = ColorPresentationParams {
        text_document: TextDocumentIdentifier {
            uri: Url::parse("file:///Never opened.vue").unwrap(),
        },
        color: Color {
            red: 0.0,
            green: 1.0,
            blue: 0.0,
            alpha: 1.0,
        },
        range: tower_lsp::lsp_types::Range::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };

    let presentations = color_presentation(&params);
    let labels: Vec<&str> = presentations
        .iter()
        .map(|presentation| presentation.label.as_str())
        .collect();
    assert_eq!(
        labels,
        vec!["#00ff00", "rgb(0, 255, 0)", "hsl(120 100% 50%)"]
    );
}
