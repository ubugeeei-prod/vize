//! The same owner fixture through the standard upstream editor boundary.
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use corsa_lsp::{LspClient, LspSpawnConfig, VirtualDocument, jsonrpc::InboundEvent};
use lsp_types::Uri;
use serde_json::{Value, json};
use vize_l0::cstr;

use super::{content_mapper_lsp_support as rpc, owners};
use rpc::raw_requests::{RawInitialize, RawInitialized, RawSetContentMapperContributions};

#[test]
fn parent_expect_editor_repairs_preserve_whole_reports() {
    let Some(tsgo) = std::env::var_os(super::TSGO_ENV).map(PathBuf::from) else {
        eprintln!(
            "skipping exact owner LSP conformance: {} is not set",
            super::TSGO_ENV
        );
        return;
    };
    assert!(tsgo.is_file());
    let retained = owners::capture_root().join("lsp");
    std::fs::create_dir_all(&retained).unwrap();
    let mut failures = Vec::new();
    for (session, (newline, prefix)) in [
        ("\n", ""),
        ("\n", "😀日本"),
        ("\r\n", ""),
        ("\r\n", "😀日本"),
    ]
    .into_iter()
    .enumerate()
    {
        run_session(
            &tsgo,
            &retained.join(cstr!("session-{session}").as_str()),
            newline,
            prefix,
            &mut failures,
        );
    }
    assert!(
        failures.is_empty(),
        "complete reports retained under {}: {failures:#?}",
        retained.display()
    );
}

