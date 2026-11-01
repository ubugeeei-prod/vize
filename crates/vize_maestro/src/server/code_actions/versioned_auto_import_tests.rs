//! Real server dispatch retains default imports and versions advertised edits.

use tower_lsp::{
    LspService,
    lsp_types::{
        ClientCapabilities, CodeActionContext, CodeActionOrCommand, CodeActionParams,
        DocumentChanges, OneOf, PartialResultParams, Position, Range,
        TextDocumentContentChangeEvent, TextDocumentIdentifier, Url, WorkDoneProgressParams,
        WorkspaceClientCapabilities, WorkspaceEditClientCapabilities,
    },
};

use super::{MaestroServer, code_actions};

const SOURCE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/auto-import-quoted-symlink-3952/Page.vue.txt"
);
const COMPONENT: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/auto-import-quoted-symlink-3952/MyButton.vue.txt"
);

fn expected(
    uri: &Url,
    specifier: &str,
    statement: &str,
    advertised: bool,
    version: i32,
) -> serde_json::Value {
    let edits = serde_json::json!([{"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}},"newText":statement}]);
    let edit = if advertised {
        serde_json::json!({"documentChanges":[{"textDocument":{"uri":uri,"version":version},"edits":edits}]})
    } else {
        serde_json::json!({"changes":{(uri.as_str()):edits}})
    };
    serde_json::json!([{"title":format!("Auto-import `MyButton` from `{specifier}`"),"kind":"quickfix","edit":edit,"isPreferred":true}])
}

fn exercise(quoted: bool) {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("app");
    std::fs::create_dir(&directory).unwrap();
    let indexed = directory.join("MyButton.vue");
    if quoted {
        #[cfg(unix)]
        {
            let parent = root.path().join("owner's-components");
            std::fs::create_dir(&parent).unwrap();
            let target = parent.join("MyButton.vue");
            std::fs::write(&target, COMPONENT).unwrap();
            std::os::unix::fs::symlink(&target, &indexed).unwrap();
            assert_eq!(
                std::fs::canonicalize(&indexed).unwrap(),
                std::fs::canonicalize(&target).unwrap()
            );
        }
        #[cfg(not(unix))]
        unreachable!("the quoted-symlink test is restricted to Unix");
    } else {
        std::fs::write(&indexed, COMPONENT).unwrap();
    }
    let (specifier, statement) = if quoted {
        (
            "../owner's-components/MyButton.vue",
            "import MyButton from \"../owner's-components/MyButton.vue\"\n",
        )
    } else {
        ("./MyButton.vue", "import MyButton from './MyButton.vue'\n")
    };
    let (service, _socket) = LspService::new(MaestroServer::new);
    let server = service.inner();
    server
        .state
        .apply_lsp_initialization_options(Some(&serde_json::json!({"typecheck":false})));
    let file = directory.join("Page.vue");
    std::fs::write(&file, SOURCE).unwrap();
    let uri = Url::from_file_path(&file).unwrap();
    server
        .state
        .documents
        .open(uri.clone(), SOURCE.into(), 7, "vue".into());
    server.state.update_virtual_docs(&uri, SOURCE);
    let offset = SOURCE.find("MyButton v-if").unwrap() + 4;
    let (line, character) = crate::ide::offset_to_position(SOURCE, offset);
    let position = Position::new(line, character);
    let params = CodeActionParams {
        text_document: TextDocumentIdentifier { uri: uri.clone() },
        range: Range::new(position, position),
        context: CodeActionContext {
            diagnostics: vec![],
            ..Default::default()
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    };
    let mut observations = Vec::new();
    for advertised in [None, Some(false), Some(true)] {
        server
            .state
            .record_client_capabilities(&ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_edit: Some(WorkspaceEditClientCapabilities {
                        document_changes: advertised,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            });
        let response = futures::executor::block_on(code_actions(server, &params));
        observations.push((advertised, response));
    }
    eprintln!("whole server auto-import capability outcomes: {observations:#?}");
    for (advertised, response) in observations {
        assert_eq!(
            serde_json::to_value(response.unwrap()).unwrap(),
            expected(&uri, specifier, statement, advertised == Some(true), 7)
        );
        assert_eq!(server.state.documents.text(&uri).unwrap(), SOURCE);
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
    let response = futures::executor::block_on(code_actions(server, &params)).unwrap();
    eprintln!("whole version-eight auto-import response: {response:#?}");
    assert_eq!(
        serde_json::to_value(&response).unwrap(),
        expected(&uri, specifier, statement, true, 8)
    );
    let CodeActionOrCommand::CodeAction(action) = &response[0] else {
        panic!("expected the whole auto-import action")
    };
    assert!(action.edit.as_ref().unwrap().changes.is_none());
    let Some(DocumentChanges::Edits(documents)) = &action.edit.as_ref().unwrap().document_changes
    else {
        panic!("expected the advertised versioned document edit");
    };
    let OneOf::Left(edit) = &documents[0].edits[0] else {
        panic!("expected an unannotated import insertion");
    };
    let start =
        crate::ide::position_to_offset(SOURCE, edit.range.start.line, edit.range.start.character)
            .unwrap();
    let end = crate::ide::position_to_offset(SOURCE, edit.range.end.line, edit.range.end.character)
        .unwrap();
    assert_eq!(start, end);
    let mut applied = SOURCE.to_owned();
    applied.insert_str(start, &edit.new_text);
    eprintln!("whole source after applying the actual import edit: {applied:?}");
    assert_eq!(applied, SOURCE.replacen('\n', &format!("\n{statement}"), 1));
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
    let offset = applied.find("MyButton v-if").unwrap() + 4;
    let (line, character) = crate::ide::offset_to_position(&applied, offset);
    let position = Position::new(line, character);
    let mut requery = params;
    requery.range = Range::new(position, position);
    let clean = futures::executor::block_on(code_actions(server, &requery));
    let current_source = server.state.documents.text(&uri).unwrap();
    eprintln!("whole applied auto-import document: {current_source:?}; requery: {clean:#?}");
    assert!(clean.is_none());
    assert_eq!(current_source, applied);
    assert_eq!(std::fs::read_to_string(file).unwrap(), SOURCE);
    assert_eq!(std::fs::read_to_string(indexed).unwrap(), COMPONENT);
}

#[test]
fn only_advertised_clients_get_the_actual_auto_import_document_version() {
    exercise(false);
}

#[test]
#[cfg(unix)]
fn advertised_quoted_symlink_import_preserves_the_whole_encoded_action() {
    exercise(true);
}
