//! Actual separate API connections on one editor; retained-reader contract only.
use super::*;
use corsa::api::{FileChangeSummary, FileChanges};
use std::collections::BTreeSet;

const SOURCE_A: &str =
    "import { label } from '../b/source';\nexport const answer = label;\nanswer.toUpperCase();\n";
const EDITED_A: &str = "import { label } from '../b/source';\nexport const answer = label;\nanswer.toUpperCase();\nexport const onlyA: number = 1;\n";
const EDITED_B: &str = "export const label: boolean = Math.random() > 0.5;\nlabel.valueOf();\n";

struct NativeResolve;
impl lsp_types::request::Request for NativeResolve {
    type Params = Value;
    type Result = Option<Value>;
    const METHOD: &'static str = "completionItem/resolve";
}

fn sockets(name: &str) -> BTreeSet<std::string::String> {
    let mut raw = Vec::new();
    let mut inodes = BTreeSet::new();
    for entry in fs::read_dir("/proc/self/fd").unwrap() {
        let entry = entry.unwrap();
        let target = match fs::read_link(entry.path()) {
            Ok(target) => target,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("cannot observe owned FD: {error}"),
        };
        let target = target.to_string_lossy().into_owned();
        if target.starts_with("socket:[") {
            assert!(target.ends_with(']'));
            inodes.insert(target.clone());
            raw.push(json!({"fd":entry.file_name().to_string_lossy(),"target":target}));
        }
    }
    record(
        name,
        &json!({"pid":std::process::id(),"socketFds":raw,"socketInodes":inodes}),
    );
    inodes
}

fn project(api: &ApiClient, name: &str, config: &Path, uri: &str) -> ProjectSession {
    let result = block_on(ProjectSession::open(
        api.clone(),
        config.to_str().unwrap(),
        Some(uri_document_identifier(uri)),
    ));
    record(name, &json!(result.as_ref().map(|session|json!({"project":session.project(),"snapshot":session.snapshot().handle,"dialect":format!("{:?}",session.client().dialect())})).map_err(|error|error.to_string())));
    let session = result.unwrap();
    assert_eq!(
        Path::new(session.project().config_file_name.as_str()),
        config
    );
    let parsed = block_on(api.parse_config_file(uri_document_identifier(
        &crate::file_uri::path_to_file_uri(config),
    )));
    record(
        cstr!("{name}-parsed-config").as_str(),
        &json!(parsed.as_ref().map_err(|error| error.to_string())),
    );
    assert_eq!(session.project().compiler_options, parsed.unwrap().options);
    assert!(session.project().root_files.iter().any(|file|Path::new(file.as_str())==crate::file_uri::file_uri_to_path(uri).unwrap()));
    session
}

fn view_type(
    api: &ApiClient,
    snapshot: &ManagedSnapshot,
    project: &ProjectHandle,
    uri: &str,
    position: u32,
    name: &str,
) -> String {
    let ty = block_on(api.get_type_at_position(
        snapshot.handle.clone(),
        project.clone(),
        uri_document_identifier(uri),
        position,
    ));
    record(
        cstr!("{name}-handle").as_str(),
        &json!(ty.as_ref().map_err(|error| error.to_string())),
    );
    let ty = ty.unwrap().unwrap();
    let value =
        block_on(api.type_to_string(snapshot.handle.clone(), project.clone(), ty.id, None, None));
    record(
        name,
        &json!({"snapshot":snapshot.handle,"project":project,"uri":uri,"position":position,"result":value.as_ref().map_err(|error|error.to_string()),"scope":"explicit readonly disk-backed API snapshot; not unsaved editor authority"}),
    );
    value.unwrap().into()
}

