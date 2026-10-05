//! Entire native link frames under real initialized server ownership.
use super::{MaestroServer, build_lsp_service, error, send};
use serde_json::{Value, json};
use tower_lsp::{ClientSocket, LspService, lsp_types::Url};

#[cfg(all(feature = "native", unix))]
mod jsx;
mod lifecycle;
#[cfg(all(feature = "native", unix))]
mod refusals;
#[cfg(all(feature = "native", unix))]
mod transport;

fn initialized(root: Option<&Url>) -> LspService<MaestroServer> {
    let (mut service, socket) = initialize_only(root);
    drop(socket);
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","method":"initialized","params":{}})
        ),
        None
    );
    service
}

fn initialize_only(root: Option<&Url>) -> (LspService<MaestroServer>, ClientSocket) {
    let (mut service, socket) = build_lsp_service();
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "capabilities":{},"rootUri":root,
                "initializationOptions":{"enabled":false}
            }}),
        ),
        Some(json!({"jsonrpc":"2.0","id":1,"result":{
            "capabilities":{
                "textDocumentSync":{"openClose":true,"change":2,"willSave":false,
                    "willSaveWaitUntil":false,"save":{"includeText":false}},
                "workspace":{"workspaceFolders":{"supported":true,"changeNotifications":true}},
                "experimental":{"vize":{"jsxTypecheck":false}}
            },
            "serverInfo":{"name":"vize-maestro","version":env!("CARGO_PKG_VERSION")}
        }}))
    );
    (service, socket)
}

fn open_at(service: &mut LspService<MaestroServer>, uri: &Url, source: &str, language: &str) {
    assert_eq!(
        send(
            service,
            json!({"jsonrpc":"2.0","method":"textDocument/didOpen","params":{
                "textDocument":{"uri":uri,"languageId":language,"version":17,"text":source}
            }})
        ),
        None
    );
}

fn links(id: i32, uri: &Url) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"vize/nativeModuleDocumentLinks",
        "params":{"textDocument":{"uri":uri}}})
}

#[cfg(feature = "native")]
fn files() -> (tempfile::TempDir, std::path::PathBuf, Url) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("child.ts"),
        "export const alpha=1;export const beta=2;",
    )
    .unwrap();
    std::fs::write(root.join("second.vue"), "<template/>").unwrap();
    let source = Url::from_file_path(root.join("original.ts")).unwrap();
    (dir, root, source)
}

#[cfg(all(feature = "native", unix))]
fn link(line: u32, start: u32, end: u32, target: Url) -> Value {
    json!({"range":{"start":{"line":line,"character":start},
        "end":{"line":line,"character":end}},"target":target})
}

#[cfg(all(feature = "native", unix))]
#[test]
fn module_link_rpc_original_ts_occurrences_have_complete_ordered_utf16_frames() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    open_at(
        &mut service,
        &uri,
        "/*😀*/import './child.ts';\r\nexport {alpha, beta} from './second.vue';\r\nexport * from './child.ts';",
        "typescript",
    );
    assert_eq!(
        send(&mut service, links(2, &uri)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[
            link(0,13,25,Url::from_file_path(root.join("child.ts")).unwrap()),
            link(1,26,40,Url::from_file_path(root.join("second.vue")).unwrap()),
            link(2,14,26,Url::from_file_path(root.join("child.ts")).unwrap())
        ]}))
    );
    // No standard-route switch or capability change follows explicit success.
    assert_eq!(
        send(
            &mut service,
            json!({"jsonrpc":"2.0","id":3,"method":"textDocument/documentLink",
            "params":{"textDocument":{"uri":uri}}})
        ),
        Some(json!({"jsonrpc":"2.0","id":3,"result":null}))
    );
    assert_eq!(
        send(&mut service, links(4, &uri)),
        Some(json!({"jsonrpc":"2.0","id":4,"result":[
            link(0,13,25,Url::from_file_path(root.join("child.ts")).unwrap()),
            link(1,26,40,Url::from_file_path(root.join("second.vue")).unwrap()),
            link(2,14,26,Url::from_file_path(root.join("child.ts")).unwrap())
        ]}))
    );
}

#[cfg(all(feature = "native", unix))]
#[test]
fn module_link_rpc_original_js_escapes_use_authored_ranges_and_decoded_file_requests() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    open_at(
        &mut service,
        &uri,
        "import '\\x2e/child.ts';\r\nimport \"./\\\r\nchild.ts\";",
        "javascript",
    );
    assert_eq!(
        send(&mut service, links(2, &uri)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[
            link(0,7,22,Url::from_file_path(root.join("child.ts")).unwrap()),
            {"range":{"start":{"line":1,"character":7},"end":{"line":2,"character":9}},
             "target":Url::from_file_path(root.join("child.ts")).unwrap()}
        ]}))
    );
}

#[cfg(all(feature = "native", unix))]
#[test]
fn module_link_rpc_valid_nonempty_local_program_returns_only_context_guarded_empty_links() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    open_at(&mut service, &uri, "const local=1;local;", "javascript");
    assert_eq!(
        send(&mut service, links(2, &uri)),
        Some(json!({"jsonrpc":"2.0","id":2,"result":[]}))
    );
}

#[test]
fn module_link_rpc_missing_actual_context_never_becomes_empty_success() {
    let uri = Url::parse("file:///original.ts").unwrap();
    let mut service = initialized(None);
    open_at(&mut service, &uri, "const local=1;local;", "javascript");
    assert_eq!(
        send(&mut service, links(2, &uri)),
        Some(error(2, -32015, "Native module-link context unavailable"))
    );
}

#[cfg(not(feature = "native"))]
#[test]
fn module_link_rpc_genuine_minimal_build_has_no_implicit_native_context() {
    let root = Url::parse("file:///actual-source-host/").unwrap();
    let uri = root.join("original.ts").unwrap();
    let mut service = initialized(Some(&root));
    open_at(&mut service, &uri, "import './child.ts';", "typescript");
    assert_eq!(
        send(&mut service, links(2, &uri)),
        Some(error(2, -32015, "Native module-link context unavailable"))
    );
}

#[cfg(all(feature = "native", not(unix)))]
#[test]
fn module_link_rpc_nonunix_backend_returns_only_typed_unsupported_platform() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    open_at(&mut service, &uri, "import './child.ts';", "typescript");
    assert_eq!(
        send(&mut service, links(2, &uri)),
        Some(error(
            2,
            -32016,
            "Native module target platform unsupported"
        ))
    );
}
