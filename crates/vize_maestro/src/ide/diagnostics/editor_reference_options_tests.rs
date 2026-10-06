//! Native mirror continuity across editor navigation and diagnostics.

use std::path::{Path, PathBuf};

use serde_json::json;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::DiagnosticService;
use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config, write_vue_test_package,
};
use crate::ide::{HoverService, IdeContext};
use crate::server::ServerState;

const SOURCE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-reference-options/App.vue.txt"
));
const BROKEN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-reference-options/AppBroken.vue.txt"
));
const STRING: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-reference-options/imports-string.d.ts.txt"
));
const NUMBER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-reference-options/imports-number.d.ts.txt"
));

#[test]
fn hover_and_diagnostics_share_native_mirrors_across_declaration_and_source_changes() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required native reference-options law has no configured runtime"
        );
        return;
    };
    crate::runtime::block_on(async {
        let project = tempfile::tempdir().unwrap();
        let root = project.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir(root.join(".nuxt")).unwrap();
        write_vue_test_package(root);
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"bundler","target":"ES2022","noEmit":true},"include":["src/**/*"]}"#,
        )
        .unwrap();
        write_corsa_config(root, &corsa_path);
        let declaration = root.join(".nuxt/imports.d.ts");
        std::fs::write(&declaration, STRING).unwrap();
        let app = root.join("src/App.vue");
        std::fs::write(&app, SOURCE).unwrap();
        let uri = Url::from_file_path(&app).unwrap();
        let state = state_for_fixture(root, &uri, SOURCE);
        state.load_workspace_config(root);
        let _observation = state.observe_editor_project_opens(&uri);

        let initial = cycle(&state, &uri, SOURCE, "string", false).await;
        let stale = state.corsa_request_scope().await;
        std::fs::write(&declaration, NUMBER).unwrap();
        refresh(&state, &[&declaration]);
        assert!(!stale.is_current());
        drop(stale);
        assert_eq!(cycle(&state, &uri, SOURCE, "number", false).await, initial);

        let renamed = root.join(".nuxt/imports.d.mts");
        std::fs::rename(&declaration, &renamed).unwrap();
        refresh(&state, &[&declaration, &renamed]);
        let renamed_root = cycle(&state, &uri, SOURCE, "number", false).await;
        assert_ne!(renamed_root, initial);

        let created = root.join(".nuxt/imports.d.cts");
        std::fs::remove_file(&renamed).unwrap();
        std::fs::write(&created, STRING).unwrap();
        refresh(&state, &[&renamed, &created]);
        let created_root = cycle(&state, &uri, SOURCE, "string", false).await;
        assert_ne!(created_root, renamed_root);

        change(&state, &uri, BROKEN, 2);
        assert_eq!(
            cycle(&state, &uri, BROKEN, "string", true).await,
            created_root
        );
        change(&state, &uri, SOURCE, 3);
        assert_eq!(
            cycle(&state, &uri, SOURCE, "string", false).await,
            created_root
        );
        let bridge = state.get_corsa_bridge().await.unwrap();
        bridge.shutdown().await.unwrap();
    });
}

fn refresh(state: &ServerState, paths: &[&Path]) {
    state.mark_corsa_disk_state_dirty();
    let uris = paths
        .iter()
        .map(|path| Url::from_file_path(path).unwrap())
        .collect::<Vec<_>>();
    assert!(state.invalidate_global_component_references(uris.iter().map(Url::as_str)));
}

fn change(state: &ServerState, uri: &Url, source: &str, version: i32) {
    assert!(state.documents.apply_changes(
        uri,
        vec![TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: source.into(),
        }],
        version,
    ));
}

async fn cycle(state: &ServerState, uri: &Url, source: &str, kind: &str, broken: bool) -> PathBuf {
    let scope = state.corsa_request_scope().await;
    let (version, revision) = state
        .documents
        .get(uri)
        .map(|document| (document.version, document.revision()))
        .unwrap();
    let bridge = state.get_corsa_bridge().await.unwrap();
    let offset = source.find("routeCount").unwrap() + 3;
    let hover_expected = json!({
        "contents":{"kind":"markdown","value":vize_l0::cstr!("\x60\x60\x60typescript\nconst routeCount: {kind}\n\x60\x60\x60")},
        "range":{"start":{"line":1,"character":14},"end":{"line":1,"character":24}}
    });
    let diagnostic_expected = if broken {
        json!([{
            "range":{"start":{"line":1,"character":25},"end":{"line":1,"character":32}},
            "severity":1,"code":2339,"source":"vize/types",
            "message":"Property 'missing' does not exist on type 'string'."
        }])
    } else {
        json!([])
    };
    for _ in 0..2 {
        let ctx = IdeContext::new(state, uri, offset).unwrap();
        let hover = HoverService::hover_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(hover).unwrap(), hover_expected);
        let diagnostics = DiagnosticService::collect_async(state, uri).await;
        assert_eq!(
            serde_json::to_value(diagnostics).unwrap(),
            diagnostic_expected
        );
    }
    assert!(scope.is_current());
    let records = serde_json::to_value(state.take_editor_project_opens()).unwrap();
    let root = records
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("root"))
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .unwrap();
    assert!(root.join("tsconfig.json").is_file());
    let request_uri = Url::from_file_path(root.join("src/App.vue.ts")).unwrap();
    let canonical = json!({"feature":"canonical","authored_uri":uri,"source":source,
        "version":version,"revision":revision,"request_uri":request_uri,"root":root});
    let diagnostics = json!({"feature":"diagnostics","authored_uri":uri,"source":source,
        "version":version,"revision":revision,"request_uri":request_uri,"root":root});
    assert_eq!(
        records,
        json!([canonical, diagnostics, canonical, diagnostics])
    );
    root
}
