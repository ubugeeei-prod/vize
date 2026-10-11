//! Actual Options API navigation, minimized from pinned tailwind-config-viewer.
#![expect(
    clippy::disallowed_types,
    reason = "exercise the shared production Corsa bridge through native definition requests"
)]

use super::{DefinitionService, IdeContext, ServerState, resolve_tsgo_binary};
use std::{fs, sync::Arc};
use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

#[test]
fn options_method_definition_returns_one_complete_authored_target() {
    crate::runtime::block_on(async {
        let corsa_path = match resolve_tsgo_binary() {
            Some(path) => path,
            None => {
                assert!(
                    std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
                    "required native Options API definition oracle has no Corsa runtime"
                );
                return;
            }
        };
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"allowJs":true,"checkJs":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["**/*"]}"#,
        )
        .unwrap();
        let source = include_str!(
            "../../../../../../tests/_fixtures/differential/lsp/options-method-definition/Canvas.vue.txt"
        );
        let path = dir.path().join("Canvas.vue");
        fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(&path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        state.update_virtual_docs(&uri, source);
        let bridge = Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();
        let offset = source.find(":is=\"sectionComponent").unwrap() + ":is=\"".len();
        let expected: Location = serde_json::from_str::<serde_json::Value>(include_str!(
            "../../../../../../tests/_fixtures/differential/lsp/options-method-definition/location.expected.json"
        ))
        .map(|value| Location {
            uri: uri.clone(),
            range: serde_json::from_value(value["range"].clone()).unwrap(),
        })
        .unwrap();
        let target = source.find("    sectionComponent").unwrap() + 4;
        let (line, character) = crate::ide::offset_to_position(source, target);
        assert_eq!(
            expected.range,
            Range::new(
                Position::new(line, character),
                Position::new(line, character + "sectionComponent".len() as u32)
            )
        );
        let native_ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let doc = crate::ide::corsa_support::open_canonical_virtual_document(&native_ctx, &bridge)
            .await
            .expect("actual canonical native document must open");
        let (line, character) =
            crate::ide::corsa_support::canonical_source_offset_to_position(&doc, offset).unwrap();
        let raw = bridge
            .definition(&doc.request_uri, line, character)
            .await
            .expect("actual native definition must answer");
        assert!(
            !raw.is_empty(),
            "actual native definition cannot use a synchronous fallback; query {line}:{character}"
        );
        let mapped =
            crate::ide::corsa_support::map_canonical_corsa_locations(&native_ctx, &doc, raw);
        assert!(
            !mapped.is_empty(),
            "synchronous fallback cannot qualify this oracle"
        );
        assert!(mapped.iter().all(|location| location == &expected));

        let mut responses = Vec::new();
        for _ in 0..3 {
            let ctx = IdeContext::new(&state, &uri, offset).unwrap();
            responses
                .push(DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone())).await);
        }
        bridge.shutdown().await.unwrap();
        assert_eq!(
            responses,
            vec![Some(GotoDefinitionResponse::Scalar(expected)); 3]
        );
    });
}
