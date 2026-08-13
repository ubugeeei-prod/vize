use super::*;

#[test]
fn signature_help_request_defaults_to_manual_invocation() {
    let uri = Uri::from_str("file:///workspace/App.vue.ts").unwrap();
    let params = signature_help_request_params(&uri, 4, 7, None);

    assert_eq!(
        params["context"],
        serde_json::json!({"triggerKind": 1, "isRetrigger": false})
    );
}

#[test]
fn signature_help_request_preserves_client_context_losslessly() {
    let uri = Uri::from_str("file:///workspace/App.vue.ts").unwrap();
    let context = serde_json::json!({
        "triggerKind": 2,
        "triggerCharacter": ",",
        "isRetrigger": true,
        "activeSignatureHelp": {
            "signatures": [{"label": "format(value: string, radix: number): string"}],
            "activeSignature": 0,
            "activeParameter": 1
        }
    });
    let params = signature_help_request_params(&uri, 8, 13, Some(context.clone()));

    assert_eq!(params["context"], context);
}

#[test]
fn will_rename_files_request_uses_lsp_file_rename_shape() {
    let params = will_rename_files_request_params(&[(
        "file:///workspace/src/Old.vue",
        "file:///workspace/src/New.vue",
    )]);

    assert_eq!(
        params,
        serde_json::json!({
            "files": [{
                "oldUri": "file:///workspace/src/Old.vue",
                "newUri": "file:///workspace/src/New.vue"
            }]
        })
    );
}
