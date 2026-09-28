use super::*;
use crate::ide::HoverService;
use tower_lsp::lsp_types::HoverContents;

#[test]
fn cjk_script_binding_hover_and_definition_cover_declaration_and_use() {
    let dir = tempfile::tempdir().unwrap();
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/script-cjk-navigation/App.vue.txt"
    ));
    let source_path = dir.path().join("App.vue");
    fs::write(&source_path, source).unwrap();
    fs::write(
        dir.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["*.vue"]}"#,
    )
    .unwrap();
    let uri = Url::from_file_path(&source_path).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(dir.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), source.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, source);

    let declaration = source.find("名前").unwrap();
    let usage = source.find("名前;").unwrap();
    let (line, character) = crate::ide::offset_to_position(source, declaration);
    for offset in [declaration, declaration + 1, usage, usage + "名前".len()] {
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let location = scalar_location(DefinitionService::definition(&ctx).unwrap());
        assert_eq!(location.uri, uri);
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
        assert_eq!(location.range.end.character, character + 2);
    }

    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();
        for offset in [declaration, usage] {
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            let hover = HoverService::hover_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .unwrap();
            let HoverContents::Markup(markdown) = hover.contents else {
                panic!("expected markdown hover");
            };
            assert!(
                markdown.value.contains("const 名前: \"x\""),
                "{}",
                markdown.value
            );
            let location = scalar_location(
                DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
                    .await
                    .unwrap(),
            );
            assert_eq!(location.range.start.line, line);
            assert_eq!(location.range.start.character, character);
            assert_eq!(location.range.end.character, character + 2);
        }
        let _ = bridge.shutdown().await;
    });
}
