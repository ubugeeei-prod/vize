//! Shared request-parameter builders for the handler guard tests.

use vize_s0::cstr;

use super::*;

pub(super) fn service_with_options(
    options: serde_json::Value,
) -> tower_lsp::LspService<MaestroServer> {
    let (service, _socket) = LspService::new(MaestroServer::new);
    service
        .inner()
        .state
        .apply_lsp_initialization_options(Some(&options));
    service
}

pub(super) fn quiet_options(
    mut overrides: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    overrides.insert("lint".to_string(), false.into());
    overrides.insert("typecheck".to_string(), false.into());
    overrides.insert("ecosystem".to_string(), false.into());
    serde_json::Value::Object(overrides)
}

pub(super) fn options(pairs: &[(&str, bool)]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, value) in pairs {
        map.insert((*key).to_string(), (*value).into());
    }
    quiet_options(map)
}

pub(super) fn uri(path: &str) -> Url {
    Url::parse(&cstr!("file:///{path}")).unwrap()
}

pub(super) fn open_vue(server: &MaestroServer, uri: &Url, source: &str) {
    server
        .state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    server.state.update_virtual_docs(uri, source);
}

pub(super) fn text_doc(uri: &Url) -> TextDocumentIdentifier {
    TextDocumentIdentifier { uri: uri.clone() }
}

pub(super) fn text_pos(uri: &Url) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: text_doc(uri),
        position: Position::new(0, 0),
    }
}

pub(super) fn range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 1))
}

pub(super) fn hover_params(uri: &Url) -> HoverParams {
    HoverParams {
        text_document_position_params: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

pub(super) fn completion_params(uri: &Url) -> CompletionParams {
    CompletionParams {
        text_document_position: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
        context: None,
    }
}

pub(super) fn signature_help_params(uri: &Url) -> SignatureHelpParams {
    SignatureHelpParams {
        text_document_position_params: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        context: None,
    }
}

pub(super) fn definition_params(uri: &Url) -> DefParams {
    DefParams {
        text_document_position_params: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn type_definition_params(uri: &Url) -> TypeDefParams {
    TypeDefParams {
        text_document_position_params: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn declaration_params(uri: &Url) -> DeclParams {
    DeclParams {
        text_document_position_params: text_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn references_params(uri: &Url) -> ReferenceParams {
    ReferenceParams {
        text_document_position: text_pos(uri),
        context: ReferenceContext {
            include_declaration: true,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn document_symbol_params(uri: &Url) -> DocumentSymbolParams {
    DocumentSymbolParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn code_action_params(uri: &Url) -> CodeActionParams {
    CodeActionParams {
        text_document: text_doc(uri),
        range: range(),
        context: CodeActionContext::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn rename_params(uri: &Url) -> RenameParams {
    RenameParams {
        text_document_position: text_pos(uri),
        new_name: "renamed".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

pub(super) fn semantic_tokens_params(uri: &Url) -> SemanticTokensParams {
    SemanticTokensParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn semantic_tokens_range_params(uri: &Url) -> SemanticTokensRangeParams {
    SemanticTokensRangeParams {
        text_document: text_doc(uri),
        range: range(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn code_lens_params(uri: &Url) -> CodeLensParams {
    CodeLensParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn document_link_params(uri: &Url) -> DocumentLinkParams {
    DocumentLinkParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

pub(super) fn inlay_hint_params(uri: &Url) -> InlayHintParams {
    InlayHintParams {
        text_document: text_doc(uri),
        range: range(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

pub(super) fn folding_range_params(uri: &Url) -> FoldingRangeParams {
    FoldingRangeParams {
        text_document: text_doc(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}