fn run_session(
    tsgo: &Path,
    retained: &Path,
    newline: &str,
    prefix: &str,
    failures: &mut Vec<String>,
) {
    std::fs::create_dir_all(retained).unwrap();
    let cases = rpc::workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-owner-lsp-")
        .tempdir_in(cases)
        .unwrap();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    rpc::copy_fixture(&fixture, project.path());
    rpc::install_packages(project.path());
    let path = project.path().join("directives/Owner.vue");
    let comp = project.path().join("directives/OwnerComp.vue");
    std::fs::write(&comp, owners::COMPONENT).unwrap();
    let uri_text = rpc::file_uri(&path);
    let comp_uri = rpc::file_uri(&comp);
    let root_uri = rpc::file_uri(project.path());
    let source = owners::source(newline, prefix, "bad", "wrong");
    let calibration = source.replace(cstr!("<!-- @vue-expect-error -->{newline}").as_str(), "");
    std::fs::write(&path, &calibration).unwrap();
    std::fs::write(retained.join("OwnerComp.vue"), owners::COMPONENT).unwrap();
    let stop = AtomicBool::new(false);
    let editor = rpc::EditorResponder::default();
    let transcript = Mutex::new(Vec::<Value>::new());
    std::thread::scope(|scope| {
        let _stop = rpc::StopOnDrop(&stop);
        corsa::runtime::block_on(async {
            let client = LspClient::spawn(
                LspSpawnConfig::new(tsgo)
                    .with_cwd(project.path())
                    .with_request_timeout(Some(Duration::from_secs(30))),
            )
            .await
            .unwrap();
            let responding = client.clone();
            let events = responding.subscribe();
            let stop_ref = &stop;
            let editor_ref = &editor;
            let log = &transcript;
            let log_path = retained.join("server-transcript.json");
            let responder = scope.spawn(move || {
                while !stop_ref.load(Ordering::Relaxed) {
                    match events.recv_timeout(Duration::from_millis(50)) {
                        Ok(InboundEvent::Request { id, method, params }) => {
                            let result = editor_ref.respond_to(method.as_str(), &params);
                            log.lock().unwrap().push(json!({"direction":"server-request", "id":id, "method":method, "params":params, "response":result}));
                            write_json(log_path.clone(), &*log.lock().unwrap());
                            responding.respond(id, result).unwrap();
                        }
                        Ok(InboundEvent::Notification { method, params }) => {
                            log.lock().unwrap().push(json!({"direction":"server-notification", "method":method, "params":params}));
                            write_json(log_path.clone(), &*log.lock().unwrap());
                        }
                        Err(_) => {}
                    }
                }
            });
            let initialize = json!({"processId":std::process::id(),"rootUri":root_uri,
                "workspaceFolders":[{"uri":root_uri,"name":"content-mapper-owner"}],
                "capabilities":rpc::editor_capabilities(),"initializationOptions":{"runExternalCode":true}});
            write_json(
                retained.join("initialize.json"),
                &json!({"request":initialize}),
            );
            let initialized = client
                .request::<RawInitialize>(initialize.clone())
                .await
                .unwrap();
            write_json(
                retained.join("initialize.json"),
                &json!({"request":initialize,"response":initialized}),
            );
            assert!(initialized["capabilities"].is_object());
            client.notify::<RawInitialized>(json!({})).unwrap();
            let contribution = json!({"contributions":[{"contributorId":"vize","extensions":[".vue"],
                "inferredProjectContribution":{"options":{},"manifest":{"name":"vize",
                    "exec":[env!("CARGO_BIN_EXE_vize"),"content-mapper"],"compilerOptions":["noUnusedLocals"],
                    "dynamicConfig":true}}}],
                "openDocuments":[{"uri":uri_text},{"uri":comp_uri}]});
            write_json(
                retained.join("contribution.json"),
                &json!({"request":contribution}),
            );
            let contributed = client
                .request::<RawSetContentMapperContributions>(contribution.clone())
                .await
                .unwrap();
            write_json(
                retained.join("contribution.json"),
                &json!({"request":contribution,"response":contributed}),
            );
            assert!(contributed.is_null());
            editor.assert_vue_did_open_registration();
            let uri = Uri::from_str(&uri_text).unwrap();
            let overlay = client.overlay();
            let opened = overlay
                .open(VirtualDocument::new(
                    uri.clone(),
                    "vue",
                    calibration.as_str(),
                ))
                .unwrap();
            let opened_comp = overlay
                .open(VirtualDocument::new(
                    Uri::from_str(&comp_uri).unwrap(),
                    "vue",
                    owners::COMPONENT,
                ))
                .unwrap();
            write_json(
                retained.join("opened.json"),
                &json!({
                    "owner":opened.text_document_item(),"component":opened_comp.text_document_item(),"initialized":{}
                }),
            );
            let mut previous_version = opened.version;
            let repaired = owners::source(newline, prefix, "ok", "correct");
            let no_child = repaired.replace("value.bad", "value");
            let removed =
                no_child.replace(cstr!("<!-- @vue-expect-error -->{newline}").as_str(), "");
            let orphan = removed.replace(
                "</template>",
                &cstr!("<!-- @vue-expect-error -->{newline}</template>"),
            );
            let states = [
                ("calibration", calibration.to_string(), 7),
                ("both-bad", source.to_string(), 3),
                (
                    "prop-fixed",
                    owners::source(newline, prefix, "ok", "wrong").to_string(),
                    3,
                ),
                (
                    "handler-fixed",
                    owners::source(newline, prefix, "bad", "correct").to_string(),
                    3,
                ),
                ("parents-fixed", repaired.to_string(), 4),
                ("both-restored", source.to_string(), 3),
                ("child-fixed", no_child.to_string(), 3),
                ("removed", removed.to_string(), 2),
                ("orphan", orphan.to_string(), 3),
                (
                    "ignore",
                    source.replace("@vue-expect-error", "@vue-ignore"),
                    3,
                ),
                ("skip", source.replace("@vue-expect-error", "@vue-skip"), 2),
            ];
            for (index, (name, source, count)) in states.into_iter().enumerate() {
                let document = overlay.replace(&uri, source.as_str()).unwrap();
                assert!(document.version > previous_version);
                previous_version = document.version;
                let expected = expected_report(&source, name);
                let case = retained.join(cstr!("state-{index}").as_str());
                std::fs::create_dir_all(&case).unwrap();
                std::fs::write(case.join("Owner.vue"), &source).unwrap();
                write_json(case.join("expected.json"), &expected);
                write_json(
                    case.join("old-expected.json"),
                    &old_expected_report(&source, name),
                );
                write_json(
                    case.join("state.json"),
                    &json!({"name":name,"edit":index,"version":document.version,"uri":uri_text,"expectedCount":count,"contract":if [0,9,10].contains(&index) { "dynamic-config-calibration" } else { "bounded-owner-or-repair" },
                        "request":{"method":"textDocument/diagnostic","params":{"textDocument":{"uri":uri_text}}}}),
                );
                let actual = match rpc::try_pull_diagnostics(&client, &uri_text).await {
                    Ok(report) => report,
                    Err(error) => {
                        write_json(
                            case.join("transport-error.json"),
                            &json!({"error":format!("{error:?}")}),
                        );
                        failures.push(format!("{} state-{index}: {error:?}", retained.display()));
                        continue;
                    }
                };
                write_json(case.join("report.json"), &actual);
                if actual != expected {
                    failures.push(format!(
                        "{} state-{index} ({name}): {actual:#}",
                        retained.display()
                    ));
                }
            }
            let closed = client.graceful_close().await;
            write_json(
                retained.join("shutdown.json"),
                &json!({"success":closed.is_ok(),"result":format!("{closed:?}")}),
            );
            stop.store(true, Ordering::Relaxed);
            responder.join().unwrap();
            closed.unwrap();
        });
    });
    write_json(
        retained.join("server-transcript.json"),
        &*transcript.lock().unwrap(),
    );
}

