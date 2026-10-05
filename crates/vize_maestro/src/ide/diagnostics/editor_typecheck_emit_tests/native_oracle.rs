//! Independent real Vue/native LSP oracle for invalid-call quick info.

use std::path::Path;

use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaScriptVirtualDocumentRequest};

const SOURCE: &str = "import { defineEmits } from 'vue';\nconst $emit = defineEmits<{ click: []; change: [value: number] }>();\n$emit('click');\n$emit('change', 1);\n$emit('clik');\n$emit('change', 'x');\n";
const CONFIG: &str = r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext","skipLibCheck":true,"noEmit":true,"types":[]},"files":["oracle.ts"]}"#;

type OracleResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(super) async fn qualify(corsa: &Path) -> OracleResult<()> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or("workspace root is absent")?;
    let parent = workspace.join("npm/cli/target/vize-tests/template-emit-editor-oracle");
    std::fs::create_dir_all(&parent)?;
    let root = tempfile::tempdir_in(parent)?;
    let file = root.path().join("oracle.ts");
    std::fs::write(&file, SOURCE)?;
    std::fs::write(root.path().join("tsconfig.json"), CONFIG)?;
    let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa.to_path_buf()),
        working_dir: Some(root.path().to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    });
    bridge.spawn().await?;
    let options = vize_canon::virtual_ts::VirtualTsOptions::default();
    let project = bridge
        .open_script_virtual_project(CorsaScriptVirtualDocumentRequest {
            source_path: &file,
            request_path: file.to_str().ok_or("oracle path is not UTF8")?,
            code: SOURCE,
            source_type: oxc_span::SourceType::ts(),
            options: Default::default(),
            overlays: &[],
            virtual_ts_options: &options,
        })
        .await?;
    let uri = &project.document.request_uri;
    let diagnostics = bridge.get_diagnostics(uri).await?;
    let hover = bridge
        .hover(uri, 4, 1)
        .await?
        .ok_or("wrong-event hover is absent")?;
    let valid_hover = bridge
        .hover(uri, 3, 1)
        .await?
        .ok_or("valid numeric hover is absent")?;
    let payload_hover = bridge
        .hover(uri, 5, 1)
        .await?
        .ok_or("wrong-payload hover is absent")?;
    bridge.shutdown().await?;
    if let Some(capture) = std::env::var_os("VIZE_TEMPLATE_EMIT_CAPTURE") {
        let capture = std::path::PathBuf::from(capture).join("editor-oracle");
        std::fs::create_dir_all(&capture)?;
        for (name, value) in [
            ("diagnostics.json", serde_json::to_value(&diagnostics)?),
            ("hover.json", hover_response(&hover)),
            ("valid-hover.json", hover_response(&valid_hover)),
            ("payload-hover.json", hover_response(&payload_hover)),
            (
                "runtime.json",
                serde_json::json!({"nativeBinary": corsa, "workingDirectory": root.path(), "timeoutMs": 30_000}),
            ),
        ] {
            std::fs::write(capture.join(name), serde_json::to_vec_pretty(&value)?)?;
        }
        std::fs::write(capture.join("oracle.ts"), SOURCE)?;
        std::fs::write(capture.join("tsconfig.json"), CONFIG)?;
        let vue = std::fs::canonicalize(workspace.join("npm/cli/node_modules/vue"))?;
        let dom = dependency(&vue, "@vue/runtime-dom")?;
        let core = dependency(&dom, "@vue/runtime-core")?;
        let reactivity = dependency(&core, "@vue/reactivity")?;
        let shared = dependency(&reactivity, "@vue/shared")?;
        let compiler_dom = dependency(&vue, "@vue/compiler-dom")?;
        let compiler_core = dependency(&compiler_dom, "@vue/compiler-core")?;
        for path in [
            vue,
            dom,
            core,
            reactivity,
            shared,
            compiler_dom,
            compiler_core,
        ] {
            let manifest = std::fs::read(path.join("package.json"))?;
            let json: serde_json::Value = serde_json::from_slice(&manifest)?;
            assert_eq!(
                json.get("version").and_then(serde_json::Value::as_str),
                Some("3.5.35")
            );
            let target = capture.join("vue-declarations").join(
                json.get("name")
                    .and_then(serde_json::Value::as_str)
                    .ok_or("Vue package name is absent")?,
            );
            std::fs::create_dir_all(&target)?;
            std::fs::write(target.join("package.json"), manifest)?;
            let declaration = json
                .get("types")
                .and_then(serde_json::Value::as_str)
                .ok_or("Vue declaration path is absent")?;
            std::fs::write(
                target.join("types.d.ts"),
                std::fs::read(path.join(declaration))?,
            )?;
        }
    }
    assert_eq!(
        serde_json::to_value(diagnostics)?,
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
    assert_eq!(
        hover_response(&payload_hover),
        serde_json::json!({
            "contents": {"kind": "markdown", "value": "```typescript\nconst $emit: (evt: \"change\" | \"click\", ...args: never[]) => void\n```\n"},
            "range": {"start": {"line": 5, "character": 0}, "end": {"line": 5, "character": 5}}
        })
    );
    Ok(())
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

fn dependency(package: &Path, name: &str) -> OracleResult<std::path::PathBuf> {
    let parent = package.parent().ok_or("Vue package parent is absent")?;
    let modules = if parent
        .file_name()
        .ok_or("Vue package parent name is absent")?
        == "@vue"
    {
        parent
            .parent()
            .ok_or("Vue scoped package modules root is absent")?
    } else {
        parent
    };
    Ok(std::fs::canonicalize(modules.join(name))?)
}
