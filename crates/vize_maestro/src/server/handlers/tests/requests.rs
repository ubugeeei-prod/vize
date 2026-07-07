use super::*;
use tower_lsp::{
    LspService,
    lsp_types::{
        CodeLensParams, DocumentLinkParams, FileRename, FoldingRangeParams, HoverParams,
        InlayHintParams, SemanticTokensRangeParams, WorkspaceSymbolParams,
    },
};

mod guards_extra;
mod responses;

const SAMPLE: &str = "<template>\n  <div>{{ message }}</div>\n</template>\n\
                      <script setup lang=\"ts\">\nconst message = 'hi'\n</script>\n\
                      <style scoped>\n.box { color: red; }\n</style>\n";

fn service_with_options(options: serde_json::Value) -> tower_lsp::LspService<MaestroServer> {
    let (service, _socket) = LspService::new(MaestroServer::new);
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&options));
    service
}

fn quiet_options(mut overrides: serde_json::Map<String, serde_json::Value>) -> serde_json::Value {
    overrides.insert("lint".to_string(), false.into());
    overrides.insert("typecheck".to_string(), false.into());
    overrides.insert("ecosystem".to_string(), false.into());
    serde_json::Value::Object(overrides)
}

fn options(pairs: &[(&str, bool)]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), (*value).into());
    }
    quiet_options(map)
}

fn uri(path: &str) -> Url {
    Url::parse(&format!("file:///{path}")).unwrap()
}

fn open_vue(server: &MaestroServer, uri: &Url, source: &str) {
    server
        .state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    server.state.update_virtual_docs(uri, source);
}

fn text_doc(uri: &Url) -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri: uri.clone() }
}

fn text_pos(uri: &Url) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: text_doc(uri),
        position: Position::new(0, 0),
    }
}

fn range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 1))
}

