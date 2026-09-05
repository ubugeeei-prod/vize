use super::*;

#[test]
fn prepare_call_hierarchy_uses_editor_lsp_runtime_items() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let Some(corsa_path) = resolve_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().expect("temp project");
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("tsconfig");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    let source = "export function run(value: string): string { return value }\nrun('x')\n";
    let path = src.join("call-hierarchy.ts");
    std::fs::write(&path, source).expect("source");
    let uri = format!("file://{}", path.display());
    let mut documents = FxHashMap::default();
    documents.insert(uri.clone().into(), source.into());

    let mut session = EditorLspSession::spawn(
        corsa_path.to_str().expect("utf8 path"),
        project.path(),
        project.path(),
    )
    .expect("editor LSP session");
    session.synchronize(&documents).expect("synchronize source");
    let response = session.prepare_call_hierarchy(&uri, 0, "export function ".len() as u32);
    session.shutdown().expect("shutdown");

    let items = response
        .expect("prepareCallHierarchy request should succeed")
        .expect("runtime should return call hierarchy items");
    assert_eq!(items[0]["name"], "run");
    assert_eq!(items[0]["selectionRange"]["start"]["character"], 16);
}

#[test]
fn call_hierarchy_incoming_and_outgoing_use_editor_lsp_runtime_items() {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return;
    }
    let Some(corsa_path) = resolve_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().expect("temp project");
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("tsconfig");
    let src = project.path().join("src");
    std::fs::create_dir_all(&src).expect("src");
    let source = "export function leaf(value: string): string { return value }\n\
export function caller(): string { return leaf('x') }\n\
caller()\n";
    let path = src.join("call-hierarchy.ts");
    std::fs::write(&path, source).expect("source");
    let uri = format!("file://{}", path.display());
    let mut documents = FxHashMap::default();
    documents.insert(uri.clone().into(), source.into());

    let mut session = EditorLspSession::spawn(
        corsa_path.to_str().expect("utf8 path"),
        project.path(),
        project.path(),
    )
    .expect("editor LSP session");
    session.synchronize(&documents).expect("synchronize source");
    let leaf_items = session
        .prepare_call_hierarchy(&uri, 0, "export function ".len() as u32)
        .expect("prepare leaf")
        .expect("leaf call hierarchy items");
    let caller_items = session
        .prepare_call_hierarchy(&uri, 1, "export function ".len() as u32)
        .expect("prepare caller")
        .expect("caller call hierarchy items");
    let incoming = session
        .call_hierarchy_incoming_calls(leaf_items[0].clone())
        .expect("incoming calls request")
        .expect("leaf incoming calls");
    let outgoing = session
        .call_hierarchy_outgoing_calls(caller_items[0].clone())
        .expect("outgoing calls request")
        .expect("caller outgoing calls");
    session.shutdown().expect("shutdown");

    assert!(
        incoming
            .as_array()
            .expect("incoming calls array")
            .iter()
            .any(|call| call["from"]["name"] == "caller"
                && !call["fromRanges"].as_array().unwrap().is_empty()),
        "leaf should have caller incoming call: {incoming:#?}",
    );
    assert!(
        outgoing
            .as_array()
            .expect("outgoing calls array")
            .iter()
            .any(|call| call["to"]["name"] == "leaf"
                && !call["fromRanges"].as_array().unwrap().is_empty()),
        "caller should have leaf outgoing call: {outgoing:#?}",
    );
}

#[test]
fn signature_help_request_defaults_to_manual_invocation() {
    let uri = Uri::from_str("file:///workspace/App.vue.ts").unwrap();
    let params = signature_help_request_params(&uri, 4, 7, None);

    assert_eq!(
        params["context"],
        serde_json::json!({"triggerKind": 1, "isRetrigger": false})
    );
}

fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    vize_s0::corsa_resolver::resolve_corsa_executable(
        vize_s0::corsa_resolver::CorsaResolveRequest {
            project_root: Some(workspace_root),
            ..Default::default()
        },
    )
    .ok()
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
