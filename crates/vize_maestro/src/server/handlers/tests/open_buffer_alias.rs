//! Whole native-handler locations for editor-open configured alias targets.

use super::*;

#[cfg(feature = "native")]
#[test]
fn definition_handler_keeps_whole_alias_and_relative_open_buffer_locations() {
    crate::runtime::block_on(async {
        for newline in ["\n", "\r\n"] {
            let workspace = tempfile::Builder::new()
                .prefix("alias-子 [赤] #-")
                .tempdir()
                .unwrap();
            let root = workspace.path().canonicalize().unwrap();
            std::fs::write(
                root.join("tsconfig.json"),
                r#"{"compilerOptions":{"paths":{"@ui/*":["./components/*"]}}}"#,
            )
            .unwrap();
            std::fs::create_dir_all(root.join("components")).unwrap();
            let target = root.join("components/子 [赤] #.vue");
            let target_uri = Url::from_file_path(&target).unwrap();
            let (service, _socket) = LspService::new(MaestroServer::new);
            service.inner().state.apply_lsp_initialization_options(Some(&serde_json::json!({"definition":true,"typecheck":false,"lint":false,"ecosystem":false})));
            let server = service.inner();
            let parents = [
                ("RelativeParent.vue", "./components/子 [赤] #.vue"),
                ("AliasParent.vue", "@ui/子 [赤] #.vue"),
            ]
            .map(|(name, specifier)| {
                let uri = Url::from_file_path(root.join(name)).unwrap();
                let source = vize_l0::cstr!(
                    "<script setup lang=\"ts\">\nimport Child from \"{specifier}\";\n</script>\n<template><Child /></template>\n"
                )
                .replace('\n', newline);
                open(server, &uri, &source);
                uri
            });
            let location = serde_json::json!({
                "uri":target_uri,
                "range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}}
            });
            for phase in ["absent", "open", "closed", "saved", "deleted"] {
                match phase {
                    "open" => open(server, &target_uri, "<template />\n"),
                    "closed" => server.state.close_document(&target_uri),
                    "saved" => std::fs::write(&target, "<template />\n").unwrap(),
                    "deleted" => std::fs::remove_file(&target).unwrap(),
                    _ => {}
                }
                let expected = if matches!(phase, "open" | "saved") {
                    location.clone()
                } else {
                    serde_json::Value::Null
                };
                for parent in &parents {
                    let mut params = params(parent);
                    params.text_document_position_params.position = Position::new(3, 12);
                    let response = server.goto_definition(params).await.unwrap();
                    assert_eq!(
                        serde_json::to_value(response).unwrap(),
                        expected,
                        "whole handler Location: {phase}, {parent}, newline={newline:?}",
                    );
                }
                assert_eq!(target.exists(), phase == "saved");
            }
        }
    });
}

fn open(server: &MaestroServer, uri: &Url, source: &str) {
    server
        .state
        .documents
        .open(uri.clone(), source.to_owned(), 1, "vue".to_owned());
    server.state.update_virtual_docs(uri, source);
}

fn params(uri: &Url) -> DefParams {
    DefParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: uri.clone() },
            position: Position::new(3, 12),
        },
        work_done_progress_params: WorkDoneProgressParams::default(),
        partial_result_params: PartialResultParams::default(),
    }
}
