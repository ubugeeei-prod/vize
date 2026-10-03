//! Independent real backend evidence for the stale-admission failure.

use super::*;
use corsa::api::{ApiMode, ApiSpawnConfig};

#[test]
fn actual_inherited_options_mismatch_retains_the_complete_new_backend_diagnostic() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let executable = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
            project_root: Some(repo),
        },
    )
    .unwrap();
    let root = tempfile::TempDir::new().unwrap();
    let config = root.path().join("tsconfig.json");
    let base = root.path().join("base.json");
    let configuration = r#"{"extends":"./base.json","include":["source.ts"]}"#;
    let initial = r#"{"compilerOptions":{"strict":false,"moduleDetection":"force","types":[],"noEmit":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext"}}"#;
    std::fs::write(&config, configuration).unwrap();
    std::fs::write(&base, initial).unwrap();
    let path = root.path().join("source.ts");
    let source = "const value=null; value.toFixed();";
    std::fs::write(&path, source).unwrap();
    let admission = block_on(ApiClient::spawn(
        ApiSpawnConfig::new(&executable)
            .with_cwd(root.path())
            .with_mode(ApiMode::AsyncJsonRpcStdio),
    ))
    .unwrap();
    let effective = block_on(admission.parse_config_file(
        super::super::super::session::uri_document_identifier(&crate::file_uri::path_to_file_uri(
            &config,
        )),
    ))
    .unwrap();
    assert_eq!(effective.options["strict"], false);
    block_on(admission.close()).unwrap();
    std::fs::write(
        &base,
        initial.replace("\"strict\":false", "\"strict\":true"),
    )
    .unwrap();
    let uri = crate::file_uri::path_to_file_uri(&path);
    let mut editor =
        EditorLspSession::spawn(executable.to_str().unwrap(), root.path(), root.path()).unwrap();
    editor.mirror(&uri, source).unwrap();
    let api = editor
        .diagnosing_api(&uri)
        .unwrap_or_else(|_| panic!("real attachment required"));
    let report = editor.diagnostics(&uri).unwrap();
    let raw = serde_json::to_value(report).unwrap();
    assert_eq!(
        raw["items"],
        serde_json::json!([{
            "range":{"start":{"line":0,"character":18},"end":{"line":0,"character":23}},
            "severity":1,"code":18047,"source":"ts","message":"'value' is possibly 'null'."
        }])
    );
    assert!(matches!(
        api.observe(&uri, &config, &effective.options),
        Err(ConfigurationError::Changed)
    ));
    api.close(&mut editor)
        .unwrap_or_else(|_| panic!("real owner/attachment cleanup required"));
    assert_eq!(std::fs::read(&config).unwrap(), configuration.as_bytes());
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
}
