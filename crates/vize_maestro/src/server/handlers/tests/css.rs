//! Real LSP CSS requests must not start an unavailable native type checker.

use tower_lsp::{
    LanguageServer, LspService,
    lsp_types::{
        ClientCapabilities, CompletionItem, CompletionParams, CompletionResponse, Documentation,
        HoverContents, HoverParams, InitializeParams, MarkupKind, Position, TextDocumentIdentifier,
        TextDocumentPositionParams, Url, WorkspaceFolder,
    },
};

use super::super::MaestroServer;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/css-documentation/App.vue"
));

fn position(uri: &Url, needle: &str, inside: usize) -> TextDocumentPositionParams {
    let offset = SOURCE.find(needle).unwrap() + inside;
    let (line, character) = crate::ide::offset_to_position(SOURCE, offset);
    TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        position: Position::new(line, character),
    }
}

fn items(response: CompletionResponse) -> Vec<CompletionItem> {
    match response {
        CompletionResponse::Array(items) => items,
        CompletionResponse::List(list) => list.items,
    }
}

fn documentation(item: &CompletionItem) -> &str {
    match item.documentation.as_ref().unwrap() {
        Documentation::MarkupContent(content) => {
            assert_eq!(content.kind, MarkupKind::Markdown);
            &content.value
        }
        Documentation::String(_) => panic!("CSS documentation must preserve rich Markdown"),
    }
}

fn unstarted(server: &MaestroServer) {
    assert!(!server.state.has_corsa_bridge());
    assert!(server.state.corsa_init_failure().is_none());
}

#[test]
fn css_language_server_handlers_complete_hover_and_resolve_without_starting_missing_corsa() {
    crate::runtime::block_on(async {
        for lazy in [false, true] {
            let project = tempfile::tempdir().unwrap();
            let missing = project.path().join("definitely-missing-corsa-css-control");
            assert!(!missing.exists());
            std::fs::write(
                project.path().join("vize.config.json"),
                serde_json::to_vec(&serde_json::json!({
                    "typeChecker": { "corsaPath": missing },
                    "lsp": { "completion": true, "hover": true, "typecheck": true, "lint": false }
                }))
                .unwrap(),
            )
            .unwrap();
            let capabilities = if lazy {
                serde_json::from_value(serde_json::json!({
                    "textDocument": { "completion": { "completionItem": {
                        "resolveSupport": { "properties": ["documentation"] }
                    } } }
                }))
                .unwrap()
            } else {
                ClientCapabilities::default()
            };
            let (service, _socket) = LspService::new(MaestroServer::new);
            let server = service.inner();
            server
                .initialize(InitializeParams {
                    capabilities,
                    workspace_folders: Some(vec![WorkspaceFolder {
                        uri: Url::from_directory_path(project.path()).unwrap(),
                        name: "CSS handler control".into(),
                    }]),
                    ..InitializeParams::default()
                })
                .await
                .unwrap();
            assert!(server.state.is_lsp_typecheck_enabled());
            assert!(!server.state.is_lsp_lint_enabled());
            assert!(server.state.lsp_features().completion);
            assert!(server.state.lsp_features().hover);
            assert_eq!(
                server.state.supports_completion_documentation_resolve(),
                lazy
            );
            assert_eq!(
                server.state.get_type_checker_config().runtime_path(),
                missing.to_str()
            );
            let path = project.path().join("App.vue");
            std::fs::write(&path, SOURCE).unwrap();
            let uri = Url::from_file_path(path).unwrap();
            // Avoid didOpen's independent background diagnostics lane.
            server
                .state
                .documents
                .open(uri.clone(), SOURCE.into(), 1, "vue".into());
            unstarted(server);

            for (needle, inside, label, useful) in [
                ("display: flex", 3, "display", "**Syntax**"),
                ("display: flex", 11, "flex", "display: flex;"),
                ("color: red", 8, "red", "#ff0000"),
                ("width: auto", 9, "auto", "width: auto;"),
                ("&:hover", 4, ":hover", "pointing device"),
                ("::before", 5, "::before", "pseudo-element"),
                ("@media", 4, "@media", "media type"),
                ("v-bind", 3, "v-bind", "both plain and scoped style blocks"),
            ] {
                let position = position(&uri, needle, inside);
                let response = server
                    .completion(CompletionParams {
                        text_document_position: position.clone(),
                        work_done_progress_params: Default::default(),
                        partial_result_params: Default::default(),
                        context: None,
                    })
                    .await
                    .unwrap()
                    .unwrap();
                let items = items(response);
                let selected = items
                    .iter()
                    .find(|item| item.label == label)
                    .unwrap()
                    .clone();
                if lazy {
                    assert!(
                        items
                            .iter()
                            .all(|item| item.documentation.is_none() && item.data.is_some())
                    );
                } else {
                    assert!(documentation(&selected).contains(useful));
                }
                unstarted(server);
                let resolved = if lazy {
                    let resolved = server.completion_resolve(selected.clone()).await.unwrap();
                    let mut insertion = resolved.clone();
                    insertion.documentation = selected.documentation.clone();
                    assert_eq!(
                        insertion, selected,
                        "LSP resolve must keep every original insertion field"
                    );
                    resolved
                } else {
                    assert!(selected.data.is_none());
                    selected
                };
                let docs = documentation(&resolved);
                assert!(docs.contains(useful), "{docs}");
                assert!(docs.contains("**Docs**"));
                unstarted(server);
                let hover = server
                    .hover(HoverParams {
                        text_document_position_params: position,
                        work_done_progress_params: Default::default(),
                    })
                    .await
                    .unwrap()
                    .unwrap();
                match hover.contents {
                    HoverContents::Markup(content) => {
                        assert_eq!(content.kind, MarkupKind::Markdown);
                        assert_eq!(content.value, docs);
                    }
                    _ => panic!("real CSS handler must return Markdown hover"),
                }
                unstarted(server);
            }

            // A positive startup control proves the pinned config cannot fall
            // through to an installed/discovered provider behind the test.
            assert!(server.state.get_corsa_bridge().await.is_none());
            assert!(!server.state.has_corsa_bridge());
            let failure = server.state.corsa_init_failure().unwrap();
            assert!(failure.contains("spawn failed"), "{failure}");
            assert!(
                failure.contains("definitely-missing-corsa-css-control"),
                "{failure}"
            );
        }
    });
}
