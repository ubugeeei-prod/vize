//! Gate tests for the three formatting handlers.
//!
//! `formatting`, `range_formatting` and `on_type_formatting` all reach the same
//! SFC formatter, so they share the same two gates: the `formatting` feature
//! flag, and the standalone-HTML exclusion. The formatting *behaviour* is
//! covered where it lives — `server::format::range` and
//! `server::format::on_type` — so these tests only prove the routing.

use super::*;
use tower_lsp::lsp_types::DocumentOnTypeFormattingParams;

const STANDALONE_HTML: &str = "<!DOCTYPE html>\n<html><body>\n<div   v-scope=\"{ count: 0 }\" >{{ count }}</div>\n</body></html>\n";
const SFC: &str = "<template>\n<div>hello</div>\n</template>\n";

pub(super) fn formatting_params(uri: Url) -> DocumentFormattingParams {
    DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri },
        options: FormattingOptions::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

fn range_params(uri: Url) -> DocumentRangeFormattingParams {
    DocumentRangeFormattingParams {
        text_document: TextDocumentIdentifier { uri },
        range: Range::new(Position::new(0, 0), Position::new(0, 1)),
        options: FormattingOptions::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

fn on_type_params(uri: Url) -> DocumentOnTypeFormattingParams {
    DocumentOnTypeFormattingParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position::new(2, 1),
        },
        ch: "}".to_string(),
        options: FormattingOptions::default(),
    }
}

/// A server with `formatting` on and `uri` open with `source`.
fn server_with(uri: &Url, source: &str, language: &str) -> LspService<MaestroServer> {
    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({ "formatting": true })));
    server
        .state
        .documents
        .open(uri.clone(), source.to_string(), 1, language.to_string());
    service
}

#[test]
fn every_formatting_handler_declines_standalone_html() {
    let uri = Url::parse("file:///index.html").unwrap();
    let service = server_with(&uri, STANDALONE_HTML, "html");
    let server = service.inner();

    // Running the SFC formatter over a petite-vue page corrupts it, so all
    // three commands must decline rather than emit edits (#1393).
    assert_eq!(
        futures::executor::block_on(server.formatting(formatting_params(uri.clone()))).unwrap(),
        None
    );
    assert_eq!(
        futures::executor::block_on(server.range_formatting(range_params(uri.clone()))).unwrap(),
        None
    );
    assert_eq!(
        futures::executor::block_on(server.on_type_formatting(on_type_params(uri))).unwrap(),
        None
    );
}

/// Guard against the standalone-HTML gate over-matching.
#[cfg(feature = "glyph")]
#[test]
fn sfc_formatting_still_runs() {
    let uri = Url::parse("file:///App.vue").unwrap();
    let service = server_with(&uri, SFC, "vue");
    let server = service.inner();

    let edits =
        futures::executor::block_on(server.formatting(formatting_params(uri.clone()))).unwrap();
    assert_eq!(
        edits,
        Some(vec![TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(3, 0)),
            // The formatter indents the block and breaks the element's text
            // onto its own line — the reflow that makes a whole-document line
            // pairing unusable, hence the per-block one in `format::on_type`.
            new_text: "<template>\n  <div>\n    hello\n  </div>\n</template>\n".to_string(),
        }])
    );
}

#[test]
fn formatting_handlers_are_gated_off_by_default() {
    // `formatting` is opt-in, so a server that was never told to format must
    // answer all three commands with "no provider here".
    let uri = Url::parse("file:///App.vue").unwrap();
    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server
        .state
        .documents
        .open(uri.clone(), SFC.to_string(), 1, "vue".to_string());

    assert_eq!(
        futures::executor::block_on(server.formatting(formatting_params(uri.clone()))).unwrap(),
        None
    );
    assert_eq!(
        futures::executor::block_on(server.range_formatting(range_params(uri.clone()))).unwrap(),
        None
    );
    assert_eq!(
        futures::executor::block_on(server.on_type_formatting(on_type_params(uri))).unwrap(),
        None
    );
}
