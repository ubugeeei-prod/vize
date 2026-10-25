//! Same public source observer copied unchanged into the pinned before role.
//! It never injects a candidate decoder into the before source.
use super::hash;
use crate::{
    file_uri::path_to_file_uri,
    lsp_client::{editor_lsp::EditorLspSession, initial_diagnostics::native_diagnostics},
};
use corsa::runtime::block_on;
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const VALID: &str = "/*🧭é日本語\u{2028}\u{2029}*/\r\nexport const label: string = \"ok\";\r\nexport const length: number = label.length;\r\nexport const echo = label;\r\n";
const NEIGHBOR: &str = "export const outside: string = 42;\r\n";

#[test]
#[ignore = "actual pinned 7.0.2/beta native source before/after Actions only"]
fn actual_sdk_semantic_error_repair_navigation_and_two_file_scope_are_complete() {
    let capture = PathBuf::from(std::env::var_os("VIZE_NATIVE_DTO_CAPTURE").unwrap());
    std::fs::create_dir_all(&capture).unwrap();
    let expected_source = std::env::var("VIZE_NATIVE_DTO_SOURCE_SHA").unwrap();
    assert_eq!(
        option_env!("VIZE_NATIVE_DTO_SOURCE_SHA"),
        Some(expected_source.as_str())
    );
    let actual_source = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(actual_source.status.success());
    assert_eq!(
        std::str::from_utf8(&actual_source.stdout).unwrap().trim(),
        expected_source
    );
    let wrapper = PathBuf::from(std::env::var_os("VIZE_NATIVE_DTO_RUNTIME_PATH").unwrap());
    let executable =
        PathBuf::from(std::env::var_os("VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH").unwrap())
            .canonicalize()
            .unwrap();
    let expected_version = std::env::var("VIZE_NATIVE_DTO_SDK_VERSION").unwrap();
    let expected_hash = std::env::var("VIZE_NATIVE_DTO_SDK_SHA256").unwrap();
    assert_eq!(
        hash(&std::fs::read(&executable).unwrap()).as_str(),
        expected_hash
    );
    let version = Command::new(&wrapper).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(
        version.stdout,
        vize_l0::cstr!("Version {expected_version}\n").as_bytes()
    );
    assert!(version.stderr.is_empty());
    let before = std::env::var("VIZE_NATIVE_DTO_ROLE").unwrap() == "before";
    let expect_refusal = before && expected_version == "7.1.0-beta";
    let test_exe = std::env::current_exe().unwrap();
    let work = tempfile::tempdir().unwrap();
    let root = work.path().canonicalize().unwrap();
    let source = root.join("source.ts");
    let neighbor = root.join("neighbor.ts");
    let config = root.join("tsconfig.json");
    let config_text=br#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","types":[],"noEmit":true},"files":["source.ts","neighbor.ts"]}"#;
    std::fs::write(&config, config_text).unwrap();
    std::fs::write(&source, VALID).unwrap();
    std::fs::write(&neighbor, NEIGHBOR).unwrap();
    let source_uri = path_to_file_uri(&source);
    let neighbor_uri = path_to_file_uri(&neighbor);
    let mut editor =
        EditorLspSession::spawn_with_config(wrapper.to_str().unwrap(), &root, &root, Some(&config))
            .unwrap();
    editor.mirror(&source_uri, VALID).unwrap();
    editor.mirror(&neighbor_uri, NEIGHBOR).unwrap();
    editor.ready_workspace_request().unwrap();
    let client = editor.configured_api.as_ref().unwrap().client.clone();
    assert_eq!(VALID.lines().nth(2).unwrap().find("label"), Some(30));
    let invalid = VALID.replace("label.length", "label");
    let repaired = VALID.replace("\"ok\"", "\"repaired\"");
    let mut rows = Vec::new();
    let mut held = None;
    for (phase, text, error) in [
        ("valid", VALID, false),
        ("error", invalid.as_str(), true),
        ("repair", repaired.as_str(), false),
    ] {
        editor.mirror(&source_uri, text).unwrap();
        editor.ready_workspace_request().unwrap();
        let snapshot = block_on(client.update_snapshot(Default::default())).unwrap();
        let [project] = snapshot.projects.as_slice() else {
            panic!("exactly one owned native project required")
        };
        assert_eq!(Path::new(&project.config_file_name), config);
        let started = std::time::Instant::now();
        let lower = native_diagnostics::read(&client, &snapshot, project, &source_uri);
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        let pull = editor.diagnostics(&source_uri).unwrap();
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(pull)) = pull
        else {
            panic!("whole native pull required")
        };
        let expected = if error {
            vec![
                json!({"range":{"start":{"line":2,"character":13},"end":{"line":2,"character":19}},
            "severity":1,"code":2322,"source":"ts","message":"Type 'string' is not assignable to type 'number'."}),
            ]
        } else {
            vec![]
        };
        assert_eq!(
            serde_json::to_value(&pull.full_document_diagnostic_report.items).unwrap(),
            json!(expected)
        );
        let lower_packet = match lower {
            Ok(Some(items)) => {
                assert!(
                    !expect_refusal,
                    "the before beta must retain its actual strict DTO failure"
                );
                assert_eq!(serde_json::to_value(&items).unwrap(), json!(expected));
                json!({"complete":items})
            }
            other => {
                assert!(
                    expect_refusal,
                    "source adapter must complete, not fall through: {other:?}"
                );
                let Err(message) = other else {
                    panic!("before beta must fail strict decoding, not claim unsupported")
                };
                assert!(
                    message.contains("Malformed") && message.contains("unknown field"),
                    "{message}"
                );
                json!({"refused":message})
            }
        };
        let hover = editor
            .hover(&source_uri, 1, 14)
            .unwrap()
            .expect("whole hover");
        assert_eq!(
            hover,
            json!({"contents":{"kind":"markdown","value":"```typescript\nconst label: string\n```\n"},
            "range":{"start":{"line":1,"character":13},"end":{"line":1,"character":18}}})
        );
        let references = editor
            .references(&source_uri, 1, 14, true)
            .unwrap()
            .expect("whole references");
        assert_eq!(
            references,
            json!([
                {"uri":source_uri,"range":{"start":{"line":1,"character":13},"end":{"line":1,"character":18}}},
                {"uri":source_uri,"range":{"start":{"line":2,"character":30},"end":{"line":2,"character":35}}},
                {"uri":source_uri,"range":{"start":{"line":3,"character":20},"end":{"line":3,"character":25}}}
            ])
        );
        let encoded = block_on(client.get_source_file(
            snapshot.handle.clone(),
            project.id.clone(),
            source.to_str().unwrap(),
        ))
        .unwrap()
        .unwrap();
        std::fs::write(
            capture.join(vize_l0::cstr!("{phase}.source-file.bin").as_str()),
            encoded.as_bytes(),
        )
        .unwrap();
        let decoded = super::super::decode_snapshot_text(&encoded, source.to_str().unwrap());
        if expect_refusal {
            assert_eq!(decoded, Err(super::super::SourceTextRefusal::Version));
        } else {
            assert_eq!(decoded.unwrap(), text);
        }
        rows.push(json!({"phase":phase,"snapshot":snapshot.handle,"project":project,"sourceText":text,
            "lower":lower_packet,"nativePull":pull,"hover":hover,"references":references,"elapsedMs":elapsed,
            "encodedSourceSha256":hash(encoded.as_bytes()),"encodedBytes":encoded.as_bytes().len()}));
        if phase == "error" {
            held = Some(snapshot);
        } else {
            block_on(snapshot.release()).unwrap();
        }
    }
    // The current repair must not replace the retained error snapshot's text.
    let old = held.take().unwrap();
    let project = old.projects.first().unwrap();
    let retained = native_diagnostics::read(&client, &old, project, &source_uri);
    if expect_refusal {
        assert!(retained.is_err());
    } else {
        assert_eq!(
            serde_json::to_value(retained.unwrap().unwrap()).unwrap(),
            rows.get(1).unwrap()["lower"]["complete"]
        );
    }
    // The neighboring error is still real, but can only appear in its own read.
    let current = block_on(client.update_snapshot(Default::default())).unwrap();
    let project = current.projects.first().unwrap();
    let outside = native_diagnostics::read(&client, &current, project, &neighbor_uri);
    if expect_refusal {
        assert!(outside.is_err());
    } else {
        assert_eq!(
            serde_json::to_value(outside.unwrap().unwrap()).unwrap(),
            json!([{
            "range":{"start":{"line":0,"character":13},"end":{"line":0,"character":20}},
            "severity":1,"code":2322,"source":"ts","message":"Type 'number' is not assignable to type 'string'."}])
        );
    }
    block_on(current.release()).unwrap();
    block_on(old.release()).unwrap();
    let record = json!({"source":expected_source,"role":std::env::var("VIZE_NATIVE_DTO_ROLE").unwrap(),
        "observerSha256":hash(include_bytes!("native_beta.rs")),"testExecutable":test_exe,"testExecutableSha256":hash(&std::fs::read(&test_exe).unwrap()),
        "sdkVersion":expected_version,"sdkExecutable":executable,"sdkSha256":expected_hash,"dialect":client.dialect().unwrap().as_str(),
        "root":root,"config":std::str::from_utf8(config_text).unwrap(),"neighbor":NEIGHBOR,"rows":rows});
    std::fs::write(
        capture.join("complete-source-packets.json"),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
    assert_eq!(std::fs::read(&source).unwrap(), VALID.as_bytes());
    assert_eq!(std::fs::read(&neighbor).unwrap(), NEIGHBOR.as_bytes());
    assert_eq!(std::fs::read(&config).unwrap(), config_text);
    assert_eq!(
        hash(&std::fs::read(executable).unwrap()).as_str(),
        expected_hash
    );
    drop(client);
    drop(editor);
}
