//! Same public source observer copied unchanged into the pinned before role.
//! It never injects a candidate decoder into the before source.
use super::hash;
use crate::{
    file_uri::path_to_file_uri,
    lsp_client::{
        CorsaProjectClient, editor_lsp::EditorLspSession, initial_diagnostics::native_diagnostics,
        session::uri_document_identifier,
    },
};
use corsa::{
    api::{FileChangeSummary, FileChanges, UpdateSnapshotParams},
    runtime::block_on,
};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use serde_json::json;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

mod storage_authority;
pub(super) use storage_authority::completion_multiset;

const VALID: &str = "/*🧭é日本語\u{2028}\u{2029}*/\r\nexport const label: string = \"ok\";\r\nexport const length: number = label.length;\r\nexport const echo = label;\r\n";
const NEIGHBOR: &str = "export const outside: string = 42;\r\n";
const BEFORE_BETA_REFUSAL: &str = "Malformed getSemanticDiagnostics response: unknown field `endPosition`, expected one of `fileName`, `pos`, `end`, `code`, `category`, `text`, `messageChain`, `relatedInformation`, `reportsUnnecessary`, `reportsDeprecated`";

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
    let runtime = PathBuf::from(std::env::var_os("VIZE_NATIVE_DTO_RUNTIME_PATH").unwrap());
    let executable =
        PathBuf::from(std::env::var_os("VIZE_NATIVE_DTO_ACTUAL_RUNTIME_PATH").unwrap())
            .canonicalize()
            .unwrap();
    assert_eq!(runtime.canonicalize().unwrap(), executable);
    let expected_version = std::env::var("VIZE_NATIVE_DTO_SDK_VERSION").unwrap();
    let expected_hash = std::env::var("VIZE_NATIVE_DTO_SDK_SHA256").unwrap();
    assert_eq!(
        hash(&std::fs::read(&executable).unwrap()).as_str(),
        expected_hash
    );
    let version = Command::new(&runtime).arg("--version").output().unwrap();
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
        EditorLspSession::spawn_with_config(runtime.to_str().unwrap(), &root, &root, Some(&config))
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
        // This lane owns disk changes explicitly. It does not borrow the
        // editor's snapshot lineage or claim that API snapshots see overlays.
        std::fs::write(&source, text).unwrap();
        editor.mirror(&source_uri, text).unwrap();
        editor.ready_workspace_request().unwrap();
        let snapshot = block_on(client.update_snapshot(UpdateSnapshotParams {
            open_project: Some(config.to_str().unwrap().into()),
            file_changes: Some(FileChanges::Summary(FileChangeSummary {
                changed: vec![uri_document_identifier(&source_uri)],
                ..Default::default()
            })),
            overlay_changes: None,
        }))
        .unwrap();
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
                assert_eq!(message.as_str(), BEFORE_BETA_REFUSAL);
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
    let dialect = client.dialect().unwrap().as_str();
    std::fs::write(&source, VALID).unwrap();
    drop(client);
    drop(editor);
    let mut record = json!({"source":expected_source,"role":std::env::var("VIZE_NATIVE_DTO_ROLE").unwrap(),
        "observerSha256":hash(include_bytes!("native_beta.rs")),"observerDependencies":[{"path":"crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority.rs","sha256":hash(include_bytes!("native_beta/storage_authority.rs"))}],"testExecutable":test_exe,"testExecutableSha256":hash(&std::fs::read(&test_exe).unwrap()),
        "sdkVersion":expected_version,"sdkExecutable":executable,"sdkSha256":expected_hash,"dialect":dialect,
        "scope":"API-owned explicit disk/FileChanges snapshots; attached overlay view is independently tested below",
        "root":root,"config":std::str::from_utf8(config_text).unwrap(),"neighbor":NEIGHBOR,"rows":rows});
    std::fs::write(
        capture.join("owned-api-packets.json"),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .unwrap();
    record["attached"] = attached_overlay_packets(&runtime, &capture, expect_refusal);
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
}

