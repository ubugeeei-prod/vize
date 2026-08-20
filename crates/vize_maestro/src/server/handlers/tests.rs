use super::*;
use tower_lsp::{
    LspService,
    lsp_types::{FormattingOptions, TextDocumentIdentifier, Url, WorkDoneProgressParams},
};

mod formatting;
#[cfg(feature = "native")]
mod initial_diagnostics;
mod lifecycle;
mod requests;

use formatting::formatting_params;

use tower_lsp::lsp_types::{
    CodeActionContext, CodeActionParams, DocumentSymbolParams, PartialResultParams,
    ReferenceContext, ReferenceParams, RenameParams, SemanticTokensParams,
    TextDocumentPositionParams,
};

fn open_tsx(server: &MaestroServer, uri: &Url, source: &str) {
    server.state.documents.open(
        uri.clone(),
        source.to_string(),
        1,
        "typescriptreact".to_string(),
    );
    server.state.update_virtual_docs(uri, source);
}

fn tsx_server(source: &str, uri: &Url) -> tower_lsp::LspService<MaestroServer> {
    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    // Enable the structural LSP features (default off in tests).
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({
            "documentSymbols": true,
            "semanticTokens": true,
            "codeActions": true,
            "lint": true,
            "references": true,
            "rename": true,
        })));
    open_tsx(server, uri, source);
    service
}

#[test]
fn document_symbol_lists_tsx_components() {
    let uri = Url::parse("file:///Counter.tsx").unwrap();
    let source = "const Counter = (props: { n: number }) => <span>{props.n}</span>;\n";
    let service = tsx_server(source, &uri);
    let server = service.inner();

    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let response = futures::executor::block_on(server.document_symbol(params)).unwrap();
    match response {
        Some(DocumentSymbolResponse::Nested(symbols)) => {
            assert_eq!(symbols.len(), 1);
            assert_eq!(symbols[0].name, "Counter");
        }
        other => panic!("expected nested TSX component symbols, got: {other:?}"),
    }
}

#[test]
fn semantic_tokens_full_highlights_tsx_expressions() {
    let uri = Url::parse("file:///Comp.tsx").unwrap();
    let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
    let service = tsx_server(source, &uri);
    let server = service.inner();

    let params = SemanticTokensParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let response = futures::executor::block_on(server.semantic_tokens_full(params)).unwrap();
    match response {
        Some(SemanticTokensResult::Tokens(tokens)) => {
            assert!(!tokens.data.is_empty(), "expected highlighted TSX tokens");
        }
        other => panic!("expected TSX semantic tokens, got: {other:?}"),
    }
}

#[test]
fn code_action_surfaces_tsx_quickfix() {
    let uri = Url::parse("file:///Comp.tsx").unwrap();
    // Multi-space inside the opening tag is a fixable JSX lint diagnostic.
    let source = "const C = () => <div    class=\"a\">x</div>;\n";
    let service = tsx_server(source, &uri);
    let server = service.inner();

    let params = CodeActionParams {
        text_document: TextDocumentIdentifier { uri },
        range: Range::new(Position::new(0, 0), Position::new(1, 0)),
        context: CodeActionContext::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let response = futures::executor::block_on(server.code_action(params)).unwrap();
    let actions = response.expect("expected TSX quickfix code actions");
    assert!(
        actions.iter().any(|action| matches!(
            action,
            tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(_)
        )),
        "expected at least one quickfix, got: {actions:?}"
    );
}

#[test]
fn embedded_scoped_style_produces_css_virtual_document() {
    let uri = Url::parse("file:///Styled.tsx").unwrap();
    let source = "const C = () => (\n  <>\n    <div class=\"box\">hi</div>\n    <style scoped>{`\n      .box { color: red; }\n    `}</style>\n  </>\n);\n";
    let service = tsx_server(source, &uri);
    let server = service.inner();

    let docs = server
        .state
        .get_virtual_docs(&uri)
        .expect("virtual docs cached for a TSX with <style scoped>");
    assert_eq!(docs.styles.len(), 1, "one CSS virtual document expected");
    let style = &docs.styles[0];
    assert!(style.uri.as_str().ends_with(".css"));
    assert!(style.content.contains(".box"));
}

#[test]
fn references_gated_off_returns_none_for_tsx() {
    // jsxTypecheck defaults off, so the type-aware references path must not
    // run for `.tsx` (and the SFC fallback never fires for non-SFC JSX).
    let uri = Url::parse("file:///Comp.tsx").unwrap();
    let source = "const C = (props: { msg: string }) => {\n  const total = props.msg;\n  return <span>{total}</span>;\n};\n";
    let service = tsx_server(source, &uri);
    let server = service.inner();
    assert!(
        !server.state.jsx_typecheck_enabled(),
        "precondition: jsxTypecheck is off by default"
    );

    let offset = source.find("total").unwrap();
    let (line, character) = crate::ide::offset_to_position(source, offset);
    let params = ReferenceParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position::new(line, character),
        },
        context: ReferenceContext {
            include_declaration: true,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let response = futures::executor::block_on(server.references(params)).unwrap();
    assert!(
        response.is_none(),
        "references must be gated off for TSX when jsxTypecheck is disabled"
    );
}

#[test]
fn rename_gated_off_returns_none_for_tsx() {
    let uri = Url::parse("file:///Comp.tsx").unwrap();
    let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
    let service = tsx_server(source, &uri);
    let server = service.inner();

    let offset = source.find("msg").unwrap();
    let (line, character) = crate::ide::offset_to_position(source, offset);
    let params = RenameParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: Position::new(line, character),
        },
        new_name: "renamed".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    };
    let response = futures::executor::block_on(server.rename(params)).unwrap();
    assert!(
        response.is_none(),
        "rename must be gated off for TSX when jsxTypecheck is disabled"
    );
}

