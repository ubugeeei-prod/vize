//! API and editor answers share one physical owner, never another package.

use crate::lsp_client::{CorsaProjectClient, session::uri_document_identifier};
use corsa::runtime::block_on;
use std::{fs, io::Write, path::Path};
use vize_l0::{FxHashMap, cstr};

#[path = "../../../../../../tests/support/original_diagnosing_process.rs"]
mod control;

fn only_owner(root: &Path, native: &Path) -> control::ProcessLife {
    only_process_owner(root, native, b"--lsp")
}

fn only_process_owner(root: &Path, native: &Path, role: &[u8]) -> control::ProcessLife {
    let owner = if role == b"--lsp" {
        control::native_lsp(root, native)
    } else {
        control::native_process(root, native, role)
    };
    let mut children = Vec::<u32>::new();
    for task in fs::read_dir(cstr!("/proc/{}/task", std::process::id())).unwrap() {
        let path = task.unwrap().path().join("children");
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("cannot observe task children: {error}"),
        };
        children.extend(
            content
                .split_whitespace()
                .map(|pid| pid.parse::<u32>().unwrap()),
        );
    }
    children.sort_unstable();
    children.dedup();
    let lives: Vec<_> = children
        .into_iter()
        .filter_map(|pid| {
            let base = std::path::PathBuf::from(cstr!("/proc/{pid}").as_str());
            if fs::read_link(base.join("exe")).ok()?.as_path() != native
                || fs::read_link(base.join("cwd")).ok()? != root.canonicalize().unwrap()
            {
                return None;
            }
            Some((pid, fs::read(base.join("cmdline")).unwrap()))
        })
        .collect();
    assert_eq!(
        lives.len(),
        1,
        "one physical native child per owner: {lives:?}"
    );
    assert_eq!(lives[0].0, owner.pid);
    assert!(lives[0].1.split(|byte| *byte == 0).any(|arg| arg == role));
    owner
}

fn complete_report(
    client: &mut CorsaProjectClient,
    root: &Path,
    source: &str,
) -> serde_json::Value {
    let uri = crate::file_uri::path_to_file_uri(&root.join("source.ts"));
    let mut documents = FxHashMap::default();
    documents.insert(uri.clone(), source.into());
    serde_json::to_value(client.diagnostics_via_editor_lsp(&uri, &documents).unwrap()).unwrap()
}

fn api_type(client: &CorsaProjectClient, root: &Path, source: &str) -> vize_l0::String {
    let session = client.session.as_ref().unwrap();
    let uri = crate::file_uri::path_to_file_uri(&root.join("source.ts"));
    let session_uri = client
        .session_document_uris
        .get(uri.as_str())
        .unwrap_or(&uri);
    let ty = block_on(session.get_type_at_position(
        uri_document_identifier(session_uri),
        source.find("answer").unwrap() as u32,
    ))
    .unwrap()
    .unwrap();
    block_on(session.type_to_string(ty.id, None, None))
        .unwrap()
        .into()
}

#[test]
fn native_editor_resolves_package_alias_from_monorepo_root_with_one_api_editor_owner() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let root = tempfile::tempdir().unwrap();
    control::write_fixture(root.path(), &fixture);
    let native = control::runtime();
    let mut client =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), root.path()).unwrap();
    assert!(client.project_session_attached_to_editor);
    let first = only_owner(root.path(), &native);
    let source = fixture["source"].as_str().unwrap();
    assert_eq!(
        api_type(&client, root.path(), source).as_str(),
        "() => string"
    );
    assert_eq!(
        complete_report(&mut client, root.path(), source),
        fixture["expected"]
    );
    assert_eq!(
        api_type(&client, root.path(), source).as_str(),
        "() => string"
    );
    assert_eq!(only_owner(root.path(), &native), first);
    let edited = fixture["leafEdit"].as_str().unwrap();
    let uri = crate::file_uri::path_to_file_uri(&root.path().join("source.ts"));
    client.did_change(&uri, edited).unwrap();
    assert_eq!(
        complete_report(&mut client, root.path(), edited),
        fixture["editedExpected"]
    );
    assert_eq!(
        api_type(&client, root.path(), edited).as_str(),
        "() => number"
    );
    let edited_owner = only_owner(root.path(), &native);
    if edited_owner != first {
        assert!(
            client.materialized_project_session,
            "only a materialized project transition replaces this owner"
        );
        control::assert_reaped(&first);
    }
    let api = &client
        .editor_lsp
        .as_ref()
        .unwrap()
        .configured_api
        .as_ref()
        .unwrap()
        .session;
    let receipt = serde_json::json!({
        "schema":"vize.same-owner-api-editor-8371",
        "native":native,"root":root.path(),
        "initialOwner":{"pid":first.pid,"birth":first.birth,"executable":first.executable},
        "currentOwner":{"pid":edited_owner.pid,"birth":edited_owner.birth,"executable":edited_owner.executable},
        "attachment":api,"originalReport":fixture["expected"],"editedReport":fixture["editedExpected"],
        "apiTypes":["() => string","() => number"],"physicalNativeChildren":1
    });
    std::io::stdout()
        .write_all(cstr!("SHARED_EDITOR_OWNER_8371 {receipt}\n").as_bytes())
        .unwrap();
    client.shutdown().unwrap();
    control::assert_reaped(&edited_owner);
    assert_eq!(
        fs::read(root.path().join("source.ts")).unwrap(),
        source.as_bytes()
    );
}