// An unchanged main file can import an unsaved dependency.
fn attached_overlay_packets(
    runtime: &Path,
    capture: &Path,
    before_beta: bool,
) -> serde_json::Value {
    use vize_l0::FxHashMap;
    const MAIN: &str = "/*🧭é日本語\u{2028}\u{2029}*/\r\nimport { value } from \"./support\";\r\nexport const checked: number = value;\r\nexport const echo = value;\r\n";
    const SUPPORT: &str = "export const value = 1;\r\n";
    let work = tempfile::tempdir().unwrap();
    let root = work.path().canonicalize().unwrap();
    let main_path = root.join("main.ts");
    let support_path = root.join("support.ts");
    let config = br#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","types":[],"noEmit":true},"files":["main.ts","support.ts"]}"#;
    std::fs::write(root.join("tsconfig.json"), config).unwrap();
    std::fs::write(&main_path, MAIN).unwrap();
    std::fs::write(&support_path, SUPPORT).unwrap();
    let main_uri = path_to_file_uri(&main_path);
    let support_uri = path_to_file_uri(&support_path);
    let mut client = CorsaProjectClient::new_for_editor_workspace(runtime.to_str(), &root).unwrap();
    assert!(client.project_session_attached_to_editor);
    client
        .did_open_batch_fast(&[(&main_uri, MAIN), (&support_uri, SUPPORT)])
        .unwrap();
    let mut rows = Vec::new();
    for (phase, support, error) in [
        ("valid", SUPPORT, false),
        (
            "dependency-error",
            "export const value = \"oops\";\r\n",
            true,
        ),
        ("dependency-repair", "export const value = 2;\r\n", false),
        ("dependency-close", SUPPORT, false),
    ] {
        if phase == "dependency-close" {
            client.did_close(&support_uri).unwrap();
        } else {
            client.did_change(&support_uri, support).unwrap();
        }
        let mut documents = FxHashMap::default();
        documents.insert(main_uri.clone(), MAIN.into());
        if phase != "dependency-close" {
            documents.insert(support_uri.clone(), support.into());
        }
        // Synchronize and observe the actual LSP independently of route choice.
        let direct = client
            .diagnostics_via_editor_lsp(&main_uri, &documents)
            .unwrap();
        let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(ref pull)) =
            direct
        else {
            panic!("whole attached native pull required")
        };
        let expected = if error {
            json!([{
                "range":{"start":{"line":2,"character":13},"end":{"line":2,"character":20}},
                "severity":1,"code":2322,"source":"ts","message":"Type 'string' is not assignable to type 'number'."
            }])
        } else {
            json!([])
        };
        assert_eq!(
            serde_json::to_value(&pull.full_document_diagnostic_report.items).unwrap(),
            expected
        );
        let selected = client.request_diagnostics_full(&main_uri);
        let selected_packet = match selected {
            Ok(fetch) => {
                assert!(!fetch.used_cache);
                let packet = serde_json::to_value(fetch.diagnostics).unwrap();
                if !before_beta {
                    assert_eq!(packet, expected);
                }
                json!({"complete":packet})
            }
            Err(message) => {
                assert!(
                    before_beta,
                    "attached supported LSP route must complete: {message}"
                );
                json!({"refused":message})
            }
        };
        let api_eligible = client.can_use_api_for_uri(&main_uri);
        let beta = std::env::var("VIZE_NATIVE_DTO_SDK_VERSION").unwrap() == "7.1.0-beta";
        if beta && !before_beta {
            assert!(!api_eligible);
            assert!(
                client
                    .prepare_initial_diagnostics_read(&main_uri)
                    .unwrap()
                    .is_none()
            );
            assert!(
                client
                    .component_types_raw(&main_uri, MAIN, 0, &[])
                    .unwrap()
                    .is_none()
            );
        }
        let batch = client.request_diagnostics_batch(std::slice::from_ref(&main_uri));
        if !before_beta {
            assert_eq!(batch.as_ref().unwrap().len(), 1);
            assert_eq!(
                batch.as_ref().unwrap().first().unwrap().1.len(),
                usize::from(error)
            );
        }
        let hover = client.hover_raw(&main_uri, 2, 31).unwrap().unwrap();
        let direct_hover = client
            .hover_via_editor_lsp(&main_uri, 2, 31)
            .unwrap()
            .unwrap();
        let references = client.references_raw(&main_uri, 2, 31, true);
        let direct_references = client
            .references_via_editor_lsp(&main_uri, 2, 31, true)
            .unwrap()
            .unwrap();
        let definition = client.definition_raw(&main_uri, 2, 31);
        let direct_definition = client
            .definition_via_editor_lsp(&main_uri, 2, 31)
            .unwrap()
            .unwrap();
        let completion = client.completion_raw(&main_uri, 2, 31).unwrap().unwrap();
        let direct_completion = client
            .completion_via_editor_lsp(&main_uri, 2, 31)
            .unwrap()
            .unwrap();
        let rename = client.rename_raw(&main_uri, 2, 31, "renamedValue");
        let direct_rename = client
            .rename_via_editor_lsp(&main_uri, 2, 31, "renamedValue")
            .unwrap();
        assert_eq!(hover, direct_hover);
        // Go map order varies; retain both packets and compare whole multisets.
        assert_eq!(
            completion_multiset(&completion),
            completion_multiset(&direct_completion)
        );
        if !before_beta {
            assert_eq!(
                references.as_ref().unwrap().as_ref().unwrap(),
                &direct_references
            );
            assert_eq!(
                definition.as_ref().unwrap().as_ref().unwrap(),
                &direct_definition
            );
            assert_eq!(rename.as_ref().unwrap(), &direct_rename);
        }
        assert!(direct_references.as_array().unwrap().len() >= 4);
        rows.push(json!({"phase":phase,"mainText":MAIN,"supportOverlay":support,"apiEligible":api_eligible,
            "nativePull":direct,"selected":selected_packet,"hover":hover,"references":references.ok(),
            "directReferences":direct_references,"definition":definition.ok(),"directDefinition":direct_definition,
            "completion":completion,"directCompletion":direct_completion,"rename":rename,"directRename":direct_rename,"batch":batch}));
        std::fs::write(
            capture.join("attached-overlay-progress.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
        assert_eq!(std::fs::read(&main_path).unwrap(), MAIN.as_bytes());
        assert_eq!(std::fs::read(&support_path).unwrap(), SUPPORT.as_bytes());
    }
    // The dependency-close row above proves the original on-disk fallback.
    // Reopen those exact bytes for the complete retained revision. The historical
    // before source materializes it; the current source keeps logical names.
    // Both must carry the original relative dependency without changing disk.
    client.did_open(&support_uri, SUPPORT).unwrap();
    // No original disk file: observe the actual source-owned identity policy.
    let virtual_path = root.join("Virtual.vue.ts");
    let virtual_uri = path_to_file_uri(&virtual_path);
    const VIRTUAL: &str = "export const bad: number = \"x\";\r\n";
    client.did_open(&virtual_uri, VIRTUAL).unwrap();
    let mapped_virtual = client.session_document_uri(&virtual_uri);
    if std::env::var("VIZE_NATIVE_DTO_ROLE").unwrap() == "before" {
        assert_ne!(mapped_virtual, virtual_uri);
    } else {
        assert_eq!(mapped_virtual, virtual_uri);
    }
    let documents = FxHashMap::from_iter([
        (main_uri.clone(), MAIN.into()),
        (virtual_uri.clone(), VIRTUAL.into()),
    ]);
    let virtual_pull = client
        .diagnostics_via_editor_lsp(&virtual_uri, &documents)
        .unwrap();
    let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(ref pull)) =
        virtual_pull
    else {
        panic!("whole virtual native pull required")
    };
    let virtual_expected = json!([{
        "range":{"start":{"line":0,"character":13},"end":{"line":0,"character":16}},
        "severity":1,"code":2322,"source":"ts","message":"Type 'string' is not assignable to type 'number'."
    }]);
    assert_eq!(
        serde_json::to_value(&pull.full_document_diagnostic_report.items).unwrap(),
        virtual_expected
    );
    let whole_batch = client.request_diagnostics_batch(&[main_uri.clone(), virtual_uri.clone()]);
    if !before_beta {
        let rows = whole_batch.as_ref().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows.first().unwrap().0, main_uri);
        assert!(rows.first().unwrap().1.is_empty());
        assert_eq!(rows.last().unwrap().0, virtual_uri);
        assert_eq!(
            serde_json::to_value(&rows.last().unwrap().1).unwrap(),
            virtual_expected
        );
    }
    assert!(!virtual_path.exists());
    let storage_authority = storage_authority::capture(
        &mut client,
        &root,
        &[
            (&main_uri, MAIN),
            (&support_uri, SUPPORT),
            (&virtual_uri, VIRTUAL),
        ],
    );
    client.did_close(&virtual_uri).unwrap();
    client.did_close(&support_uri).unwrap();
    client.did_close(&main_uri).unwrap();
    assert!(client.hover_raw(&main_uri, 2, 31).unwrap().is_none());
    client.shutdown().unwrap();
    let result = json!({"scope":"attached editor; dependency close and whole source-selected virtual revision",
        "root":root,"mainUri":main_uri,"supportUri":support_uri,"config":std::str::from_utf8(config).unwrap(),"rows":rows,
        "materializedBatch":{"virtualUri":virtual_uri,"mappedUri":mapped_virtual,"source":VIRTUAL,"nativePull":virtual_pull,"selected":whole_batch,"storageAuthority":storage_authority}});
    std::fs::write(
        capture.join("attached-overlay-packets.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    result
}
