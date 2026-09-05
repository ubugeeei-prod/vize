use std::path::{Path, PathBuf};

use vize_canon::{CorsaBridge, CorsaBridgeConfig, LspLocation};
use vize_s0::ToCompactString;

#[test]
fn bridge_requests_implementation_from_corsa_tsgo() {
    let Some(corsa_path) = resolve_tsgo_binary() else {
        return;
    };
    let source = "interface Service {\n  run(): string;\n}\nclass ConcreteService implements Service {\n  run(): string { return 'ok'; }\n}\n";
    let (_project, project_root, source_path) = write_ts_project("service.ts", source);
    let (line, character) = position(source, "interface ".len() + 1);
    let (uri, locations) = request_locations(
        &corsa_path,
        &project_root,
        &source_path,
        source,
        line,
        character,
    );

    assert_contains_location(&locations, &uri, 3, "class ".len() as u32);
}

fn request_locations(
    corsa_path: &Path,
    project_root: &Path,
    source_path: &Path,
    source: &str,
    line: u32,
    character: u32,
) -> (String, Vec<LspLocation>) {
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path.to_path_buf()),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    corsa::runtime::block_on(async {
        bridge.spawn().await.expect("tsgo session");
        let uri = source_path.display().to_compact_string();
        let uri = bridge
            .open_or_update_virtual_document(uri.as_str(), source)
            .await
            .expect("open source");
        let locations = bridge
            .implementation(&uri, line, character)
            .await
            .expect("implementation request");
        bridge.shutdown().await.expect("shutdown");
        (uri.into(), locations)
    })
}

fn write_ts_project(file_name: &str, source: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let project = tempfile::TempDir::new().expect("temp project");
    let project_root = project.path().to_path_buf();
    let src = project_root.join("src");
    std::fs::create_dir_all(&src).expect("src");
    std::fs::create_dir(project_root.join("node_modules")).expect("node_modules");
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .expect("tsconfig");
    let source_path = src.join(file_name);
    std::fs::write(&source_path, source).expect("source");
    (project, project_root, source_path)
}

fn assert_contains_location(
    locations: &[LspLocation],
    uri: &str,
    start_line: u32,
    start_character: u32,
) {
    assert!(
        locations.iter().any(|location| location.uri == uri
            && location.range.start.line == start_line
            && location.range.start.character == start_character),
        "expected {uri}:{start_line}:{start_character}, got {locations:#?}",
    );
}

fn position(source: &str, offset: usize) -> (u32, u32) {
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let character = prefix[line_start..].encode_utf16().count() as u32;
    (line, character)
}

fn resolve_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    vize_s0::corsa_resolver::resolve_corsa_executable(
        vize_s0::corsa_resolver::CorsaResolveRequest {
            project_root: Some(workspace_root),
            ..Default::default()
        },
    )
    .ok()
}
