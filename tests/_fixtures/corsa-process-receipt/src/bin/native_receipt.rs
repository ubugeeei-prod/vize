use corsa_client::ApiClient;
use corsa_core::ProcessObserver;
use corsa_lsp::{InitializeApiSessionParams, LspClient, LspSpawnConfig, jsonrpc::InboundEvent};
use corsa_runtime::block_on;
use lsp_types::{notification, request};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

const EXPECTED: &[u8] = b"error handling method 'exit': EOF\n";

enum RawDiagnostic {}
impl request::Request for RawDiagnostic {
    type Params = Value;
    type Result = Value;
    const METHOD: &'static str = "textDocument/diagnostic";
}
type Events = Arc<Mutex<Vec<Value>>>;
fn record(events: &Events, value: Value) {
    events.lock().unwrap().push(value);
}
async fn raw(api: &ApiClient, events: &Events, method: &str, params: Value) -> Value {
    record(
        events,
        json!({"kind":"apiRequest", "method":method, "params":params}),
    );
    let result = api.raw_json_request(method, params).await.unwrap();
    record(
        events,
        json!({"kind":"apiResponse", "method":method, "result":result}),
    );
    result
}

async fn configured(name: &str, broken: bool, builtin_watcher: bool, keep_open: bool) -> Value {
    let root = Path::new("native-receipts").join(name);
    fs::create_dir_all(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let config = root.join("tsconfig.json");
    let leaf = root.join("Leaf.ts");
    fs::write(&config, "{\"compilerOptions\":{\"strict\":true,\"noEmit\":true,\"types\":[]},\"files\":[\"Leaf.ts\"]}\n").unwrap();
    fs::write(
        &leaf,
        if broken {
            "export const answer: number = \"bad\";\nMissingLeaf;\n"
        } else {
            "export const answer: number = 1;\nMissingLeaf;\n"
        },
    )
    .unwrap();
    let native =
        std::env::var("VIZE_CORSA_PATH").expect("VIZE_CORSA_PATH must name the real native binary");
    let native = Path::new(&native).canonicalize().unwrap();
    let identity: Value = serde_json::from_slice(
        &fs::read(std::env::var("VIZE_CORSA_IDENTITY_FILE").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(identity["canonicalPath"], native.to_str().unwrap());
    let observer = ProcessObserver::default();
    let mut spawn = LspSpawnConfig::new(&native).with_cwd(&root);
    spawn.command = spawn.command.with_process_observer(observer.clone());
    let lsp = LspClient::spawn(spawn).await.unwrap();
    let events: Events = Arc::default();
    let receiver = lsp.subscribe();
    let responder = lsp.clone();
    let received = events.clone();
    let stop = Arc::new(AtomicBool::new(false));
    let done = stop.clone();
    let handler = thread::spawn(move || {
        while !done.load(Ordering::SeqCst) {
            match receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(InboundEvent::Request { id, method, params }) => {
                    let body = if method == "workspace/configuration" {
                        Value::Array(
                            params["items"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|_| Value::Null)
                                .collect(),
                        )
                    } else {
                        Value::Null
                    };
                    record(
                        &received,
                        json!({"kind":"serverRequest", "id":id, "method":method.as_str(), "params":params, "response":body}),
                    );
                    if let Err(error) = responder.respond(id, body) {
                        record(
                            &received,
                            json!({"kind":"responseError", "error":error.to_string()}),
                        );
                    }
                }
                Ok(InboundEvent::Notification { method, params }) => record(
                    &received,
                    json!({"kind":"serverNotification", "method":method.as_str(), "params":params}),
                ),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),
                Err(_) => break,
            }
        }
    });
    let params = json!({"processId":std::process::id(), "rootPath":root, "rootUri":format!("file://{}", root.display()), "workspaceFolders":[{"uri":format!("file://{}",root.display()),"name":name}], "capabilities":{"textDocument":{"diagnostic":{"dynamicRegistration":false,"relatedDocumentSupport":true,"relatedInformation":true}},"workspace":{"didChangeWatchedFiles":{"dynamicRegistration":!builtin_watcher,"relativePatternSupport":true},"diagnostics":{"refreshSupport":true}}}, "initializationOptions":{"userPreferences":{"tsserver":{"automaticTypeAcquisition":{"enabled":false}},"inlayHints":{"variableTypes":{"enabled":true}}}}});
    record(
        &events,
        json!({"kind":"lspRequest", "method":"initialize", "params":params}),
    );
    let initialized = lsp
        .request::<request::Initialize>(serde_json::from_value(params).unwrap())
        .await
        .unwrap();
    record(
        &events,
        json!({"kind":"lspResponse", "method":"initialize", "result":initialized}),
    );
    lsp.notify::<notification::Initialized>(lsp_types::InitializedParams {})
        .unwrap();
    record(
        &events,
        json!({"kind":"lspNotification", "method":"initialized", "params":{}}),
    );
    let session = lsp
        .initialize_api_session(InitializeApiSessionParams::default())
        .await
        .unwrap();
    record(
        &events,
        json!({"kind":"lspResponse", "method":"custom/initializeAPISession", "result":session}),
    );
    let api = ApiClient::connect_pipe(session.pipe.as_str())
        .await
        .unwrap();
    let opened = raw(
        &api,
        &events,
        "updateSnapshot",
        json!({"openProjects":[config]}),
    )
    .await;
    raw(
        &api,
        &events,
        "release",
        json!({"handle":opened["snapshot"],"snapshot":opened["snapshot"]}),
    )
    .await;
    let document_uri = format!("file://{}", leaf.display());
    let open = json!({"textDocument":{"uri":document_uri,"languageId":"typescript","version":1,"text":fs::read_to_string(&leaf).unwrap()}});
    record(
        &events,
        json!({"kind":"lspNotification", "method":"textDocument/didOpen", "params":open}),
    );
    lsp.notify::<notification::DidOpenTextDocument>(serde_json::from_value(open).unwrap())
        .unwrap();
    let params = json!({"textDocument":{"uri":document_uri}});
    record(
        &events,
        json!({"kind":"lspRequest", "method":"textDocument/diagnostic", "params":params}),
    );
    let diagnostics = lsp.request::<RawDiagnostic>(params).await.unwrap();
    let diagnostics = serde_json::to_value(diagnostics).unwrap();
    record(
        &events,
        json!({"kind":"lspResponse", "method":"textDocument/diagnostic", "result":diagnostics}),
    );
    let snapshot = raw(&api, &events, "updateSnapshot", json!({})).await["snapshot"].clone();
    let project = raw(
        &api,
        &events,
        "getDefaultProjectForFile",
        json!({"snapshot":snapshot,"file":leaf}),
    )
    .await;
    let normalized = raw(
        &api,
        &events,
        "parseConfigFile",
        json!({"file":project["configFileName"]}),
    )
    .await;
    let sources = raw(
        &api,
        &events,
        "getSourceFileNames",
        json!({"snapshot":snapshot,"project":project["id"]}),
    )
    .await;
    raw(
        &api,
        &events,
        "release",
        json!({"handle":snapshot,"snapshot":snapshot}),
    )
    .await;
    assert_eq!(project["compilerOptions"]["strict"], true);
    assert_eq!(project["configFileName"], config.to_str().unwrap());
    assert_eq!(project["rootFiles"], json!([leaf]));
    assert_eq!(sources.as_array().unwrap().len(), 83);
    let authored: Vec<_> = sources
        .as_array()
        .unwrap()
        .iter()
        .filter(|source| Path::new(source.as_str().unwrap()).starts_with(&root))
        .cloned()
        .collect();
    assert_eq!(authored, vec![json!(leaf)]);
    let codes: Vec<_> = diagnostics["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["code"].as_i64().unwrap())
        .collect();
    assert_eq!(codes, if broken { vec![2322, 2304] } else { vec![2304] });
    let close_started = std::time::Instant::now();
    let closed = if keep_open {
        lsp.request_without_params::<request::Shutdown>()
            .await
            .unwrap();
        record(
            &events,
            json!({"kind":"lspResponse", "method":"shutdown", "result":null, "paramsOmitted":true}),
        );
        lsp.notify_without_params::<notification::Exit>().unwrap();
        record(
            &events,
            json!({"kind":"lspNotification", "method":"exit", "paramsOmitted":true}),
        );
        thread::sleep(Duration::from_millis(30));
        lsp.close().await
    } else {
        lsp.graceful_close().await
    };
    record(
        &events,
        json!({"kind":"sdkCloseResult", "method":if keep_open {"close"} else {"graceful_close"}, "result":format!("{closed:?}"),"elapsedNs":close_started.elapsed().as_nanos()}),
    );
    let receipts = observer.receipts();
    assert_eq!(receipts.len(), 1);
    let receipt = &receipts[0];
    fs::write(root.join("stderr.bin"), &receipt.stderr).unwrap();
    let status = receipt.exit_status.unwrap();
    let result = json!({"name":name,"native":native,"nativeIdentity":identity,"pid":receipt.pid,"completed":receipt.completed,"wholeDiagnostics":diagnostics,"selectedProject":project,"normalizedConfig":normalized,"sourceFiles":sources,"stderrBytes":receipt.stderr,"stderrComplete":receipt.stderr_complete,"stderrError":receipt.stderr_error,"processError":receipt.process_error,"exitCode":status.code(),"statusDebug":format!("{status:?}"),"timedOut":receipt.timed_out,"killRequested":receipt.kill_requested,"reaped":receipt.reaped,"closeResult":format!("{closed:?}"),"closeToReapMs":close_started.elapsed().as_secs_f64()*1000.0});
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    stop.store(true, Ordering::SeqCst);
    handler.join().unwrap();
    let attachment_close = api.close().await;
    record(
        &events,
        json!({"kind":"attachmentClosedAfterOwnerReaped", "result":format!("{attachment_close:?}")}),
    );
    fs::write(
        root.join("semantic-events.json"),
        serde_json::to_vec_pretty(&*events.lock().unwrap()).unwrap(),
    )
    .unwrap();
    assert!(receipt.completed && receipt.stderr_complete && receipt.reaped);
    assert!(!receipt.timed_out && !receipt.kill_requested);
    assert_eq!(receipt.stderr_error, None);
    assert_eq!(receipt.process_error, None);
    if !builtin_watcher && !keep_open {
        assert_eq!(status.code(), Some(0));
        assert_eq!(receipt.stderr, EXPECTED);
        assert!(closed.is_ok());
    }
    if keep_open {
        assert_eq!(status.code(), Some(1));
        assert_eq!(receipt.stderr, b"context canceled\n");
    }
    if builtin_watcher && status.code() == Some(2) {
        assert!(closed.is_err());
        assert_eq!(
            receipt.stderr.split(|byte| *byte == b'\n').next().unwrap(),
            b"panic: assignment to entry in nil map [recovered, repanicked]"
        );
    }
    println!(
        "{name}: status={:?} stderr={}B codes={codes:?} closed={closed:?}",
        status.code(),
        receipt.stderr.len()
    );
    result
}
fn main() {
    block_on(async {
        let mut results = Vec::new();
        results.push(configured("configured-broken", true, false, false).await);
        results.push(configured("configured-repaired", false, false, false).await);
        results.push(configured("configured-keepopen", true, false, true).await);
        for i in 0..12 {
            let result = configured(&format!("configured-builtin-{i}"), true, true, false).await;
            let panic = result["exitCode"] == 2;
            results.push(result);
            if panic {
                break;
            }
        }
        assert_eq!(
            results
                .iter()
                .filter(|result| result["exitCode"] == 2)
                .count(),
            1,
            "the genuine native panic control must reproduce"
        );
        fs::write(
            "native-receipts/summary.json",
            serde_json::to_vec_pretty(&results).unwrap(),
        )
        .unwrap();
    });
}