fn write_json(path: PathBuf, value: &impl serde::Serialize) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

fn diagnostic(
    source: &str,
    offset: usize,
    len: usize,
    code: u32,
    origin: &str,
    message: &str,
) -> Value {
    json!({"range":{"start":rpc::position(source,offset),"end":rpc::position(source,offset+len)},
        "code":code,"severity":1,"source":origin,"message":message})
}

fn expected_report(source: &str, state: &str) -> Value {
    let mut items = Vec::new();
    if ["parents-fixed", "child-fixed", "orphan"].contains(&state) {
        let offset = source.find("@vue-expect-error").unwrap();
        items.push(diagnostic(
            source,
            offset,
            "@vue-expect-error".len(),
            4,
            "vize",
            "Unused '@vue-expect-error' directive",
        ));
    }
    if !["child-fixed", "removed", "orphan", "skip"].contains(&state) {
        let offset = source.find("value.bad").unwrap() + "value.".len();
        items.push(diagnostic(
            source,
            offset,
            3,
            2339,
            "ts",
            "Property 'bad' does not exist on type '1'.",
        ));
    }
    // These are exact, explicitly frozen failures, never a filtered allowlist.
    if state == "calibration" {
        let id = source.find("id=\"bad\"").unwrap();
        items.push(diagnostic(
            source,
            id,
            2,
            2322,
            "ts",
            "Type '\"bad\"' is not assignable to type '\"ok\"'.",
        ));
        let handler = source.find("@save=\"wrong\"").unwrap() + 1;
        items.push(diagnostic(source, handler, 4, 2322, "ts", "Type '(value: string) => void' is not assignable to type '(value: number) => any'.\n  Types of parameters 'value' and 'value' are incompatible.\n    Type 'number' is not assignable to type 'string'."));
        items.push(diagnostic(
            source,
            id,
            2,
            2322,
            "ts",
            "Type '\"bad\"' is not assignable to type '\"ok\"'.",
        ));
    }
    items.extend(parameter_suggestions(source));
    json!({"kind":"full","items":items})
}

fn parameter_suggestions(source: &str) -> Vec<Value> {
    ["const wrong = (", "const correct = ("]
        .into_iter()
        .map(|anchor| {
            let start = source.find(anchor).unwrap() + anchor.len();
            let mut packet = diagnostic(
                source,
                start,
                "value".len(),
                6133,
                "ts",
                "'value' is declared but its value is never read.",
            );
            packet["severity"] = json!(4);
            packet
        })
        .collect()
}

fn old_expected_report(source: &str, state: &str) -> Value {
    if [
        "both-bad",
        "prop-fixed",
        "handler-fixed",
        "parents-fixed",
        "both-restored",
    ]
    .contains(&state)
    {
        // Actual OLD RED: legitimate script suggestions remain, child is lost.
        json!({"kind":"full","items":parameter_suggestions(source)})
    } else {
        expected_report(source, state)
    }
}
