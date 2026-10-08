//! Actual API launches inspect the prior physical owner before native exec.

use super::CorsaProjectClient;
use std::{fs, io::Write, os::unix::fs::PermissionsExt, path::Path};
use vize_l0::{FxHashMap, cstr};

#[path = "../../../tests/support/original_diagnosing_process.rs"]
mod control;

const FIXTURE: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/native-project-retirement-3952/input.json"
);
const OBSERVER: &str = include_str!(
    "../../../../../tests/_fixtures/differential/lsp/native-project-retirement-3952/observe-native-owner.py"
);

fn quote(path: &Path) -> vize_l0::String {
    cstr!("'{}'", path.to_str().unwrap().replace('\'', "'\"'\"'"))
}

#[test]
fn native_project_replacement_reaps_previous_api_before_launch_and_keeps_whole_answer() {
    if !control::enabled() {
        return;
    }
    let fixture: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    let root = tempfile::tempdir().unwrap();
    control::write_fixture(root.path(), &fixture);
    assert_eq!(fixture["source"], control::fixture()["source"]);
    let native = control::runtime();
    let trace = tempfile::tempdir().unwrap();
    let observer = trace.path().join("observe.py");
    fs::write(&observer, OBSERVER).unwrap();
    let log = trace.path().join("native-owners.jsonl");
    // The existing .bin classification selects the genuine async API recipe.
    let bin = trace.path().join(".bin");
    fs::create_dir(&bin).unwrap();
    let launcher = bin.join("tsgo");
    fs::write(
        &launcher,
        cstr!(
            "#!/bin/sh\nexec python3 {} {} {} \"$@\"\n",
            quote(&observer),
            quote(&native),
            quote(&log),
        )
        .as_str(),
    )
    .unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
    let mut client = CorsaProjectClient::new_for_workspace(launcher.to_str(), root.path()).unwrap();
    assert!(
        client.has_project_session(),
        "actual native API is required"
    );
    let mut api_owners = vec![control::native_process(root.path(), &native, b"--api")];
    let uri = crate::file_uri::path_to_file_uri(&root.path().join("source.ts"));
    let source = fixture["source"].as_str().unwrap();
    let mut documents = FxHashMap::default();
    documents.insert(uri.clone(), source.into());
    let mut editor_owner = None;
    for _ in 0..3 {
        let report = client
            .diagnostics_via_editor_lsp(uri.as_str(), &documents)
            .unwrap();
        assert_eq!(serde_json::to_value(report).unwrap(), fixture["expected"]);
        let observed = control::native_lsp(root.path(), &native);
        if let Some(previous) = &editor_owner {
            assert_eq!(
                &observed, previous,
                "topology reload reuses the native editor"
            );
        }
        editor_owner = Some(observed);
        client
            .activate_workspace_project_with_reload(root.path(), None, true)
            .unwrap();
        control::assert_reaped(api_owners.last().unwrap());
        api_owners.push(control::native_process(root.path(), &native, b"--api"));
        assert!(
            client.editor_lsp.is_some(),
            "topology reload retains the editor owner"
        );
    }
    let final_report = client
        .diagnostics_via_editor_lsp(uri.as_str(), &documents)
        .unwrap();
    assert_eq!(
        serde_json::to_value(final_report).unwrap(),
        fixture["expected"]
    );
    assert_eq!(
        control::native_lsp(root.path(), &native),
        *editor_owner.as_ref().unwrap(),
        "the third topology reload retains the complete editor answer and owner",
    );
    client.document_texts = documents;
    client.activate_materialized_project_session().unwrap();
    assert!(client.has_project_session());
    assert!(
        client.editor_lsp.is_none(),
        "mode transition retires the old editor"
    );
    control::assert_reaped(editor_owner.as_ref().unwrap());
    control::assert_reaped(api_owners.last().unwrap());
    api_owners.push(control::native_process(root.path(), &native, b"--api"));
    client.shutdown().unwrap();
    control::assert_reaped(api_owners.last().unwrap());

    let launches: Vec<serde_json::Value> = fs::read_to_string(&log)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let receipt = serde_json::json!({
        "schema": "vize.native-project-retirement-3952",
        "native": native,
        "editor": editor_owner.map(|owner| serde_json::json!({
            "pid": owner.pid, "birth": owner.birth, "executable": owner.executable,
        })),
        "physicalApiOwners": api_owners.iter().map(|owner| serde_json::json!({
            "pid": owner.pid, "birth": owner.birth, "executable": owner.executable,
        })).collect::<Vec<_>>(),
        "launches": launches,
        "expectedDiagnosticReport": fixture["expected"],
    });
    std::io::stdout()
        .write_all(cstr!("NATIVE_PROJECT_RETIREMENT_3952 {}\n", receipt).as_bytes())
        .unwrap();
    let api: Vec<_> = launches
        .iter()
        .filter(|entry| entry["role"] == "--api")
        .collect();
    assert_eq!(
        api.len(),
        5,
        "initial API, three reloads and mode transition"
    );
    for (launch, physical) in api.iter().zip(&api_owners) {
        assert_eq!(launch["pid"].as_u64(), Some(u64::from(physical.pid)));
        assert_eq!(
            launch["birth"].as_str().unwrap().parse::<u64>().unwrap(),
            physical.birth
        );
        assert_eq!(physical.executable, native);
        let previous = launch["previousNativeOwners"].as_array().unwrap();
        assert!(
            previous.iter().all(|owner| owner["role"] != "--api"),
            "previous physical API process was still present at replacement: {launch}"
        );
    }
    assert!(
        api.iter().skip(1).all(|launch| {
            launch["previousNativeOwners"]
                .as_array()
                .unwrap()
                .iter()
                .any(|owner| owner["role"] == "--lsp")
        }),
        "all replacements must exercise the live editor plus API ownership boundary"
    );
}
