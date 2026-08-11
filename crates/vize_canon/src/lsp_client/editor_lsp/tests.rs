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
