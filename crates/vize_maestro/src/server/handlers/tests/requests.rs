use super::*;
use tower_lsp::{
    LspService,
    lsp_types::{
        CodeLensParams, DocumentLinkParams, FileRename, FoldingRangeParams, HoverParams,
        InlayHintParams, SemanticTokensRangeParams, SignatureHelpParams, WorkspaceSymbolParams,
    },
};

mod guards_extra;
mod params;
mod responses;

use params::*;

const SAMPLE: &str = "<template>\n  <div>{{ message }}</div>\n</template>\n\
                      <script setup lang=\"ts\">\nconst message = 'hi'\n</script>\n\
                      <style scoped>\n.box { color: red; }\n</style>\n";

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
    signature_help_disabled_returns_none,
    &[("signatureHelp", false)],
    |server, uri| server.signature_help(signature_help_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    signature_help_missing_document_returns_none,
    &[("signatureHelp", true)],
    |server, uri| server.signature_help(signature_help_params(&uri))
);

#[cfg(feature = "native")]
#[test]
fn signature_help_handler_routes_an_authored_vue_position() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["**/*"]}"#,
        )
        .unwrap();
        std::fs::write(
            root.path().join("vize.config.json"),
            serde_json::json!({"typeChecker":{"corsaPath":corsa_path}}).to_string(),
        )
        .unwrap();

        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        server.state.load_workspace_config(root.path());
        server.state.set_workspace_root(root.path().to_path_buf());

        let uri = Url::from_file_path(root.path().join("Handler.vue")).unwrap();
        let source = "<script setup lang=\"ts\">\nfunction format(value: string, precision: number): string { return value.repeat(precision) }\nformat('handler', )\n</script>\n";
        std::fs::write(uri.to_file_path().unwrap(), source).unwrap();
        open_vue(server, &uri, source);

        let marker = "format('handler', ";
        let offset = source.find(marker).unwrap() + marker.len();
        let (line, character) = crate::ide::offset_to_position(source, offset);
        let mut params = signature_help_params(&uri);
        params.text_document_position_params.position = Position::new(line, character);

        let help = server
            .signature_help(params)
            .await
            .unwrap()
            .expect("handler should return signature help");
        assert_eq!(help.active_parameter, Some(1));
        assert!(help.signatures[0].label.contains("precision: number"));

        if let Some(bridge) = server.state.get_corsa_bridge().await {
            bridge.shutdown().await.unwrap();
        }
    });
}

#[cfg(feature = "native")]
fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    [
        workspace_root.parent()?.join("corsa-bind/.cache/tsgo"),
        workspace_root
            .parent()?
            .join("corsa-bind/ref/corsa-upstream/.cache/tsgo"),
        workspace_root.join("node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
    .or_else(|| vize_s0::corsa_resolver::discover_corsa_in_ancestors(workspace_root))
}
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
    type_definition_disabled_returns_none,
    &[("definition", false)],
    |server, uri| server.goto_type_definition(type_definition_params(&uri))
);
enabled_missing_doc_request_returns_none!(
    type_definition_missing_document_returns_none,
    &[("definition", true)],
    |server, uri| server.goto_type_definition(type_definition_params(&uri))
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
