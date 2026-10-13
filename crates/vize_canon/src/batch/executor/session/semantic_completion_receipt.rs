//! Separately scoped test evidence for required same-config native completion.
use super::{CorsaProjectClient, VirtualProject};
use corsa_lsp::process::ProcessObserver;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
};
use vize_l0::cstr;

thread_local! { static PENDING: RefCell<Option<(ProcessObserver, PathBuf)>> = const { RefCell::new(None) }; }

pub(super) fn enabled() -> bool {
    std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR").is_some()
}

pub(super) fn capture(
    client: &mut CorsaProjectClient,
    project: &VirtualProject,
    uris: &[vize_l0::String],
    raw: &impl std::fmt::Debug,
) {
    let Some(dir) = std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR").map(PathBuf::from)
    else {
        return;
    };
    let name = project.project_root().file_name().unwrap();
    let dir = dir.join(name);
    fs::create_dir_all(&dir).unwrap();
    let observer = client.batch_test_process_observer();
    if let Some(observer) = &observer {
        PENDING.with(|pending| *pending.borrow_mut() = Some((observer.clone(), dir.clone())));
    }
    let config = project.generated_tsconfig_path();
    let diagnostic_paths = project.diagnostic_paths_sorted();
    let mut receipt = json!({"native":null,"nativeByUri":[],"requestedUris":uris,
        "generatedConfig":config,"rawDiagnostics":cstr!("{raw:#?}"),
        "authoredDiagnosticPaths":diagnostic_paths,"captureStage":"pending native metadata",
        "workloadScope":if diagnostic_paths.is_empty() {"five generated program URIs only"}
            else {"public workload: five generated URIs plus authored shared.ts"},
        "spawnReceiptsBeforeClose":cstr!("{:?}",observer.as_ref().map(ProcessObserver::receipts))});
    save(&dir, &receipt);
    receipt["generatedConfigBytes"] = json!(fs::read(&config).unwrap());
    receipt["authoredConfigBytes"] =
        json!(fs::read(project.project_root().join("tsconfig.json")).unwrap());
    let case: Value = serde_json::from_str(include_str!(
        "../../../../../../tests/_fixtures/differential/typechecker/native-semantic-completion/case.json"
    )).unwrap();
    receipt["wholeAuthoredInputFiles"] = json!(
        case["inputFiles"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                let path = project.project_root().join(row["file"].as_str().unwrap());
                json!({"file":row["file"],"physical":path,"bytes":fs::read(&path).unwrap()})
            })
            .collect::<Vec<_>>()
    );
    receipt["generatedFiles"] = json!(project.virtual_files_sorted().into_iter().map(|file|
        json!({"original":file.original_path,"virtual":file.virtual_path,"code":file.content.as_str(),
            "sourceMap":cstr!("{:#?}",file.source_map)})).collect::<Vec<_>>());
    receipt["wholePhysicalFiles"] = json!(physical(project.virtual_root()));
    save(&dir, &receipt);
    let observer = observer.expect("real diagnosing LSP process required");
    let requested: Vec<_> = uris.iter().map(|uri| uri.as_str()).collect();
    let authorities = client.batch_test_receipts(&requested);
    receipt["nativeByUri"] = json!(authorities);
    receipt["captureStage"] = json!("complete per-URI native metadata");
    save(&dir, &receipt);
    let primary_uri =
        crate::file_uri::path_to_file_uri(&project.virtual_root().join("Comp0.vue.ts"));
    let native = authorities
        .iter()
        .find(|row| row["diagnosedUri"] == primary_uri.as_str())
        .expect("the scoped frozen corpus owns Comp0.vue");
    receipt["native"] = native.clone();
    save(&dir, &receipt);
    receipt["leafProvider"] = client.batch_test_leaf_source(primary_uri.as_str(), native);
    save(&dir, &receipt);
    let original_uri = crate::file_uri::path_to_file_uri(&project.project_root().join("shared.ts"));
    if !diagnostic_paths.is_empty() {
        let authored = authorities
            .iter()
            .find(|row| row["diagnosedUri"] == original_uri.as_str())
            .expect("the public workload diagnoses original authored shared.ts");
        receipt["authoredNative"] = authored.clone();
        save(&dir, &receipt);
        receipt["authoredSource"] =
            client.batch_test_authored_source(original_uri.as_str(), authored);
        save(&dir, &receipt);
    }
    receipt["captureStage"] = json!("complete source custody");
    save(&dir, &receipt);
    let mut expected: Vec<_> = [
        "Comp0.vue.ts",
        "Comp1.vue.ts",
        "Comp2.vue.ts",
        "Comp3.vue.ts",
        "shared.ts",
    ]
    .iter()
    .map(|name| crate::file_uri::path_to_file_uri(&project.virtual_root().join(name)))
    .collect();
    if !diagnostic_paths.is_empty() {
        expected.push(original_uri.clone());
    }
    expected.sort();
    assert_eq!(uris, expected);
    assert_eq!(authorities.len(), expected.len());
    for authority in &authorities {
        assert_native(authority, observer.receipts()[0].pid);
        if authority["diagnosedUri"] != original_uri.as_str() {
            assert_eq!(authority["selectedConfig"], json!(config));
            assert_eq!(authority["project"], native["project"]);
            assert_eq!(
                authority["sourceNamesResponse"],
                native["sourceNamesResponse"]
            );
        }
    }
    let sources = native["sourceNamesResponse"].as_array().unwrap();
    let mut generated: Vec<_> = sources
        .iter()
        .filter_map(Value::as_str)
        .filter(|name| Path::new(name).parent() == Some(project.virtual_root()))
        .map(|name| Path::new(name).file_name().unwrap().to_str().unwrap())
        .filter(|name| !name.ends_with(".d.ts"))
        .collect();
    generated.sort();
    assert_eq!(
        generated,
        vec![
            "Comp0.vue.ts",
            "Comp1.vue.ts",
            "Comp2.vue.ts",
            "Comp3.vue.ts",
            "shared.ts"
        ]
    );
    assert_source(
        &receipt["leafProvider"],
        "loadedLeafSources",
        include_bytes!(
            "../../../../../../tests/_fixtures/differential/typechecker/native-semantic-completion/inputs/node_modules/leaf-types/index.d.ts.txt"
        ),
    );
    if !diagnostic_paths.is_empty() {
        let authored = &receipt["authoredNative"];
        assert_eq!(
            authored["selectedConfig"],
            json!(project.project_root().join("tsconfig.json"))
        );
        assert_ne!(authored["project"], native["project"]);
        assert_eq!(authored["attachment"], native["attachment"]);
        assert_eq!(
            receipt["leafProvider"]["originalSingletonAdmission"]["status"],
            "refused"
        );
        assert_eq!(
            receipt["leafProvider"]["originalSingletonAdmission"]["wholeRefusal"],
            "ProjectIdentity"
        );
        assert_eq!(
            receipt["authoredSource"]["originalSingletonAdmission"]["status"],
            "refused"
        );
        assert_eq!(
            receipt["authoredSource"]["originalSingletonAdmission"]["wholeRefusal"],
            "ProjectIdentity"
        );
        assert_source(
            &receipt["authoredSource"],
            "loadedAuthoredSources",
            include_bytes!(
                "../../../../../../tests/_fixtures/differential/typechecker/native-semantic-completion/inputs/shared.ts.txt"
            ),
        );
    }
}

