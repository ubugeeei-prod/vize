//! Same diagnosing native process and terminal evidence for the frozen corpus.
use super::{CorsaProjectClient, VirtualProject};
use corsa_lsp::process::ProcessObserver;
use serde_json::{Value, json};
use std::{
    cell::{Cell, RefCell},
    fs,
    path::PathBuf,
};
use vize_l0::cstr;

thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static PENDING: RefCell<Option<(ProcessObserver, Option<PathBuf>)>> = const { RefCell::new(None) };
}

/// Scope observation to this test thread without changing process environment.
pub(super) fn begin_for_test() {
    ACTIVE.with(|active| assert!(!active.replace(true), "previous native receipt unfinished"));
}

pub(super) fn enabled() -> bool {
    ACTIVE.with(Cell::get) || std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR").is_some()
}

pub(super) fn capture(
    client: &mut CorsaProjectClient,
    project: &VirtualProject,
    uris: &[vize_l0::String],
    raw: &impl std::fmt::Debug,
) {
    if !enabled() {
        return;
    }
    let dir = std::env::var_os("VIZE_SEMANTIC_COMPLETION_CAPTURE_DIR")
        .map(PathBuf::from)
        .map(|dir| dir.join(project.project_root().file_name().unwrap()));
    let observer = client
        .batch_test_process_observer()
        .expect("actual diagnosing native owner");
    let pid = observer.receipts()[0].pid;
    PENDING.with(|pending| {
        assert!(
            pending.borrow().is_none(),
            "one diagnosing native process per check"
        );
        *pending.borrow_mut() = Some((observer, dir.clone()));
    });
    let requested: Vec<_> = uris.iter().map(|uri| uri.as_str()).collect();
    let authorities = client.batch_test_receipts(&requested);
    assert_eq!(authorities.len(), uris.len());
    let config = project.generated_tsconfig_path();
    let authored_uri = crate::file_uri::path_to_file_uri(&project.project_root().join("shared.ts"));
    let primary_uri =
        crate::file_uri::path_to_file_uri(&project.virtual_root().join("Comp0.vue.ts"));
    let primary = authorities
        .iter()
        .find(|authority| authority["diagnosedUri"] == primary_uri.as_str())
        .expect("frozen corpus diagnoses generated Comp0.vue");
    let leaf = client.batch_test_source(primary_uri.as_str(), primary);
    let authored = authorities
        .iter()
        .find(|authority| authority["diagnosedUri"] == authored_uri.as_str())
        .map(|authority| client.batch_test_source(authored_uri.as_str(), authority));
    let receipt = json!({"nativeByUri":authorities,"requestedUris":uris,
        "leafProvider":leaf,"authoredProvider":authored,
        "generatedConfig":config,"generatedConfigBytes":fs::read(&config).unwrap(),
        "authoredConfigBytes":fs::read(project.project_root().join("tsconfig.json")).unwrap(),
        "rawDiagnostics":cstr!("{raw:#?}"),
        "authoredDiagnosticPaths":project.diagnostic_paths_sorted(),
        "generatedFiles":project.virtual_files_sorted().into_iter().map(|file|
            json!({"original":file.original_path,"virtual":file.virtual_path,"code":file.content.as_str(),
                "sourceMap":cstr!("{:#?}",file.source_map)})).collect::<Vec<_>>()});
    if let Some(dir) = &dir {
        fs::create_dir_all(dir).unwrap();
        fs::write(
            dir.join("same-owner.json"),
            serde_json::to_vec_pretty(&receipt).unwrap(),
        )
        .unwrap();
    }
    // Persist every native answer and refusal before judging complete outputs.
    for (uri, authority) in uris.iter().zip(&authorities) {
        assert_eq!(authority["mode"], "editor-lsp");
        assert_eq!(authority["diagnosedUri"], uri.as_str());
        assert_eq!(authority["processPid"], json!(pid));
        assert!(authority["nativeInitializeResponse"].is_object());
        assert!(authority["nativeShutdownMode"].is_string());
        assert_eq!(authority["sourceFileNamesResponse"]["status"], "ok");
        assert!(authority["sourceFileNamesResponse"]["wholeResponse"].is_array());
        if uri != &authored_uri {
            assert_eq!(authority["selectedConfig"], json!(config));
            assert_eq!(
                authority["project"]["compilerOptions"],
                authority["normalizedRequestedOptions"]
            );
            assert_eq!(authority["project"]["compilerOptions"]["strict"], true);
            let mut roots = authority["project"]["rootFiles"]
                .as_array()
                .unwrap()
                .clone();
            let mut requested = authority["normalizedRequestedFiles"]
                .as_array()
                .unwrap()
                .clone();
            roots.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
            requested.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
            assert_eq!(roots, requested, "full configured root membership");
        }
    }
    assert_source(
        &leaf,
        "loadedLeafSources",
        include_bytes!(
            "../../../../../../tests/_fixtures/differential/typechecker/native-semantic-completion/inputs/node_modules/leaf-types/index.d.ts.txt"
        ),
    );
    if let Some(authored) = authored {
        assert_source(
            &authored,
            "loadedAuthoredSources",
            include_bytes!(
                "../../../../../../tests/_fixtures/differential/typechecker/native-semantic-completion/inputs/shared.ts.txt"
            ),
        );
    }
}

fn assert_source(provider: &Value, key: &str, bytes: &[u8]) {
    assert_eq!(provider["qualified"], true, "{provider}");
    let [source] = provider[key].as_array().unwrap().as_slice() else {
        panic!("one complete native source required: {provider}");
    };
    assert_eq!(source["completeNativeText"], json!(bytes));
    assert_eq!(source["completePhysicalBytes"], json!(bytes));
    assert!(
        source["wholeEncodedPayload"]
            .as_array()
            .is_some_and(|bytes| !bytes.is_empty())
    );
}

pub(in crate::batch::executor) fn finish() -> Value {
    ACTIVE.with(|active| active.set(false));
    PENDING.with(|pending| {
        let Some((observer,dir)) = pending.borrow_mut().take() else { return Value::Null; };
        let rows: Vec<_> = observer.receipts().iter().map(|r| json!({
            "pid":r.pid,"completed":r.completed,"stderr":r.stderr,"stderrComplete":r.stderr_complete,
            "stderrError":r.stderr_error,"processError":r.process_error,"status":cstr!("{:?}",r.exit_status),
            "exitCode":r.exit_status.and_then(|s|s.code()),"timedOut":r.timed_out,
            "killRequested":r.kill_requested,"reaped":r.reaped,"elapsedNs":r.elapsed.as_nanos()
        })).collect();
        let rows = json!(rows);
        if let Some(dir) = dir {
            fs::write(dir.join("terminal.json"),serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
        }
        rows
    })
}
