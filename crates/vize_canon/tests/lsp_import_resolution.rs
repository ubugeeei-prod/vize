use std::path::{Path, PathBuf};

use vize_canon::{CorsaBridge, CorsaBridgeConfig, LspHover, LspHoverContents, LspMarkedString};
use vize_s0::ToCompactString;

#[path = "lsp_import_resolution/shared_editor_session.rs"]
mod shared_editor_session;
#[path = "lsp_import_resolution/virtual_dependencies.rs"]
mod virtual_dependencies;

#[test]
fn bridge_virtual_overlay_preserves_workspace_project_options() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir(project_root.join("node_modules")).unwrap();

    let tsconfig = r#"{
  "compilerOptions": {
    "strict": false,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["src/**/*"]
}"#;
    std::fs::write(project_root.join("tsconfig.json"), tsconfig).unwrap();
    std::fs::write(src_dir.join("App.vue"), "<template><div /></template>\n").unwrap();
    std::fs::write(src_dir.join("Child.vue"), "<template><span /></template>\n").unwrap();
    std::fs::write(src_dir.join("util.ts"), "export const label = 'ok';\n").unwrap();

    let app_virtual_path = src_dir.join("App.vue.ts");
    let child_virtual_path = src_dir.join("Child.vue.ts");
    let app_virtual = "import Child from './Child.vue.ts';\nimport { label } from '@/util';\nfunction identity(value) { return value; }\nvoid Child;\nvoid label;\nvoid identity;\n";
    let child_virtual = "export default {};\n";

    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let diagnostics = corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let child_uri = child_virtual_path.display().to_compact_string();
        bridge
            .open_or_update_virtual_document(child_uri.as_str(), child_virtual)
            .await
            .unwrap();
        let app_uri = app_virtual_path.display().to_compact_string();
        let app_uri = bridge
            .open_or_update_virtual_document(app_uri.as_str(), app_virtual)
            .await
            .unwrap();
        let diagnostics = bridge.get_diagnostics(app_uri.as_str()).await.unwrap();
        bridge.shutdown().await.unwrap();
        diagnostics
    });

    assert_eq!(
        diagnostics.len(),
        1,
        "unexpected diagnostics: {diagnostics:#?}"
    );
    assert_eq!(diagnostics[0].code, Some(serde_json::json!(7044)));
    assert_eq!(diagnostics[0].severity, Some(4));
    assert_eq!(
        diagnostics[0].message,
        "Parameter 'value' implicitly has an 'any' type, but a better type may be inferred from usage."
    );
    assert_eq!(
        std::fs::read_to_string(project_root.join("tsconfig.json")).unwrap(),
        tsconfig
    );
    let overlay_root = project_root.join("node_modules/.vize/corsa-overlay");
    if overlay_root.join("tsconfig.json").is_file() {
        let overlay_src = overlay_root.join("src");
        assert_eq!(
            std::fs::read_to_string(overlay_src.join("App.vue.ts")).unwrap(),
            app_virtual
        );
        assert_eq!(
            std::fs::read_to_string(overlay_src.join("Child.vue.ts")).unwrap(),
            child_virtual
        );
    } else {
        assert!(
            !overlay_root.exists(),
            "editor-only mode must not partially materialize an overlay"
        );
    }
    assert!(!app_virtual_path.exists());
    assert!(!child_virtual_path.exists());
}

#[test]
fn bridge_requests_signature_help_from_corsa_tsgo() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir(project_root.join("node_modules")).unwrap();
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
    .unwrap();

    let source = "function format(value: number, precision: number): string {\n  return value.toFixed(precision);\n}\nformat(1, );\n";
    let source_path = src_dir.join("main.ts");
    std::fs::write(&source_path, source).unwrap();

    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let signature = corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let uri = source_path.display().to_compact_string();
        let uri = bridge
            .open_or_update_virtual_document(uri.as_str(), source)
            .await
            .unwrap();
        let signature = bridge
            .signature_help(uri.as_str(), 3, "format(1, ".encode_utf16().count() as u32)
            .await
            .unwrap();
        bridge.shutdown().await.unwrap();
        signature
    })
    .expect("Corsa-backed tsgo should return signature help");

    assert_eq!(signature.active_signature, Some(0));
    assert_eq!(signature.active_parameter, Some(1));
    assert_eq!(signature.signatures.len(), 1);
    let label = &signature.signatures[0].label;
    assert!(label.contains("format("), "unexpected signature: {label}");
    assert!(
        label.contains("value: number"),
        "unexpected signature: {label}"
    );
    assert!(
        label.contains("precision: number"),
        "unexpected signature: {label}"
    );
}

