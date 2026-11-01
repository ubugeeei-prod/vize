use super::{MaestroServer, code_actions};
use tower_lsp::{
    LspService,
    lsp_types::{
        ClientCapabilities, CodeActionContext, CodeActionOrCommand, CodeActionParams, Diagnostic,
        DocumentChanges, OneOf, PartialResultParams, TextDocumentContentChangeEvent,
        TextDocumentIdentifier, Url, WorkDoneProgressParams, WorkspaceClientCapabilities,
        WorkspaceEditClientCapabilities,
    },
};

const SOURCE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/vize.config.json"
);
const REFERENCE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/code-action-diagnostics/references.json"
);

fn params(uri: &Url, diagnostic: Diagnostic) -> CodeActionParams {
    CodeActionParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        range: diagnostic.range,
        context: CodeActionContext {
            diagnostics: vec![diagnostic],
            ..Default::default()
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}
fn edit(
    uri: &Url,
    line: u32,
    start: u32,
    end: u32,
    text: &str,
    version: Option<i32>,
) -> serde_json::Value {
    let edits = serde_json::json!([{"range":{"start":{"line":line,"character":start},"end":{"line":line,"character":end}},"newText":text}]);
    if let Some(version) = version {
        serde_json::json!({"documentChanges":[{"textDocument":{"uri":uri,"version":version},"edits":edits}]})
    } else {
        serde_json::json!({"changes":{(uri.as_str()):edits}})
    }
}
fn suppression(uri: &Url, diagnostic: &Diagnostic, version: Option<i32>) -> serde_json::Value {
    let rule = match diagnostic.code.as_ref().unwrap() {
        tower_lsp::lsp_types::NumberOrString::String(x) => x.as_str(),
        _ => panic!("expected original rule"),
    };
    let title = ["Suppress with @vize:forget (", rule, ")"].concat();
    let text = ["  <!-- @vize:forget ", rule, " -->\n"].concat();
    serde_json::json!({"title":title,"kind":"quickfix","diagnostics":[diagnostic],"edit":edit(uri,diagnostic.range.start.line,0,0,&text,version),"isPreferred":false})
}

#[test]
fn selected_no_fix_suppression_keeps_full_diagnostic_and_only_suppresses_that_rule() {
    let original: serde_json::Value = serde_json::from_str(REFERENCE).unwrap();
    let diagnostic: Diagnostic =
        serde_json::from_value(original["diagnostics"][0].clone()).unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("vite.config.mjs"),
        vize_l0::cstr!("export default {{ vue: {} }};", CONFIG).as_bytes(),
    )
    .unwrap();
    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server.state.load_lsp_config(root.path());
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({"typecheck":false})));
    let uri = Url::from_file_path(root.path().join("App.vue")).unwrap();
    server
        .state
        .documents
        .open(uri.clone(), SOURCE.into(), 7, "vue".into());
    let mut observations = Vec::new();
    for capability in [None, Some(false), Some(true)] {
        server
            .state
            .record_client_capabilities(&ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_edit: Some(WorkspaceEditClientCapabilities {
                        document_changes: capability,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            });
        let response =
            futures::executor::block_on(code_actions(server, &params(&uri, diagnostic.clone())));
        observations.push((capability, response));
    }
    eprintln!("whole selected no-fix suppression capability outcomes: {observations:#?}");
    for (capability, response) in observations {
        assert_eq!(
            serde_json::to_value(response.unwrap()).unwrap(),
            serde_json::json!([suppression(
                &uri,
                &diagnostic,
                (capability == Some(true)).then_some(7)
            )])
        );
    }
    assert!(server.state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: SOURCE.into()
        }],
        8
    ));
    let response =
        futures::executor::block_on(code_actions(server, &params(&uri, diagnostic.clone())))
            .unwrap();
    eprintln!("whole selected suppression version-eight response: {response:#?}");
    assert_eq!(
        serde_json::to_value(&response).unwrap(),
        serde_json::json!([suppression(&uri, &diagnostic, Some(8))])
    );
    let CodeActionOrCommand::CodeAction(action) = &response[0] else {
        panic!("expected suppression action")
    };
    let Some(DocumentChanges::Edits(documents)) = &action.edit.as_ref().unwrap().document_changes
    else {
        panic!("expected document edit")
    };
    let OneOf::Left(change) = &documents[0].edits[0] else {
        panic!("expected text insertion")
    };
    let start = crate::ide::position_to_offset(
        SOURCE,
        change.range.start.line,
        change.range.start.character,
    )
    .unwrap();
    let end =
        crate::ide::position_to_offset(SOURCE, change.range.end.line, change.range.end.character)
            .unwrap();
    assert_eq!(start, end);
    let mut applied = SOURCE.to_owned();
    applied.insert_str(start, &change.new_text);
    eprintln!("whole actual applied no-fix suppression: {applied:?}");
    assert_eq!(
        applied,
        "<template>\n  <!-- @vize:forget a11y/alt-text -->\n  <img src=\"/logo.png\">\n</template>\n"
    );
    assert!(server.state.documents.apply_changes(
        &uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: applied.clone()
        }],
        9
    ));
    server.state.update_virtual_docs(&uri, &applied);
    let mut peer: Diagnostic = serde_json::from_value(original["diagnostics"][1].clone()).unwrap();
    peer.range.start.line = 2;
    peer.range.end.line = 2;
    let actual_peer =
        futures::executor::block_on(code_actions(server, &params(&uri, peer.clone())));
    eprintln!("whole fresh peer-rule suppression requery: {actual_peer:#?}");
    assert_eq!(
        serde_json::to_value(actual_peer.unwrap()).unwrap(),
        serde_json::json!([
            {"title":"Fix: Use self-closing syntax","kind":"quickfix","diagnostics":[peer],"edit":edit(&uri,2,2,23,"<img src=\"/logo.png\" />",Some(9)),"isPreferred":true},
            suppression(&uri,&peer,Some(9))
        ])
    );
    let mut suppressed = diagnostic;
    suppressed.range.start.line = 2;
    suppressed.range.end.line = 2;
    let actual_absent =
        futures::executor::block_on(code_actions(server, &params(&uri, suppressed)));
    eprintln!("whole suppressed-rule fresh requery: {actual_absent:#?}");
    assert!(actual_absent.is_none());
    assert_eq!(server.state.documents.text(&uri).unwrap(), applied);
}
