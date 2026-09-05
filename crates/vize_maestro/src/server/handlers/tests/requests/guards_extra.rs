use super::*;

fn out_of_range_pos(uri: &Url) -> TextDocumentPositionParams {
    TextDocumentPositionParams {
        text_document: text_doc(uri),
        position: Position::new(999, 0),
    }
}

fn out_of_range_range() -> Range {
    Range::new(Position::new(999, 0), Position::new(999, 1))
}

fn out_of_range_code_action_params(uri: &Url) -> CodeActionParams {
    CodeActionParams {
        text_document: text_doc(uri),
        range: out_of_range_range(),
        context: CodeActionContext::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn out_of_range_rename_params(uri: &Url) -> RenameParams {
    RenameParams {
        text_document_position: out_of_range_pos(uri),
        new_name: "renamed".to_string(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

fn out_of_range_document_highlight_params(uri: &Url) -> DocumentHighlightParams {
    DocumentHighlightParams {
        text_document_position_params: out_of_range_pos(uri),
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn out_of_range_reference_params(uri: &Url) -> ReferenceParams {
    ReferenceParams {
        text_document_position: out_of_range_pos(uri),
        context: ReferenceContext {
            include_declaration: true,
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}

fn range_formatting_params_with_range(uri: &Url, range: Range) -> DocumentRangeFormattingParams {
    DocumentRangeFormattingParams {
        text_document: text_doc(uri),
        range,
        options: FormattingOptions::default(),
        work_done_progress_params: WorkDoneProgressParams::default(),
    }
}

macro_rules! enabled_missing_doc_request_returns_none {
    ($name:ident, $opts:expr, |$server:ident, $uri:ident| $call:expr) => {
        #[test]
        fn $name() {
            let service = service_with_options(options($opts));
            let $server = service.inner();
            let $uri = uri("MissingExtra.vue");
            let response = futures::executor::block_on($call).unwrap();
            assert!(response.is_none());
        }
    };
}

macro_rules! enabled_out_of_range_request_returns_none {
    ($name:ident, $opts:expr, |$server:ident, $uri:ident| $call:expr) => {
        #[test]
        fn $name() {
            let service = service_with_options(options($opts));
            let $server = service.inner();
            let $uri = uri("OutOfRange.vue");
            open_vue($server, &$uri, SAMPLE);
            let response = futures::executor::block_on($call).unwrap();
            assert!(response.is_none());
        }
    };
}

enabled_missing_doc_request_returns_none!(
    references_missing_document_returns_none,
    &[("references", true)],
    |server, uri| server.references(references_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    document_highlight_missing_document_returns_none,
    &[("references", true)],
    |server, uri| {
        server.document_highlight(DocumentHighlightParams {
            text_document_position_params: text_pos(&uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }
);
enabled_missing_doc_request_returns_none!(
    document_symbol_missing_document_returns_none,
    &[("documentSymbols", true)],
    |server, uri| server.document_symbol(document_symbol_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    code_action_missing_document_returns_none,
    &[("lint", true), ("codeActions", true)],
    |server, uri| server.code_action(code_action_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    prepare_rename_missing_document_returns_none,
    &[("rename", true)],
    |server, uri| server.prepare_rename(text_pos(&uri))
);
enabled_missing_doc_request_returns_none!(
    rename_missing_document_returns_none,
    &[("rename", true)],
    |server, uri| server.rename(rename_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    semantic_tokens_full_missing_document_returns_none,
    &[("semanticTokens", true)],
    |server, uri| server.semantic_tokens_full(semantic_tokens_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    semantic_tokens_range_missing_document_returns_none,
    &[("semanticTokens", true)],
    |server, uri| server.semantic_tokens_range(semantic_tokens_range_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    code_lens_missing_document_returns_none,
    &[("codeLens", true)],
    |server, uri| server.code_lens(code_lens_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    document_link_missing_document_returns_none,
    &[("documentLinks", true)],
    |server, uri| server.document_link(document_link_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    inlay_hint_missing_document_returns_none,
    &[("inlayHints", true)],
    |server, uri| server.inlay_hint(inlay_hint_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    folding_range_missing_document_returns_none,
    &[("foldingRanges", true)],
    |server, uri| server.folding_range(folding_range_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    formatting_missing_document_returns_none,
    &[("formatting", true)],
    |server, uri| server.formatting(formatting_params(uri))
);
enabled_missing_doc_request_returns_none!(
    range_formatting_missing_document_returns_none,
    &[("formatting", true)],
    |server, uri| server.range_formatting(range_formatting_params_with_range(&uri, range()))
);

enabled_out_of_range_request_returns_none!(
    hover_out_of_range_returns_none,
    &[("hover", true)],
    |server, uri| {
        server.hover(HoverParams {
            text_document_position_params: out_of_range_pos(&uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
        })
    }
);
enabled_out_of_range_request_returns_none!(
    completion_out_of_range_returns_none,
    &[("completion", true)],
    |server, uri| {
        server.completion(CompletionParams {
            text_document_position: out_of_range_pos(&uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })
    }
);
enabled_out_of_range_request_returns_none!(
    definition_out_of_range_returns_none,
    &[("definition", true)],
    |server, uri| {
        server.goto_definition(DefParams {
            text_document_position_params: out_of_range_pos(&uri),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        })
    }
);
enabled_out_of_range_request_returns_none!(
    references_out_of_range_returns_none,
    &[("references", true)],
    |server, uri| server.references(out_of_range_reference_params(&uri))
);
enabled_out_of_range_request_returns_none!(
    document_highlight_out_of_range_returns_none,
    &[("references", true)],
    |server, uri| server.document_highlight(out_of_range_document_highlight_params(&uri))
);
enabled_out_of_range_request_returns_none!(
    code_action_out_of_range_returns_none,
    &[("lint", true), ("codeActions", true)],
    |server, uri| server.code_action(out_of_range_code_action_params(&uri))
);
enabled_out_of_range_request_returns_none!(
    prepare_rename_out_of_range_returns_none,
    &[("rename", true)],
    |server, uri| server.prepare_rename(out_of_range_pos(&uri))
);
enabled_out_of_range_request_returns_none!(
    rename_out_of_range_returns_none,
    &[("rename", true)],
    |server, uri| server.rename(out_of_range_rename_params(&uri))
);
enabled_out_of_range_request_returns_none!(
    range_formatting_out_of_range_returns_none,
    &[("formatting", true)],
    |server, uri| {
        server.range_formatting(range_formatting_params_with_range(
            &uri,
            out_of_range_range(),
        ))
    }
);
