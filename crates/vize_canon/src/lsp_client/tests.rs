use super::{
    bootstrap::resolve_corsa_executable,
    paths::{find_corsa_in_local_node_modules, find_corsa_in_search_roots, resolve_temp_dir_base},
    utils::convert_diagnostics,
};
use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
use tempfile::TempDir;

fn unique_case_dir(name: &str) -> PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("__agent_only")
        .join("tests")
        .join(format!(
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
            .join("__agent_only")
            .join("vize-corsa")
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
        workspace_root.join("__agent_only").join("vize-corsa")
    );
    assert!(!resolved.starts_with(&source_dir));

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn prefers_project_local_cache_before_native_preview() {
    let case_dir = unique_case_dir("project-cache");
    let workspace_root = case_dir.join("workspace");
    let source_dir = workspace_root.join("packages").join("demo").join("src");
    let local_cache = workspace_root.join(".cache").join("tsgo");
    let native_preview = workspace_root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(local_cache.parent().unwrap()).unwrap();
    fs::create_dir_all(native_preview.parent().unwrap()).unwrap();
    fs::write(&local_cache, "").unwrap();
    fs::write(&native_preview, "").unwrap();

    let resolved = find_corsa_in_local_node_modules(Some(source_dir.to_string_lossy().as_ref()));

    assert_eq!(
        resolved,
        Some(local_cache.to_string_lossy().into_owned().into())
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn falls_back_to_sibling_corsa_bind_cache() {
    let case_dir = unique_case_dir("sibling-cache");
    let workspace_root = case_dir.join("workspace");
    let source_dir = workspace_root.join("packages").join("demo").join("src");
    let sibling_cache = case_dir.join("corsa-bind").join(".cache").join("tsgo");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(sibling_cache.parent().unwrap()).unwrap();
    fs::write(&sibling_cache, "").unwrap();

    let resolved = find_corsa_in_local_node_modules(Some(source_dir.to_string_lossy().as_ref()));

    assert_eq!(
        resolved,
        Some(sibling_cache.to_string_lossy().into_owned().into())
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn prefers_sibling_cache_before_workspace_native_preview() {
    let case_dir = unique_case_dir("sibling-over-native");
    let workspace_root = case_dir.join("workspace");
    let source_dir = workspace_root.join("packages").join("demo").join("src");
    let sibling_cache = case_dir.join("corsa-bind").join(".cache").join("tsgo");
    let native_preview = workspace_root
        .join("node_modules")
        .join("@typescript")
        .join("native-preview")
        .join("lib")
        .join("tsgo");

    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(sibling_cache.parent().unwrap()).unwrap();
    fs::create_dir_all(native_preview.parent().unwrap()).unwrap();
    fs::write(&sibling_cache, "").unwrap();
    fs::write(&native_preview, "").unwrap();

    let resolved = find_corsa_in_local_node_modules(Some(source_dir.to_string_lossy().as_ref()));

    assert_eq!(
        resolved,
        Some(sibling_cache.to_string_lossy().into_owned().into())
    );

    let _ = fs::remove_dir_all(&case_dir);
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
fn resolves_corsa_from_secondary_search_root() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let case_dir = temp_dir.path();
    let project_root = case_dir.join("project");
    let fallback_root = case_dir.join("fallback");
    let fallback_cache = fallback_root.join(".cache").join("tsgo");

    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(fallback_cache.parent().unwrap()).unwrap();
    fs::write(&fallback_cache, "").unwrap();

    let search_roots = vec![project_root.clone(), fallback_root.clone()];

    let resolved = find_corsa_in_search_roots(&search_roots);

    assert_eq!(
        resolved,
        Some(fallback_cache.to_string_lossy().into_owned().into())
    );
}

#[test]
fn prefers_workspace_native_preview_over_nested_wrapper() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path().join("workspace");
    let source_dir = workspace_root.join("packages").join("demo").join("src");
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

    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(wrapper.parent().unwrap()).unwrap();
    fs::create_dir_all(native_preview.parent().unwrap()).unwrap();
    fs::write(&wrapper, "").unwrap();
    fs::write(&native_preview, "").unwrap();

    let resolved = find_corsa_in_local_node_modules(Some(source_dir.to_string_lossy().as_ref()));

    assert_eq!(
        resolved,
        Some(native_preview.to_string_lossy().into_owned().into())
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
    );

    assert_eq!(resolved, native_preview.to_string_lossy().into_owned());
}