fn hover_params(uri: &Url) -> HoverParams {
    HoverParams {
        text_document_position_params: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

fn completion_params(uri: &Url) -> CompletionParams {
    CompletionParams {
        text_document_position: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    }
}

fn definition_params(uri: &Url) -> GotoDefinitionParams {
    GotoDefinitionParams {
        text_document_position_params: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn references_params(uri: &Url) -> ReferenceParams {
    ReferenceParams {
        text_document_position: text_pos(uri),
        context: ReferenceContext {
            include_declaration: true,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn document_symbol_params(uri: &Url) -> DocumentSymbolParams {
    DocumentSymbolParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn code_action_params(uri: &Url) -> CodeActionParams {
    CodeActionParams {
        text_document: text_doc(uri),
        range: range(),
        context: CodeActionContext::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn rename_params(uri: &Url) -> RenameParams {
    RenameParams {
        text_document_position: text_pos(uri),
        new_name: "renamed".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

fn semantic_tokens_params(uri: &Url) -> SemanticTokensParams {
    SemanticTokensParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn semantic_tokens_range_params(uri: &Url) -> SemanticTokensRangeParams {
    SemanticTokensRangeParams {
        text_document: text_doc(uri),
        range: range(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn code_lens_params(uri: &Url) -> CodeLensParams {
    CodeLensParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn document_link_params(uri: &Url) -> DocumentLinkParams {
    DocumentLinkParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn inlay_hint_params(uri: &Url) -> InlayHintParams {
    InlayHintParams {
        text_document: text_doc(uri),
        range: range(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

fn folding_range_params(uri: &Url) -> FoldingRangeParams {
    FoldingRangeParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

macro_rules! disabled_open_doc_request_returns_none {
    ($name:ident, $opts:expr, |$server:ident, $uri:ident| $call:expr) => {
        #[test]
        fn $name() {
            let service = service_with_options(options($opts));
            let $server = service.inner();
            let $uri = uri("Guard.vue");
            open_vue($server, &$uri, SAMPLE);
            let response = futures::executor::block_on($call).unwrap();
            assert!(response.is_none());
        }
    };
}

macro_rules! enabled_missing_doc_request_returns_none {
    ($name:ident, $opts:expr, |$server:ident, $uri:ident| $call:expr) => {
        #[test]
        fn $name() {
            let service = service_with_options(options($opts));
            let $server = service.inner();
            let $uri = uri("Missing.vue");
            let response = futures::executor::block_on($call).unwrap();
            assert!(response.is_none());
        }
    };
}

disabled_open_doc_request_returns_none!(
    hover_disabled_returns_none,
    &[("hover", false)],
    |server, uri| server.hover(hover_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    hover_missing_document_returns_none,
    &[("hover", true)],
    |server, uri| server.hover(hover_params(&uri))
);
disabled_open_doc_request_returns_none!(
    completion_disabled_returns_none,
    &[("completion", false)],
    |server, uri| server.completion(completion_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    completion_missing_document_returns_none,
    &[("completion", true)],
    |server, uri| server.completion(completion_params(&uri))
);
disabled_open_doc_request_returns_none!(
    definition_disabled_returns_none,
    &[("definition", false)],
    |server, uri| server.goto_definition(definition_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    definition_missing_document_returns_none,
    &[("definition", true)],
    |server, uri| server.goto_definition(definition_params(&uri))
);
disabled_open_doc_request_returns_none!(
    references_disabled_returns_none,
    &[("references", false)],
    |server, uri| server.references(references_params(&uri))
);
disabled_open_doc_request_returns_none!(
    document_highlight_disabled_returns_none,
    &[("references", false)],
    |server, uri| {
        server.document_highlight(DocumentHighlightParams {
            text_document_position_params: text_pos(&uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }
);
disabled_open_doc_request_returns_none!(
    document_symbol_disabled_returns_none,
    &[("documentSymbols", false)],
    |server, uri| server.document_symbol(document_symbol_params(&uri))
);
disabled_open_doc_request_returns_none!(
    code_action_disabled_when_lint_is_off,
    &[("lint", false), ("codeActions", true)],
    |server, uri| server.code_action(code_action_params(&uri))
);
disabled_open_doc_request_returns_none!(
    code_action_disabled_when_code_actions_are_off,
    &[("lint", true), ("codeActions", false)],
    |server, uri| server.code_action(code_action_params(&uri))
);
disabled_open_doc_request_returns_none!(
    prepare_rename_disabled_returns_none,
    &[("rename", false)],
    |server, uri| server.prepare_rename(text_pos(&uri))
);
disabled_open_doc_request_returns_none!(
    rename_disabled_returns_none,
    &[("rename", false)],
    |server, uri| server.rename(rename_params(&uri))
);
disabled_open_doc_request_returns_none!(
    semantic_tokens_full_disabled_returns_none,
    &[("semanticTokens", false)],
    |server, uri| server.semantic_tokens_full(semantic_tokens_params(&uri))
);
disabled_open_doc_request_returns_none!(
    semantic_tokens_range_disabled_returns_none,
    &[("semanticTokens", false)],
    |server, uri| server.semantic_tokens_range(semantic_tokens_range_params(&uri))
);
disabled_open_doc_request_returns_none!(
    code_lens_disabled_returns_none,
    &[("codeLens", false)],
    |server, uri| server.code_lens(code_lens_params(&uri))
);
disabled_open_doc_request_returns_none!(
    document_link_disabled_returns_none,
    &[("documentLinks", false)],
    |server, uri| server.document_link(document_link_params(&uri))
);
disabled_open_doc_request_returns_none!(
    inlay_hint_disabled_returns_none,
    &[("inlayHints", false)],
    |server, uri| server.inlay_hint(inlay_hint_params(&uri))
);
disabled_open_doc_request_returns_none!(
    folding_range_disabled_returns_none,
    &[("foldingRanges", false)],
    |server, uri| server.folding_range(folding_range_params(&uri))
);
disabled_open_doc_request_returns_none!(
    formatting_disabled_returns_none,
    &[("formatting", false)],
    |server, uri| server.formatting(formatting_params(uri))
);
disabled_open_doc_request_returns_none!(
    range_formatting_disabled_returns_none,
    &[("formatting", false)],
    |server, uri| {
        server.range_formatting(DocumentRangeFormattingParams {
            text_document: text_doc(&uri),
            range: range(),
            options: FormattingOptions::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }
);

#[test]
fn workspace_symbols_disabled_returns_none() {
    let service = service_with_options(options(&[("workspaceSymbols", false)]));
    let response = futures::executor::block_on(service.inner().symbol(WorkspaceSymbolParams {
        query: "message".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }))
    .unwrap();
    assert!(response.is_none());
}

#[test]
fn file_rename_disabled_returns_none() {
    let service = service_with_options(options(&[("fileRename", false)]));
    let old_uri = uri("Old.vue");
    let new_uri = uri("New.vue");
    let response =
        futures::executor::block_on(service.inner().will_rename_files(RenameFilesParams {
            files: vec![FileRename {
                old_uri: old_uri.to_string(),
                new_uri: new_uri.to_string(),
            }],
        }))
        .unwrap();
    assert!(response.is_none());
}
