use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use corsa::jsonrpc::InboundEvent;
use corsa::lsp::{LspClient, LspSpawnConfig, VirtualDocument};
use lsp_types::Uri;
use serde_json::{Value, json};
use vize_carton::String as CompactString;

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";

struct StopOnDrop<'a>(&'a AtomicBool);

impl Drop for StopOnDrop<'_> {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

struct RawInitialize;
struct RawDiscoverContentMappers;
struct RawCompletion;
struct RawSignatureHelp;
struct RawHover;
struct RawDefinition;
struct RawReferences;
struct RawRename;

macro_rules! raw_request {
    ($request:ty, $method:literal) => {
        impl lsp_types::request::Request for $request {
            type Params = Value;
            type Result = Value;
            const METHOD: &'static str = $method;
        }
    };
}

raw_request!(RawInitialize, "initialize");
raw_request!(RawDiscoverContentMappers, "custom/discoverContentMappers");
raw_request!(RawCompletion, "textDocument/completion");
raw_request!(RawSignatureHelp, "textDocument/signatureHelp");
raw_request!(RawHover, "textDocument/hover");
raw_request!(RawDefinition, "textDocument/definition");
raw_request!(RawReferences, "textDocument/references");
raw_request!(RawRename, "textDocument/rename");

struct RawInitialized;

impl lsp_types::notification::Notification for RawInitialized {
    type Params = Value;
    const METHOD: &'static str = "initialized";
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "node_modules" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).unwrap();
        }
    }
}

fn install_packages(project_root: &Path) {
    let mapper_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&mapper_root).unwrap();
    std::fs::write(
        mapper_root.join("package.json"),
        serde_json::to_vec_pretty(&json!({
            "name": "vize",
            "private": true,
            "tsContentMapper": {
                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "compilerOptions": ["noUnusedLocals"],
            },
        }))
        .unwrap(),
    )
    .unwrap();

    let store = workspace_root().join("node_modules/.pnpm");
    let mut candidates = std::fs::read_dir(&store)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
        .map(|entry| entry.path().join("node_modules/vue"))
        .filter(|path| path.join("package.json").is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    let source = candidates.pop().expect("workspace Vue package");
    let target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn file_uri(path: &Path) -> CompactString {
    let mut uri = CompactString::from("file://");
    for byte in path.to_string_lossy().bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                uri.push(byte as char)
            }
            _ => {
                use std::fmt::Write;
                write!(uri, "%{byte:02X}").unwrap();
            }
        }
    }
    uri
}

fn position(source: &str, offset: usize) -> Value {
    let before = &source[..offset];
    let line = before.matches('\n').count();
    let character = before.rsplit_once('\n').map_or(before, |(_, tail)| tail);
    json!({ "line": line, "character": character.encode_utf16().count() })
}

fn contains_location(value: &Value, uri: &str, start: &Value) -> bool {
    match value {
        Value::Array(values) => values
            .iter()
            .any(|value| contains_location(value, uri, start)),
        Value::Object(object) => {
            let location_uri = object.get("uri").or_else(|| object.get("targetUri"));
            let range = object
                .get("range")
                .or_else(|| object.get("targetSelectionRange"));
            location_uri == Some(&Value::String(uri.to_owned()))
                && range.and_then(|range| range.get("start")) == Some(start)
        }
        _ => false,
    }
}

fn editor_capabilities() -> Value {
    json!({
        "workspace": {
            "configuration": true,
            "didChangeWatchedFiles": { "dynamicRegistration": true }
        },
        "textDocument": {
            "synchronization": { "dynamicRegistration": true },
            "diagnostic": { "dynamicRegistration": true },
            "completion": { "dynamicRegistration": true },
            "signatureHelp": { "dynamicRegistration": true },
            "hover": { "dynamicRegistration": true },
            "definition": { "dynamicRegistration": true },
            "references": { "dynamicRegistration": true },
            "rename": { "dynamicRegistration": true }
        }
    })
}