/// A component directive change (`"use vue:vdom"` / `"use vue:vapor"`)
/// invalidates the derived virtual docs and diagnostics (#1498). `did_change`
/// fully re-runs `update_virtual_docs` and the diagnostics re-lower from the
/// current content, so a malformed directive's error clears the moment the
/// directive is corrected — no stale cache survives the edit.
#[test]
fn directive_edit_reinvalidates_jsx_diagnostics_and_virtual_docs() {
    use tower_lsp::lsp_types::{
        DidChangeTextDocumentParams, DidOpenTextDocumentParams, TextDocumentContentChangeEvent,
        TextDocumentItem, VersionedTextDocumentIdentifier,
    };

    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({ "lint": true })));

    let uri = Url::parse("file:///ModeSwitch.tsx").unwrap();
    // A typo'd mode directive (error) plus a `<style scoped>` so the JSX
    // scoped-CSS virtual doc is part of what must be rebuilt.
    let before = "const C = () => {\n  \"use vue:vdomx\";\n  return (\n    <>\n      <div class=\"box\">hi</div>\n      <style scoped>{`.box { color: red; }`}</style>\n    </>\n  );\n};\n";
    futures::executor::block_on(server.did_open(DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "typescriptreact".to_string(),
            version: 1,
            text: before.to_string(),
        },
    }));

    // The malformed directive is surfaced, and the scoped-CSS virtual doc is
    // cached from the initial content.
    let before_diags = crate::ide::DiagnosticService::collect(&server.state, &uri);
    assert!(
            before_diags.iter().any(|d| d.message
                == "unknown JSX mode directive \"use vue:vdomx\": expected \"use vue:vdom\" or \"use vue:vapor\""),
            "expected the malformed-directive diagnostic before the edit, got: {before_diags:?}"
        );
    assert_eq!(
        server
            .state
            .get_virtual_docs(&uri)
            .map(|docs| docs.styles.len()),
        Some(1),
        "scoped-CSS virtual doc must be cached from the initial content"
    );

    // Correct the directive to a valid `"use vue:vapor"`.
    let after = "const C = () => {\n  \"use vue:vapor\";\n  return (\n    <>\n      <div class=\"box\">hi</div>\n      <style scoped>{`.box { color: red; }`}</style>\n    </>\n  );\n};\n";
    futures::executor::block_on(server.did_change(DidChangeTextDocumentParams {
        text_document: VersionedTextDocumentIdentifier {
            uri: uri.clone(),
            version: 2,
        },
        content_changes: vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: after.to_string(),
        }],
    }));

    // The stale malformed-directive diagnostic is gone: the diagnostics
    // re-lowered from the edited content (cache invalidated).
    let after_diags = crate::ide::DiagnosticService::collect(&server.state, &uri);
    assert!(
        !after_diags.iter().any(|d| d.source.as_deref()
            == Some(crate::ide::diagnostics::sources::JSX_COMPILER)
            && d.severity == Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR)),
        "directive fix must clear the JSX mode error, got: {after_diags:?}"
    );
    // And the virtual docs were rebuilt against the new content (still one
    // scoped-CSS doc; the rebuild ran rather than serving a stale entry).
    assert_eq!(
        server
            .state
            .get_virtual_docs(&uri)
            .map(|docs| docs.styles.len()),
        Some(1),
        "virtual docs must be rebuilt on the directive edit"
    );
}

#[test]
fn document_symbol_still_parses_sfc_after_jsx_routing() {
    // Guard against the JSX gate over-matching: a regular `.vue` SFC must
    // still go through the SFC document-symbol path.
    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({ "documentSymbols": true })));
    let uri = Url::parse("file:///App.vue").unwrap();
    let source = "<template>\n  <div>hi</div>\n</template>\n<script setup lang=\"ts\">\nconst x = 1\n</script>\n";
    server
        .state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());

    let params = DocumentSymbolParams {
        text_document: TextDocumentIdentifier { uri },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let response = futures::executor::block_on(server.document_symbol(params)).unwrap();
    match response {
        Some(DocumentSymbolResponse::Nested(symbols)) => {
            assert!(
                symbols.iter().any(|s| s.name == "template"),
                "SFC document symbols must still include the template block"
            );
        }
        other => panic!("expected SFC block symbols, got: {other:?}"),
    }
}