fn save(dir: &Path, receipt: &Value) {
    fs::write(
        dir.join("same-owner.json"),
        serde_json::to_vec_pretty(receipt).unwrap(),
    )
    .unwrap();
}

fn assert_native(authority: &Value, pid: u32) {
    assert_eq!(authority["mode"], "editor-lsp");
    assert_eq!(
        authority["nativeShutdownMode"],
        "authenticated-native-7.0.2-shutdown-eof"
    );
    assert_eq!(
        authority["nativeInitializeResponse"]["serverInfo"],
        json!({"name":"typescript-go","version":"7.0.2"})
    );
    assert_eq!(
        authority["project"]["compilerOptions"],
        authority["normalizedRequestedOptions"]
    );
    assert_eq!(authority["project"]["compilerOptions"]["strict"], true);
    assert_eq!(authority["processPid"], json!(pid));
    assert_eq!(
        authority["materializedConfigurationObservation"]["status"],
        "ok"
    );
    assert_eq!(
        authority["materializedConfigurationObservation"]["selectedConfig"],
        authority["project"]["configFileName"]
    );
    let mut roots: Vec<_> = authority["project"]["rootFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let mut requested: Vec<_> = authority["normalizedRequestedFiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    roots.sort();
    requested.sort();
    assert_eq!(
        roots, requested,
        "the full configured root membership must agree"
    );
}

fn assert_source(provider: &Value, key: &str, bytes: &[u8]) {
    assert_eq!(provider["qualified"], true);
    for refusal in [
        "absentProjectRefused",
        "foreignDescriptorRefused",
        "duplicateProjectRefused",
        "changedWholeDescriptorRefused",
    ] {
        assert_eq!(provider["descriptorRefusals"][refusal], true);
    }
    let [source] = provider[key].as_array().unwrap().as_slice() else {
        panic!("one whole physical source must match the native source text")
    };
    assert_eq!(source["completeNativeText"], json!(bytes));
    assert_eq!(source["completePhysicalBytes"], json!(bytes));
}

pub(in crate::batch::executor) fn finish() -> Value {
    PENDING.with(|pending| {
        let Some((observer,dir))=pending.borrow_mut().take() else {return Value::Null};
        let receipts=observer.receipts();
        let rows:Vec<_>=receipts.iter().map(|r|json!({"pid":r.pid,"completed":r.completed,"stderr":r.stderr,"stderrComplete":r.stderr_complete,"stderrError":r.stderr_error,"processError":r.process_error,"status":cstr!("{:?}",r.exit_status),"exitCode":r.exit_status.and_then(|s|s.code()),"timedOut":r.timed_out,"killRequested":r.kill_requested,"reaped":r.reaped,"elapsedNs":r.elapsed.as_nanos()})).collect();
        let rows=json!(rows);
        fs::write(dir.join("terminal.json"),serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
        rows
    })
}

fn physical(root: &Path) -> Vec<Value> {
    fn visit(root: &Path, path: &Path, rows: &mut Vec<Value>) {
        let mut entries: Vec<_> = fs::read_dir(path).unwrap().map(|e| e.unwrap()).collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let p = entry.path();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                visit(root, &p, rows)
            } else if kind.is_file() {
                rows.push(
                    json!({"file":p.strip_prefix(root).unwrap(),"bytes":fs::read(&p).unwrap()}),
                )
            } else {
                rows.push(json!({"file":p.strip_prefix(root).unwrap(),"symlinkTarget":fs::read_link(&p).unwrap()}))
            }
        }
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows);
    rows
}
