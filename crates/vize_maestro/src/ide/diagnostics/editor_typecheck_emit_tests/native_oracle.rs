//! Independent real Vue/native LSP oracle for invalid-call quick info.

use std::path::Path;

use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaScriptVirtualDocumentRequest};

const SOURCE: &str = "import { defineEmits } from 'vue';\nconst $emit = defineEmits<{ click: []; change: [value: number] }>();\n$emit('click');\n$emit('change', 1);\n$emit('clik');\n$emit('change', 'x');\n";
const CONFIG: &str = r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext","skipLibCheck":true,"noEmit":true,"types":[]},"files":["oracle.ts"]}"#;

pub(super) async fn qualify(corsa: &Path) {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let parent = workspace.join("npm/cli/target/vize-tests/template-emit-editor-oracle");
    std::fs::create_dir_all(&parent).unwrap();
    let root = tempfile::tempdir_in(parent).unwrap();
    let file = root.path().join("oracle.ts");
    std::fs::write(&file, SOURCE).unwrap();
    std::fs::write(root.path().join("tsconfig.json"), CONFIG).unwrap();
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa.to_path_buf()),
        working_dir: Some(root.path().to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });
    bridge.spawn().await.unwrap();
    let options = vize_canon::virtual_ts::VirtualTsOptions::default();
    let project = bridge
        .open_script_virtual_project(CorsaScriptVirtualDocumentRequest {
            source_path: &file,
            request_path: file.to_str().unwrap(),
            code: SOURCE,
            source_type: oxc_span::SourceType::ts(),
            options: Default::default(),
            overlays: &[],
            virtual_ts_options: &options,
        })
        .await
        .unwrap();
    let uri = &project.document.request_uri;
    let diagnostics = bridge.get_diagnostics(uri).await.unwrap();
    let hover = bridge.hover(uri, 4, 1).await.unwrap().unwrap();
    let valid_hover = bridge.hover(uri, 3, 1).await.unwrap().unwrap();
    bridge.shutdown().await.unwrap();
    if let Some(capture) = std::env::var_os("VIZE_TEMPLATE_EMIT_CAPTURE") {
        let capture = std::path::PathBuf::from(capture).join("editor-oracle");
        std::fs::create_dir_all(&capture).unwrap();
        for (name, value) in [
            (
                "diagnostics.json",
                serde_json::to_value(&diagnostics).unwrap(),
            ),
            ("hover.json", hover_response(&hover)),
            ("valid-hover.json", hover_response(&valid_hover)),
            (
                "runtime.json",
                serde_json::json!({"nativeBinary": corsa, "workingDirectory": root.path(), "timeoutMs": 30_000}),
            ),
        ] {
            std::fs::write(
                capture.join(name),
                serde_json::to_vec_pretty(&value).unwrap(),
            )
            .unwrap();
        }
        std::fs::write(capture.join("oracle.ts"), SOURCE).unwrap();
        std::fs::write(capture.join("tsconfig.json"), CONFIG).unwrap();
        let vue = std::fs::canonicalize(workspace.join("npm/cli/node_modules/vue")).unwrap();
        let dom = dependency(&vue, "@vue/runtime-dom");
        let core = dependency(&dom, "@vue/runtime-core");
        let reactivity = dependency(&core, "@vue/reactivity");
        let shared = dependency(&reactivity, "@vue/shared");
        let compiler_dom = dependency(&vue, "@vue/compiler-dom");
        let compiler_core = dependency(&compiler_dom, "@vue/compiler-core");
        for path in [
            vue,
            dom,
            core,
            reactivity,
            shared,
            compiler_dom,
            compiler_core,
        ] {
            let manifest = std::fs::read(path.join("package.json")).unwrap();
            let json: serde_json::Value = serde_json::from_slice(&manifest).unwrap();
            assert_eq!(json["version"], "3.5.35");
            let target = capture
                .join("vue-declarations")
                .join(json["name"].as_str().unwrap());
            std::fs::create_dir_all(&target).unwrap();
            std::fs::write(target.join("package.json"), manifest).unwrap();
            let declaration = json["types"].as_str().unwrap();
            std::fs::write(
                target.join("types.d.ts"),
                std::fs::read(path.join(declaration)).unwrap(),
            )
            .unwrap();
        }
    }
    assert_eq!(
        serde_json::to_value(diagnostics).unwrap(),
        serde_json::json!([
            { "range": {"start": {"line": 4, "character": 6}, "end": {"line": 4, "character": 12}}, "severity": 1, "code": 2345, "source": "ts", "message": "Argument of type '\"clik\"' is not assignable to parameter of type '\"click\"'.", "relatedInformation": null },
            { "range": {"start": {"line": 5, "character": 16}, "end": {"line": 5, "character": 19}}, "severity": 1, "code": 2345, "source": "ts", "message": "Argument of type 'string' is not assignable to parameter of type 'number'.", "relatedInformation": null }
        ])
    );
    assert_eq!(
        hover_response(&hover),
        serde_json::json!({
            "contents": {"kind": "markdown", "value": "```typescript\nconst $emit: (evt: \"change\" | \"click\", ...args: never[]) => void\n```\n"},
            "range": {"start": {"line": 4, "character": 0}, "end": {"line": 4, "character": 5}}
        })
    );
    assert_eq!(
        hover_response(&valid_hover),
        serde_json::json!({
            "contents": {"kind": "markdown", "value": "```typescript\nconst $emit: (evt: \"change\", value: number) => void\n```\n"},
            "range": {"start": {"line": 3, "character": 0}, "end": {"line": 3, "character": 5}}
        })
    );
}

fn hover_response(hover: &vize_canon::LspHover) -> serde_json::Value {
    use vize_canon::{LspHoverContents, LspMarkedString};
    let contents = match &hover.contents {
        LspHoverContents::Markup(value) => {
            serde_json::json!({"kind": value.kind, "value": value.value})
        }
        LspHoverContents::String(value) => serde_json::json!(value),
        LspHoverContents::Array(values) => serde_json::Value::Array(
            values
                .iter()
                .map(|value| match value {
                    LspMarkedString::String(value) => serde_json::json!(value),
                    LspMarkedString::LanguageString { language, value } => {
                        serde_json::json!({"language": language, "value": value})
                    }
                })
                .collect(),
        ),
    };
    serde_json::json!({"contents": contents, "range": hover.range})
}

fn dependency(package: &Path, name: &str) -> std::path::PathBuf {
    let parent = package.parent().unwrap();
    let modules = if parent.file_name().unwrap() == "@vue" {
        parent.parent().unwrap()
    } else {
        parent
    };
    std::fs::canonicalize(modules.join(name)).unwrap()
}
