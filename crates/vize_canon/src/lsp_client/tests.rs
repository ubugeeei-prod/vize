use super::{
    bootstrap::resolve_corsa_executable, paths::resolve_temp_dir_base,
    session::build_session_document_uri, utils::convert_diagnostics,
};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use tempfile::TempDir;
use vize_carton::cstr;

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

#[test]
fn resolves_temp_dir_under_package_root_when_node_modules_exists() {
    let case_dir = unique_case_dir("package-root");
    let source_dir = case_dir.join("playground").join("src").join("shared");
    let node_modules_vue = case_dir.join("playground").join("node_modules").join("vue");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&node_modules_vue).unwrap();

    let resolved = resolve_temp_dir_base(Some(&source_dir));

    assert_eq!(
        resolved,
        case_dir
            .join("playground")
            .join("node_modules")
            .join(".vize")
            .join("corsa")
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn falls_back_to_nearest_available_node_modules_root() {
    let case_dir = unique_case_dir("fallback");
    let workspace_root = case_dir.join("workspace");
    let source_dir = workspace_root.join("playground").join("src").join("shared");
    let node_modules_vue = workspace_root.join("node_modules").join("vue");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&node_modules_vue).unwrap();

    let resolved = resolve_temp_dir_base(Some(&source_dir));

    assert_eq!(
        resolved,
        workspace_root
            .join("node_modules")
            .join(".vize")
            .join("corsa")
    );
    assert!(!resolved.starts_with(&source_dir));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn overlay_documents_materialize_under_node_modules_vize() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).unwrap();

    let external_uri = "file:///external/App.vue.setup.ts";
    let document_uri = build_session_document_uri(external_uri, &project_root, false);
    let test_output_fragment = PathBuf::from("target")
        .join("vize-tests")
        .to_string_lossy()
        .replace('\\', "/");

    assert!(document_uri.contains("/node_modules/.vize/corsa-overlay/"));
    assert!(!document_uri.contains(test_output_fragment.as_str()));
}

#[test]
fn internal_vize_sessions_keep_overlays_inside_session_root() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir
        .path()
        .join("node_modules")
        .join(".vize")
        .join("corsa")
        .join("session");
    fs::create_dir_all(&project_root).unwrap();

    let external_uri = "file:///external/App.vue.setup.ts";
    let document_uri = build_session_document_uri(external_uri, &project_root, false);
    let test_output_fragment = PathBuf::from("target")
        .join("vize-tests")
        .to_string_lossy()
        .replace('\\', "/");

    assert!(document_uri.contains("/node_modules/.vize/corsa/session/overlays/"));
    assert!(!document_uri.contains(test_output_fragment.as_str()));
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
fn normalizes_explicit_wrapper_path_to_native_binary() {
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
        native_preview
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    );
}