#[test]
fn bridge_virtual_sfc_editor_queries_resolve_relative_workspace_imports() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::create_dir(project_root.join("node_modules")).unwrap();
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
    .unwrap();
    std::fs::write(src_dir.join("App.vue"), "<template><div /></template>\n").unwrap();
    std::fs::write(
        src_dir.join("format.ts"),
        "export function format(value: number, precision: number): string { return value.toFixed(precision) }\n",
    )
    .unwrap();

    let virtual_source = "import { format } from './format';\n\nformat(1, );\n";
    let virtual_path = src_dir.join("App.vue.template.ts");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let signature = corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let uri = virtual_path.display().to_compact_string();
        let uri = bridge
            .open_or_update_virtual_document(uri.as_str(), virtual_source)
            .await
            .unwrap();
        let signature = bridge
            .signature_help(uri.as_str(), 2, "format(1, ".encode_utf16().count() as u32)
            .await
            .unwrap();
        bridge.shutdown().await.unwrap();
        signature
    })
    .expect("virtual SFC editor query should resolve the relative workspace import");

    assert_eq!(signature.active_parameter, Some(1));
    assert_eq!(signature.signatures.len(), 1);
    assert!(signature.signatures[0].label.contains("value: number"));
    assert!(signature.signatures[0].label.contains("precision: number"));
    assert!(!virtual_path.exists());
}

#[test]
fn bridge_virtual_sfc_definition_resolves_relative_workspace_import() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().unwrap();
    let project_root = project.path();
    let src_dir = project_root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
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
    .unwrap();
    std::fs::write(src_dir.join("App.vue"), "<template><div /></template>\n").unwrap();
    let dependency = "export function format(value: number): string { return value.toFixed(2) }\n";
    std::fs::write(src_dir.join("format.ts"), dependency).unwrap();

    let virtual_source = "import { format } from './format';\n\nformat(1);\n";
    let virtual_path = src_dir.join("App.vue.template.ts");
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(project_root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });

    let definitions = corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let uri = virtual_path.display().to_compact_string();
        let uri = bridge
            .open_or_update_virtual_document(uri.as_str(), virtual_source)
            .await
            .unwrap();
        let definitions = bridge.definition(uri.as_str(), 2, 1).await.unwrap();
        bridge.shutdown().await.unwrap();
        definitions
    });

    assert_eq!(
        definitions.len(),
        1,
        "unexpected definitions: {definitions:#?}"
    );
    let definition = &definitions[0];
    assert!(
        definition.uri.ends_with("/src/format.ts"),
        "unexpected definition URI: {}",
        definition.uri
    );
    assert_eq!(definition.range.start.line, 0);
    assert_eq!(
        definition.range.start.character,
        dependency.find("format").unwrap() as u32
    );
    assert!(!virtual_path.exists());
}

fn hover_contains(hover: &Option<LspHover>, expected: &str) -> bool {
    hover.as_ref().is_some_and(|hover| match &hover.contents {
        LspHoverContents::Markup(markup) => markup.value.contains(expected),
        LspHoverContents::String(value) => value.contains(expected),
        LspHoverContents::Array(items) => items.iter().any(|item| match item {
            LspMarkedString::String(value) | LspMarkedString::LanguageString { value, .. } => {
                value.contains(expected)
            }
        }),
    })
}

fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    let root = workspace_root();
    vize_s0::corsa_resolver::resolve_corsa_executable(
        vize_s0::corsa_resolver::CorsaResolveRequest {
            project_root: Some(&root),
            ..Default::default()
        },
    )
    .ok()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("vize_canon should live under crates/")
        .to_path_buf()
}