#[test]
fn standard_tsgo_lsp_maps_core_symbol_features_to_authored_vue() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper LSP conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-lsp-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let child_path = project.path().join("src/Child.vue");
    let source = std::fs::read_to_string(&child_path).unwrap();
    let child_uri = file_uri(&child_path);
    let root_uri = file_uri(project.path());
    let symbol_offset = source.rfind("count.toFixed").unwrap();
    let symbol_position = position(&source, symbol_offset + 1);
    let completion_position = position(&source, symbol_offset + "count.".len());
    let signature_position = position(&source, symbol_offset + "count.toFixed(".len());
    let declaration_offset = source.find("count: number").unwrap();
    let declaration_position = position(&source, declaration_offset);

    let stop = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let _stop_on_drop = StopOnDrop(&stop);
        corsa::runtime::block_on(async {
            let client = LspClient::spawn(
                LspSpawnConfig::new(&tsgo)
                    .with_cwd(project.path())
                    .with_request_timeout(Some(Duration::from_secs(30))),
            )
            .await
            .unwrap();
            let responder_client = client.clone();
            let events = responder_client.subscribe();
            let stop_ref = &stop;
            let responder = scope.spawn(move || {
                while !stop_ref.load(Ordering::Relaxed) {
                    if let Ok(InboundEvent::Request { id, method, params }) =
                        events.recv_timeout(Duration::from_millis(50))
                    {
                        let result = if method.as_str() == "workspace/configuration" {
                            let count = params
                                .get("items")
                                .and_then(Value::as_array)
                                .map_or(0, Vec::len);
                            Value::Array(vec![Value::Null; count])
                        } else {
                            Value::Null
                        };
                        let _ = responder_client.respond(id, result);
                    }
                }
            });
            let initialize = client
                .request::<RawInitialize>(json!({
                    "processId": std::process::id(),
                    "rootUri": root_uri,
                    "workspaceFolders": [{ "uri": root_uri, "name": "content-mapper-lsp" }],
                    "capabilities": editor_capabilities(),
                    "initializationOptions": { "loadExternalPlugins": true }
                }))
                .await
                .unwrap();
            assert!(initialize["capabilities"].is_object(), "{initialize:#}");
            client.notify::<RawInitialized>(json!({})).unwrap();

            let discovered = client
                .request::<RawDiscoverContentMappers>(json!({
                    "textDocuments": [{ "uri": child_uri }],
                    "extensions": [".vue"]
                }))
                .await
                .unwrap();
            assert_eq!(discovered["extensions"], json!([".vue"]), "{discovered:#}");

            let uri = Uri::from_str(&child_uri).unwrap();
            client
                .overlay()
                .open(VirtualDocument::new(uri, "vue", source.as_str()))
                .unwrap();
            let params =
                json!({ "textDocument": { "uri": child_uri }, "position": symbol_position });

            let completion = client
                .request::<RawCompletion>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": completion_position
                }))
                .await
                .unwrap();
            assert!(
                serde_json::to_string(&completion)
                    .unwrap()
                    .contains("toFixed"),
                "{completion:#}"
            );

            let signature = client
                .request::<RawSignatureHelp>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": signature_position
                }))
                .await
                .unwrap();
            let signature_text = serde_json::to_string(&signature).unwrap();
            assert!(
                signature_text.contains("fractionDigits") && signature_text.contains("number"),
                "{signature:#}"
            );

            let hover = client.request::<RawHover>(params.clone()).await.unwrap();
            let hover_text = serde_json::to_string(&hover).unwrap();
            assert!(
                hover_text.contains("count") && hover_text.contains("number"),
                "{hover:#}"
            );

            let definition = client
                .request::<RawDefinition>(params.clone())
                .await
                .unwrap();
            let definition_text = serde_json::to_string(&definition).unwrap();
            assert!(
                definition_text.contains(child_uri.as_str()),
                "{definition:#}"
            );
            assert!(
                contains_location(&definition, &child_uri, &declaration_position),
                "{definition:#}"
            );
            assert!(!definition_text.contains(".vue.ts"), "{definition:#}");

            let references = client
                .request::<RawReferences>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": symbol_position,
                    "context": { "includeDeclaration": true }
                }))
                .await
                .unwrap();
            let references_text = serde_json::to_string(&references).unwrap();
            assert!(
                references_text.matches(child_uri.as_str()).count() >= 2,
                "{references:#}"
            );
            assert!(!references_text.contains(".vue.ts"), "{references:#}");

            let rename = client
                .request::<RawRename>(json!({
                    "textDocument": { "uri": child_uri },
                    "position": symbol_position,
                    "newName": "renamedCount"
                }))
                .await
                .unwrap();
            let rename_text = serde_json::to_string(&rename).unwrap();
            assert!(rename_text.contains(child_uri.as_str()), "{rename:#}");
            assert!(
                rename_text.matches("renamedCount").count() >= 2,
                "{rename:#}"
            );
            assert!(!rename_text.contains(".vue.ts"), "{rename:#}");

            stop.store(true, Ordering::Relaxed);
            client.close().await.unwrap();
            responder.join().unwrap();
        });
    });
}
