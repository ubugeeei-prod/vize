//! Actual materialized-client role; distinct from retained-attached Maestro proof.
use crate::{
    file_uri::{file_uri_to_path, path_to_file_uri},
    lsp_client::CorsaProjectClient,
};
use lsp_types::{DocumentDiagnosticReport, DocumentDiagnosticReportResult};
use serde_json::json;
use std::path::Path;
// An unchanged main file can import an unsaved dependency.
pub(super) fn materialized_overlay_packets(
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
            super::completion_multiset(&completion),
            super::completion_multiset(&direct_completion)
        );
        let completion_order = if completion == direct_completion {
            "identical"
        } else {
            "known-order-difference-10747"
        };
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
        rows.push(json!({"phase":phase,"mainText":MAIN,"supportOverlay":support,"apiEligible":api_eligible,"projectSessionAttachedToEditor":client.project_session_attached_to_editor,"materializedProject":client.materialized_project_session,
            "nativePull":direct,"selected":selected_packet,"hover":hover,"references":references.ok(),
            "directReferences":direct_references,"definition":definition.ok(),"directDefinition":direct_definition,
            "completion":completion,"directCompletion":direct_completion,"completionOrder":completion_order,"rename":rename,"directRename":direct_rename,"batch":batch}));
        std::fs::write(
            capture.join("materialized-overlay-progress.json"),
            serde_json::to_vec_pretty(&rows).unwrap(),
        )
        .unwrap();
        assert_eq!(std::fs::read(&main_path).unwrap(), MAIN.as_bytes());
        assert_eq!(std::fs::read(&support_path).unwrap(), SUPPORT.as_bytes());
    }
    // No original disk file: the batch must exercise a mapped identity.
    let virtual_path = root.join("Virtual.vue.ts");
    let virtual_uri = path_to_file_uri(&virtual_path);
    const VIRTUAL: &str = "export const bad: number = \"x\";\r\n";
    client.did_open(&virtual_uri, VIRTUAL).unwrap();
    let mapped_virtual = client.session_document_uri(&virtual_uri);
    assert_ne!(mapped_virtual, virtual_uri);
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
    let mapped_main = client.session_document_uri(&main_uri);
    let mapped_path = file_uri_to_path(&mapped_main).unwrap();
    let directory = mapped_path.parent().unwrap();
    let evidence = json!({"scope":"Original close-to-root-expansion frontier; API project descriptor is independently owned metadata, not an LS overlay lease",
        "authoredMain":identity(&main_path),"authoredSupport":identity(&support_path),"mappedMain":identity(&mapped_path),
        "relativeImportCandidates":["support.ts","support.tsx","support.d.ts","support/package.json","support/index.ts"].map(|name|identity(&directory.join(name))),
        "project":client.session.as_ref().map(|session|session.project()),"materializedProject":client.materialized_project_session,
        "attached":client.project_session_attached_to_editor,"virtualPull":virtual_pull,"selectedWholeBatch":whole_batch});
    std::fs::write(
        capture.join("close-topology-frontier-identity.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
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
    client.did_close(&virtual_uri).unwrap();
    client.did_close(&main_uri).unwrap();
    assert!(client.hover_raw(&main_uri, 2, 31).unwrap().is_none());
    client.shutdown().unwrap();
    let result = json!({"scope":"materialized client; actual attachment transitions observed; authored originals retained",
        "root":root,"mainUri":main_uri,"supportUri":support_uri,"config":std::str::from_utf8(config).unwrap(),"rows":rows,
        "materializedBatch":{"virtualUri":virtual_uri,"mappedUri":mapped_virtual,"source":VIRTUAL,"nativePull":virtual_pull,"selected":whole_batch}});
    std::fs::write(
        capture.join("materialized-overlay-packets.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    result
}

fn identity(path: &Path) -> serde_json::Value {
    use std::os::unix::fs::MetadataExt;
    match std::fs::read(path) {
        Ok(bytes) => {
            let meta = std::fs::metadata(path).unwrap();
            json!({"path":path,"canonical":path.canonicalize().unwrap(),"device":meta.dev(),"inode":meta.ino(),"bytes":bytes.len(),"sha256":super::hash(&bytes),"text":std::str::from_utf8(&bytes).unwrap()})
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            json!({"path":path,"state":"absent"})
        }
        Err(error) => panic!("cannot establish actual source identity: {error}"),
    }
}