#[test]
fn native_editor_resolves_package_alias_from_monorepo_root_without_cross_owner_attachment() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let native = control::runtime();
    let a = tempfile::tempdir().unwrap();
    let b = tempfile::tempdir().unwrap();
    control::write_fixture(a.path(), &fixture);
    control::write_fixture(b.path(), &fixture);
    let clean = fixture["leafEdit"].as_str().unwrap();
    fs::write(b.path().join("source.ts"), clean).unwrap();
    let mut first =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), a.path()).unwrap();
    let mut second =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), b.path()).unwrap();
    let a_owner = only_owner(a.path(), &native);
    let b_owner = only_owner(b.path(), &native);
    assert_ne!(a_owner, b_owner);
    let a_api = &first
        .editor_lsp
        .as_ref()
        .unwrap()
        .configured_api
        .as_ref()
        .unwrap()
        .session;
    let b_api = &second
        .editor_lsp
        .as_ref()
        .unwrap()
        .configured_api
        .as_ref()
        .unwrap()
        .session;
    // Session counters belong to each native server; the pipe scopes their IDs.
    assert_ne!(
        (&a_api.pipe, &a_api.session_id),
        (&b_api.pipe, &b_api.session_id)
    );
    assert_ne!(a_api.pipe, b_api.pipe);
    assert_eq!(
        Path::new(&first.project_session().unwrap().project().config_file_name),
        a.path().join("tsconfig.json")
    );
    assert_eq!(
        Path::new(&second.project_session().unwrap().project().config_file_name),
        b.path().join("tsconfig.json")
    );
    assert_eq!(
        complete_report(&mut first, a.path(), fixture["source"].as_str().unwrap()),
        fixture["expected"]
    );
    assert_eq!(
        complete_report(&mut second, b.path(), clean),
        fixture["editedExpected"]
    );
    first.shutdown().unwrap();
    control::assert_reaped(&a_owner);
    assert_eq!(
        complete_report(&mut second, b.path(), clean),
        fixture["editedExpected"]
    );
    assert_eq!(only_owner(b.path(), &native), b_owner);
    second.shutdown().unwrap();
    control::assert_reaped(&b_owner);
}

#[test]
fn native_editor_resolves_package_alias_from_monorepo_root_reaps_stopped_shared_owner() {
    if !control::enabled() {
        return;
    }
    let fixture = control::fixture();
    let native = control::runtime();
    let root = tempfile::tempdir().unwrap();
    control::write_fixture(root.path(), &fixture);
    let mut client =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), root.path()).unwrap();
    let owner = only_owner(root.path(), &native);
    assert!(client.project_session_attached_to_editor);
    let stopped = std::process::Command::new("kill")
        .arg("-STOP")
        .arg(cstr!("{}", owner.pid).as_str())
        .status()
        .unwrap();
    assert!(stopped.success());
    let start = std::time::Instant::now();
    client.invalidate_disk_project_state().unwrap();
    assert!(
        start.elapsed() < std::time::Duration::from_secs(5),
        "discard must not request a snapshot from its stopped backend"
    );
    control::assert_reaped(&owner);
    assert!(client.session.is_none());
    assert!(client.editor_lsp.is_none());
    assert!(!client.project_session_attached_to_editor);
    assert!(client.editor_lsp_documents_dirty);
    assert_eq!(
        complete_report(
            &mut client,
            root.path(),
            fixture["source"].as_str().unwrap()
        ),
        fixture["expected"]
    );
    let replacement = only_owner(root.path(), &native);
    assert_ne!(replacement, owner);
    client.shutdown().unwrap();
    control::assert_reaped(&replacement);
}

#[test]
fn native_editor_resolves_package_alias_from_monorepo_root_keeps_memberless_api_fallback() {
    if !control::enabled() {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    let configuration = r#"{"files":[],"compilerOptions":{"noEmit":true}}"#;
    fs::write(&config, configuration).unwrap();
    let native = control::runtime();
    let mut client =
        CorsaProjectClient::new_for_editor_workspace(native.to_str(), root.path()).unwrap();
    assert!(client.shared_editor_transport_requested);
    assert!(!client.shared_editor_transport);
    assert!(!client.project_session_attached_to_editor);
    assert!(client.editor_lsp.is_none());
    let session = client.project_session().unwrap();
    assert_eq!(Path::new(&session.project().config_file_name), config);
    block_on(session.describe_capabilities()).unwrap();
    let owner = only_process_owner(root.path(), &native, b"--api");
    client.shutdown().unwrap();
    control::assert_reaped(&owner);
    assert_eq!(fs::read(config).unwrap(), configuration.as_bytes());
}

mod replay;
