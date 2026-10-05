//! Complete editor diagnostics and hover for the original #7820 input.

use std::sync::Arc;
use tower_lsp::lsp_types::{HoverContents, MarkupKind, Position, Range, Url};

use super::DiagnosticService;
use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config, write_vue_test_package,
};
use crate::ide::{HoverService, IdeContext};

#[path = "editor_typecheck_emit_tests/native_oracle.rs"]
mod native_oracle;

const ORIGINAL: &str = include_str!(
    "../../../../../tests/_fixtures/differential/typechecker/template-dollar-emit/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../../../tests/_fixtures/differential/typechecker/template-dollar-emit/tsconfig.json.txt"
);

#[test]
fn original_emit_preserves_complete_editor_diagnostics_and_typed_hover() {
    let Some(corsa) = resolve_test_tsgo_binary() else {
        return;
    };
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    let source_path = project.path().join("src/A.vue");
    std::fs::write(&source_path, ORIGINAL).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    write_vue_test_package(project.path());
    write_corsa_config(project.path(), &corsa);
    let uri = Url::from_file_path(&source_path).unwrap();
    let state = state_for_fixture(project.path(), &uri, ORIGINAL);
    state.load_workspace_config(project.path());
    let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
    let expected = vec![
        tower_lsp::lsp_types::Diagnostic {
            range: Range::new(Position::new(8, 38), Position::new(8, 44)),
            severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
            code: Some(tower_lsp::lsp_types::NumberOrString::Number(2345)),
            source: Some("vize/types".into()),
            message:
                "Argument of type '\"clik\"' is not assignable to parameter of type '\"click\"'."
                    .into(),
            ..Default::default()
        },
        tower_lsp::lsp_types::Diagnostic {
            range: Range::new(Position::new(9, 48), Position::new(9, 51)),
            severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
            code: Some(tower_lsp::lsp_types::NumberOrString::Number(2345)),
            source: Some("vize/types".into()),
            message: "Argument of type 'string' is not assignable to parameter of type 'number'."
                .into(),
            ..Default::default()
        },
    ];
    state.update_virtual_docs(&uri, ORIGINAL);
    crate::runtime::block_on(async {
        let bridge = Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa.clone()),
                working_dir: Some(project.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();
        let start = ORIGINAL.find("$emit('clik')").unwrap();
        let context = IdeContext::new(&state, &uri, start).unwrap();
        let hover = HoverService::hover_with_corsa(&context, Some(bridge.clone()))
            .await
            .unwrap();
        let payload_start = ORIGINAL.find("$emit('change', 'x')").unwrap();
        let payload_context = IdeContext::new(&state, &uri, payload_start).unwrap();
        let payload_hover = HoverService::hover_with_corsa(&payload_context, Some(bridge.clone()))
            .await
            .unwrap();
        let _ = bridge.shutdown().await;
        if let Some(capture) = std::env::var_os("VIZE_TEMPLATE_EMIT_CAPTURE") {
            let capture = std::path::PathBuf::from(capture);
            std::fs::create_dir_all(&capture).unwrap();
            std::fs::write(
                capture.join("editor-diagnostics.json"),
                serde_json::to_vec_pretty(&diagnostics).unwrap(),
            )
            .unwrap();
            std::fs::write(
                capture.join("editor-hover.json"),
                serde_json::to_vec_pretty(&hover).unwrap(),
            )
            .unwrap();
            std::fs::write(
                capture.join("editor-payload-hover.json"),
                serde_json::to_vec_pretty(&payload_hover).unwrap(),
            )
            .unwrap();
            std::fs::write(capture.join("editor-source.vue"), ORIGINAL).unwrap();
            std::fs::write(capture.join("editor-tsconfig.json"), CONFIG).unwrap();
            std::fs::write(capture.join("editor-runtime.json"), serde_json::json!({ "nativeBinary": corsa, "workingDirectory": project.path(), "timeoutMs": 30_000 }).to_string()).unwrap();
            for file in [
                "vize.config.json",
                "node_modules/vue/package.json",
                "node_modules/vue/index.d.ts",
            ] {
                let target = capture.join("editor-inputs").join(file);
                std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                std::fs::copy(project.path().join(file), target).unwrap();
            }
        }
        assert_eq!(diagnostics, expected);
        let (line, character) = crate::ide::offset_to_position(ORIGINAL, start);
        assert_eq!(
            hover.range,
            Some(Range::new(
                Position::new(line, character),
                Position::new(line, character + 5)
            ))
        );
        let HoverContents::Markup(contents) = &hover.contents else {
            panic!("unexpected hover: {hover:?}");
        };
        assert_eq!(contents.kind, MarkupKind::Markdown);
        native_oracle::qualify(&corsa).await;
        assert_eq!(
            contents.value,
            "```typescript\nconst $emit: (event: \"change\" | \"click\", ...args: never[]) => void\n```"
        );
        let (line, character) = crate::ide::offset_to_position(ORIGINAL, payload_start);
        assert_eq!(payload_hover, tower_lsp::lsp_types::Hover {
            contents: HoverContents::Markup(tower_lsp::lsp_types::MarkupContent {
                kind: MarkupKind::Markdown,
                value: "```typescript\nconst $emit: (event: \"change\" | \"click\", ...args: never[]) => void\n```".into(),
            }),
            range: Some(Range::new(Position::new(line, character), Position::new(line, character + 5))),
        });
    });
}
