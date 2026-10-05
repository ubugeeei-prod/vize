//! Complete editor diagnostics and hover for the original #7819 input.

use std::sync::Arc;
use tower_lsp::lsp_types::{HoverContents, MarkupKind, Position, Range, Url};

use super::DiagnosticService;
use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config, write_vue_test_package,
};
use crate::ide::{HoverService, IdeContext};

const ORIGINAL: &str = include_str!(
    "../../../../../tests/_fixtures/differential/typechecker/undefined-default-prop/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../../../tests/_fixtures/differential/typechecker/undefined-default-prop/tsconfig.json.txt"
);

#[test]
fn undefined_default_preserves_complete_editor_diagnostics_and_optional_hover() {
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
    assert_eq!(diagnostics, Vec::new());
    state.update_virtual_docs(&uri, ORIGINAL);
    crate::runtime::block_on(async {
        let bridge = Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa),
                working_dir: Some(project.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();
        let start = ORIGINAL.find("v-if=\"onLoad").unwrap() + "v-if=\"".len();
        let context = IdeContext::new(&state, &uri, start).unwrap();
        let hover = HoverService::hover_with_corsa(&context, Some(bridge.clone()))
            .await
            .unwrap();
        let _ = bridge.shutdown().await;
        let (line, character) = crate::ide::offset_to_position(ORIGINAL, start);
        assert_eq!(
            hover.range,
            Some(Range::new(
                Position::new(line, character),
                Position::new(line, character + 6)
            ))
        );
        let HoverContents::Markup(contents) = &hover.contents else {
            panic!("unexpected hover: {hover:?}");
        };
        assert_eq!(contents.kind, MarkupKind::Markdown);
        assert_eq!(
            contents.value,
            "```typescript\nconst onLoad: (() => void) | undefined\n```"
        );
        if let Some(capture) = std::env::var_os("VIZE_DEFAULT_PROP_CAPTURE") {
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
            std::fs::write(capture.join("editor-source.vue"), ORIGINAL).unwrap();
            std::fs::write(capture.join("editor-tsconfig.json"), CONFIG).unwrap();
        }
    });
}
