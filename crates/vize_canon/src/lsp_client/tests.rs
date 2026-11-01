#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use super::{
    bootstrap::resolve_corsa_executable, session::materialize_session_document,
    session_paths::build_session_document_uri, utils::convert_diagnostics,
};
use crate::file_uri::path_to_file_uri;
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use tempfile::TempDir;
use vize_l0::cstr;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(&*cstr!(
            "corsa-temp-dir-{name}-{}-{case_id}",
            std::process::id()
        ))
}

fn assert_private_storage_base(storage: &super::session_storage::SessionStorage) {
    assert_eq!(
        storage.root().file_name(),
        Some(std::ffi::OsStr::new("project"))
    );
    assert_eq!(
        storage
            .root()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap(),
        std::env::temp_dir().canonicalize().unwrap()
    );
}

#[test]
fn private_scratch_storage_resolves_package_dependencies_without_package_writes() {
    let case_dir = unique_case_dir("package-root");
    let source_dir = case_dir.join("playground").join("src").join("shared");
    let node_modules_vue = case_dir.join("playground").join("node_modules").join("vue");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&node_modules_vue).unwrap();

    let storage = super::session_storage::SessionStorage::new(&source_dir).unwrap();
    assert_private_storage_base(&storage);
    let linked = storage.root().parent().unwrap().join("node_modules/vue");
    assert_eq!(
        linked.canonicalize().unwrap(),
        node_modules_vue.canonicalize().unwrap()
    );
    assert!(!node_modules_vue.parent().unwrap().join(".vize").exists());
    drop(storage);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn private_scratch_storage_resolves_ancestor_dependencies_without_workspace_writes() {
    let case_dir = unique_case_dir("fallback");
    let workspace_root = case_dir.join("workspace");
    let source_dir = workspace_root.join("playground").join("src").join("shared");
    let node_modules_vue = workspace_root.join("node_modules").join("vue");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&node_modules_vue).unwrap();

    let storage = super::session_storage::SessionStorage::new(&source_dir).unwrap();
    assert_private_storage_base(&storage);
    let linked = storage.root().parent().unwrap().join("node_modules/vue");
    assert_eq!(
        linked.canonicalize().unwrap(),
        node_modules_vue.canonicalize().unwrap()
    );
    assert!(!workspace_root.join("node_modules/.vize").exists());
    drop(storage);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn overlay_documents_materialize_in_the_owned_private_root() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).unwrap();

    let storage = super::session_storage::SessionStorage::new(&project_root).unwrap();
    let external_uri = "file:///external/App.vue.setup.ts";
    let document_uri =
        build_session_document_uri(external_uri, &project_root, storage.root(), false);
    assert_eq!(
        document_uri,
        path_to_file_uri(&storage.root().join("external/App.vue.setup.ts"))
    );
    assert_private_storage_base(&storage);
}

#[test]
fn internal_vize_sessions_use_their_independent_owned_root() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir
        .path()
        .join("node_modules")
        .join(".vize")
        .join("corsa")
        .join("session");
    fs::create_dir_all(&project_root).unwrap();

    let storage = super::session_storage::SessionStorage::new(&project_root).unwrap();
    let external_uri = "file:///external/App.vue.setup.ts";
    let document_uri =
        build_session_document_uri(external_uri, &project_root, storage.root(), false);
    assert_eq!(
        document_uri,
        path_to_file_uri(&storage.root().join("external/App.vue.setup.ts"))
    );
    assert_private_storage_base(&storage);
}

#[test]
fn converts_lsp_diagnostics_to_legacy_shape() {
    let diagnostics = vec![Diagnostic {
        range: Range::new(Position::new(1, 2), Position::new(3, 4)),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(NumberOrString::String("TS2322".into())),
        code_description: None,
        source: Some("ts".into()),
        message: "broken".into(),
        related_information: None,
        tags: None,
        data: None,
    }];

    let converted = convert_diagnostics(&diagnostics);

    assert_eq!(converted.len(), 1);
    assert_eq!(converted[0].range.start.line, 1);
    assert_eq!(converted[0].range.start.character, 2);
    assert_eq!(converted[0].message, "broken");
    assert_eq!(
        converted[0].code,
        Some(serde_json::Value::String("TS2322".into()))
    );
}

#[test]
fn keeps_explicit_opaque_wrapper_instead_of_unrelated_native_binary() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path().join("workspace");
    let wrapper = workspace_root
        .join("packages")
        .join("demo")
        .join("node_modules/.bin/tsgo");
    let native_preview = workspace_root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");

    fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    fs::create_dir_all(native_preview.parent().unwrap()).unwrap();
    fs::write(&wrapper, "").unwrap();
    fs::write(&native_preview, "").unwrap();

    let resolved = resolve_corsa_executable(
        Some(wrapper.to_string_lossy().as_ref()),
        Some(workspace_root.to_string_lossy().as_ref()),
    )
    .unwrap();

    assert_eq!(
        resolved,
        wrapper
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    );
}

#[test]
fn keeps_encoded_vue_virtual_overlay_uri_at_real_path_inside_project() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let project = std::env::temp_dir().join(format!(
        "vize-canon-session-uri-eq={}-{}",
        std::process::id(),
        nonce
    ));
    let components = project.join("src/components");
    std::fs::create_dir_all(&components).unwrap();
    let real = components.join("Button.vue");
    std::fs::write(&real, "<template><div /></template>").unwrap();

    let virtual_path = components.join("Button.vue.ts");
    let uri = path_to_file_uri(&virtual_path);
    let encoded_component = cstr!("vize-canon-session-uri-eq%3D{}-{nonce}", std::process::id());
    assert_eq!(uri.rsplit('/').nth(3), Some(encoded_component.as_str()));

    let storage = super::session_storage::SessionStorage::new(&project).unwrap();
    let mapped = build_session_document_uri(&uri, &project, storage.root(), true);
    assert_eq!(
        mapped, uri,
        "encoded in-project overlay URI must stay stable"
    );
    assert!(
        materialize_session_document(&uri, &mapped, "export {};", storage.root())
            .unwrap()
            .is_none(),
        "stable overlay URIs must not materialize sibling files"
    );

    let mapped_without_overlay = build_session_document_uri(&uri, &project, storage.root(), false);
    assert_ne!(
        mapped_without_overlay, uri,
        "without overlay support, virtual mirrors must move into the session overlay root"
    );
    assert_eq!(
        mapped_without_overlay,
        path_to_file_uri(&storage.root().join("src/components/Button.vue.ts"))
    );
    assert!(!virtual_path.exists());

    let _ = std::fs::remove_dir_all(project);
}