#[test]
#[ignore = "private actual API attachment control; fixed SDK Actions only"]
fn native_one_editor_two_api_attachments_retain_readers_until_owner_reap() {
    let version = std::env::var("VIZE_NATIVE_DTO_SDK_VERSION").unwrap();
    let (image, dialect) = match version.as_str() {
        "7.0.2" => (
            "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
            ApiDialect::SessionSnapshots,
        ),
        "7.1.0-beta" => (
            "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
            ApiDialect::DerivedSnapshots,
        ),
        _ => panic!("exact prospective provider required"),
    };
    let native = runtime(&version, image);
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let a_config = config(&root, "a", true, SOURCE_A);
    let b_config = config(&root, "b", false, B_GOOD);
    let a_path = root.join("a/source.ts");
    let b_path = root.join("b/source.ts");
    let a_uri = crate::file_uri::path_to_file_uri(&a_path);
    let b_uri = crate::file_uri::path_to_file_uri(&b_path);
    let a_config_before = fs::read(&a_config).unwrap();
    let b_config_before = fs::read(&b_config).unwrap();
    let mut editor = EditorLspSession::spawn(native.to_str().unwrap(), &root, &root).unwrap();
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, SOURCE_A)), Some((&b_uri, B_GOOD))).unwrap(),
    );
    let owner = only_owner(&root, &native);
    record(
        "physical-owner",
        &json!({"pid":owner.pid,"birth":owner.birth,"executable":owner.executable,"cwd":root,"nativeChildren":1}),
    );
    let before = sockets("sockets-before-api");
    let a_api = editor
        .attach_api()
        .unwrap_or_else(|_| panic!("actual API A attachment required"));
    record("attachment-a", &json!(a_api.session));
    let after_a = sockets("sockets-after-api-a");
    let a_socket = after_a
        .difference(&before)
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(a_socket.len(), 1);
    let b_api = editor
        .attach_api()
        .unwrap_or_else(|_| panic!("actual API B attachment required"));
    record("attachment-b", &json!(b_api.session));
    let after_b = sockets("sockets-after-api-b");
    let b_socket = after_b
        .difference(&after_a)
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(b_socket.len(), 1);
    assert!(a_socket.is_disjoint(&b_socket));
    assert!(a_socket.is_subset(&after_b));
    assert_ne!(
        (&a_api.session.pipe, &a_api.session.session_id),
        (&b_api.session.pipe, &b_api.session.session_id)
    );
    assert_eq!(only_owner(&root, &native), owner);
    let a_session = project(&a_api.client, "api-a-initial", &a_config, &a_uri);
    let mut b_session = project(&b_api.client, "api-b-initial", &b_config, &b_uri);
    assert_eq!(a_api.client.dialect(), Some(dialect));
    assert_eq!(b_api.client.dialect(), Some(dialect));
    let a_snapshot = a_session.snapshot_lease();
    let a_project = a_session.project_handle();
    let a_position = SOURCE_A.find("answer").unwrap() as u32;
    let b_position = B_GOOD.find("label").unwrap() as u32;
    assert_eq!(
        view_type(
            &a_api.client,
            &a_snapshot,
            &a_project,
            &a_uri,
            a_position,
            "api-a-held-initial"
        ),
        "string"
    );
    assert_eq!(
        view_type(
            &b_api.client,
            b_session.snapshot(),
            &b_session.project_handle(),
            &b_uri,
            b_position,
            "api-b-initial-type"
        ),
        "string"
    );
    let b_before = packet(&mut editor, "editor-b-before", &b_uri, &b_config, 6);
    assert_diagnostics(&b_before, clean());
    let issued = completion_item(&b_before, "toUpperCase");
    record("original-issued-b-item", &issued);
    let resolved_before = editor.completion_resolve_for_shared_context_test(&b_uri, issued.clone());
    record("vize-resolve-before-a-edit", &json!(resolved_before));
    let resolved_before = resolved_before.unwrap().unwrap();
    assert_eq!(resolved_before["label"], "toUpperCase");
    assert_eq!(
        resolved_before["detail"],
        "(method) String.toUpperCase(): string"
    );
    assert_eq!(resolved_before["data"], issued["data"]);
    let old_generation = editor.document_generation;
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, EDITED_A)), Some((&b_uri, B_GOOD))).unwrap(),
    );
    assert_ne!(editor.document_generation, old_generation);
    let refused = editor.completion_resolve_for_shared_context_test(&b_uri, issued.clone());
    record("vize-global-generation-refusal", &json!(refused));
    assert_eq!(refused.unwrap(), None);
    // Separate native primitive; the real Vize gate above remains unchanged.
    record("native-resolve-exact-original-request", &issued);
    let native_resolved = block_on(editor.client.request::<NativeResolve>(issued.clone()));
    record(
        "native-resolve-after-a-edit",
        &json!(native_resolved.as_ref().map_err(|error| error.to_string())),
    );
    // Prospective strict whole-packet invariance, including all opaque fields.
    assert_eq!(native_resolved.unwrap(), Some(resolved_before));
    let b_old = b_session.snapshot_lease();
    fs::write(&b_path, EDITED_B).unwrap();
    record(
        "owned-b-disk-update",
        &json!({"path":b_path,"before":B_GOOD,"after":EDITED_B,"authoredUserFiles":false}),
    );
    acknowledge(
        &mut editor,
        &union(Some((&a_uri, EDITED_A)), Some((&b_uri, EDITED_B))).unwrap(),
    );
    let changes = FileChanges::Summary(FileChangeSummary {
        changed: vec![uri_document_identifier(&b_uri)],
        ..Default::default()
    });
    record("api-b-real-file-changes", &json!(changes));
    let refreshed = block_on(b_session.refresh(Some(changes)));
    record(
        "api-b-refresh",
        &json!(refreshed.as_ref().map_err(|error| error.to_string())),
    );
    refreshed.unwrap();
    assert_eq!(
        view_type(
            &b_api.client,
            b_session.snapshot(),
            &b_session.project_handle(),
            &b_uri,
            b_position,
            "api-b-current-boolean"
        ),
        "boolean"
    );
    assert_eq!(
        view_type(
            &a_api.client,
            &a_snapshot,
            &a_project,
            &a_uri,
            a_position,
            "api-a-held-after-b-update"
        ),
        "string"
    );
    let b_current = packet(
        &mut editor,
        "editor-b-current-boolean",
        &b_uri,
        &b_config,
        6,
    );
    assert_diagnostics(&b_current, clean());
    assert_eq!(
        b_current["hover"],
        json!({"contents":{"kind":"markdown","value":"```typescript\nconst label: boolean\n```\n"},"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":5}}})
    );
    let released_old_b = block_on(b_old.release());
    record(
        "api-b-old-snapshot-release",
        &json!(released_old_b.as_ref().map_err(|error| error.to_string())),
    );
    released_old_b.unwrap();
    drop(b_old);
    drop(b_session);
    let b_drain = block_on(b_api.client.drain_snapshot_releases());
    record(
        "api-b-old-releases-drained",
        &json!(b_drain.as_ref().map_err(|error| error.to_string())),
    );
    b_drain.unwrap();
    assert_eq!(
        view_type(
            &a_api.client,
            &a_snapshot,
            &a_project,
            &a_uri,
            a_position,
            "api-a-held-after-b-drop-release"
        ),
        "string"
    );
    let b_session = project(&b_api.client, "api-b-reopened", &b_config, &b_uri);
    assert_eq!(
        view_type(
            &b_api.client,
            b_session.snapshot(),
            &b_session.project_handle(),
            &b_uri,
            b_position,
            "api-b-reopened-boolean"
        ),
        "boolean"
    );
    drop(a_session);
    let released_a = block_on(a_snapshot.release());
    record(
        "api-a-snapshot-share-release",
        &json!(released_a.as_ref().map_err(|error| error.to_string())),
    );
    released_a.unwrap();
    drop(a_snapshot);
    let a_drain = block_on(a_api.client.drain_snapshot_releases());
    record(
        "api-a-releases-drained",
        &json!(a_drain.as_ref().map_err(|error| error.to_string())),
    );
    a_drain.unwrap();
    // This closes API acceptance and clears/drains its last derived head while
    // native is alive. SDK may retain its reader until final editor reap.
    let closed_a = block_on(a_api.client.close());
    record(
        "api-a-logical-close-retained-reader",
        &json!(closed_a.as_ref().map_err(|error| error.to_string())),
    );
    closed_a.unwrap();
    let drained_a_after_close = block_on(a_api.client.drain_snapshot_releases());
    record(
        "api-a-post-logical-close-releases-drained",
        &json!(
            drained_a_after_close
                .as_ref()
                .map_err(|error| error.to_string())
        ),
    );
    drained_a_after_close.unwrap();
    let refused_a = block_on(a_api.client.parse_config_file(uri_document_identifier(
        &crate::file_uri::path_to_file_uri(&a_config),
    )));
    let is_closed = matches!(&refused_a, Err(corsa::CorsaError::Closed(_)));
    record(
        "api-a-after-logical-close-refusal",
        &json!(refused_a.as_ref().map_err(|error| error.to_string())),
    );
    assert!(is_closed);
    let after_logical_a = sockets("sockets-after-logical-close-a");
    assert!(b_socket.is_subset(&after_logical_a));
    record(
        "api-a-reader-retirement-unclaimed",
        &json!({"aSocketStillObserved":!a_socket.is_disjoint(&after_logical_a),"scope":"logical API close only; exact API/driver owners retained until final editor reap"}),
    );
    assert_eq!(
        view_type(
            &b_api.client,
            b_session.snapshot(),
            &b_session.project_handle(),
            &b_uri,
            b_position,
            "api-b-after-a-logical-close"
        ),
        "boolean"
    );
    assert_diagnostics(
        &packet(
            &mut editor,
            "editor-b-after-a-logical-close",
            &b_uri,
            &b_config,
            6,
        ),
        clean(),
    );
    assert_eq!(only_owner(&root, &native), owner);
    let released_b = block_on(b_session.snapshot().release());
    record(
        "api-b-final-snapshot-share-release",
        &json!(released_b.as_ref().map_err(|error| error.to_string())),
    );
    released_b.unwrap();
    drop(b_session);
    let b_drain = block_on(b_api.client.drain_snapshot_releases());
    record(
        "api-b-final-releases-drained",
        &json!(b_drain.as_ref().map_err(|error| error.to_string())),
    );
    b_drain.unwrap();
    let closed_b = block_on(b_api.client.close());
    record(
        "api-b-logical-close-retained-reader",
        &json!(closed_b.as_ref().map_err(|error| error.to_string())),
    );
    closed_b.unwrap();
    let drained_b_after_close = block_on(b_api.client.drain_snapshot_releases());
    record(
        "api-b-post-logical-close-releases-drained",
        &json!(
            drained_b_after_close
                .as_ref()
                .map_err(|error| error.to_string())
        ),
    );
    drained_b_after_close.unwrap();
    assert_eq!(only_owner(&root, &native), owner);
    let editor_closed = editor.shutdown();
    record("final-editor-shutdown", &json!(editor_closed));
    editor_closed.unwrap();
    control::assert_reaped(&owner);
    let joined_a = block_on(a_api.client.close());
    record(
        "api-a-close-after-owner-reap",
        &json!(joined_a.as_ref().map_err(|error| error.to_string())),
    );
    joined_a.unwrap();
    let joined_b = block_on(b_api.client.close());
    record(
        "api-b-close-after-owner-reap",
        &json!(joined_b.as_ref().map_err(|error| error.to_string())),
    );
    joined_b.unwrap();
    let final_sockets = sockets("sockets-final-after-owner-reap");
    assert!(a_socket.is_disjoint(&final_sockets));
    assert!(b_socket.is_disjoint(&final_sockets));
    assert_eq!(fs::read(&a_path).unwrap(), SOURCE_A.as_bytes());
    assert_eq!(fs::read(&b_path).unwrap(), EDITED_B.as_bytes());
    assert_eq!(fs::read(&a_config).unwrap(), a_config_before);
    assert_eq!(fs::read(&b_config).unwrap(), b_config_before);
}
