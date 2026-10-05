//! Genuine native/editor controls for the complete original #7949 app.

use super::DiagnosticService;
use super::editor_typecheck_fixture::{
    resolve_test_tsgo_binary, state_for_fixture, write_corsa_config,
};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url};

#[path = "../../../../vize/tests/support/vue_helper_fixture.rs"]
mod fixture;

#[test]
fn config_scoped_vue_helpers_preserve_whole_original_and_positive_editor_vectors() {
    let Some(corsa) = resolve_test_tsgo_binary() else {
        return;
    };
    for root_vue in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let vue = fixture::fixture(&root, root_vue).unwrap();
        let app = root.join("apps/web");
        write_corsa_config(&app, &corsa);
        let case = if root_vue {
            "editor-root-vue-present"
        } else {
            "editor-root-vue-absent"
        };
        let capture = fixture::capture(&root, &vue, &corsa, case).unwrap();
        for (name, source, expected) in [
            ("App", fixture::APP, Vec::new()),
            ("Counter", fixture::COUNTER, Vec::new()),
            (
                "Toggle",
                fixture::TOGGLE,
                vec![Diagnostic {
                    range: Range::new(Position::new(1, 11), Position::new(1, 19)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::Number(2322)),
                    source: Some("vize/types".into()),
                    message: "Type '42' is not assignable to type 'Booleanish | undefined'.".into(),
                    ..Default::default()
                }],
            ),
        ] {
            let uri = Url::from_file_path(app.join(vize_l0::cstr!("{name}.vue").as_str())).unwrap();
            let state = state_for_fixture(&app, &uri, source);
            state.load_workspace_config(&app);
            let diagnostics =
                crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
            if let Some(capture) = &capture {
                std::fs::write(
                    capture.join(vize_l0::cstr!("{name}.diagnostics.json").as_str()),
                    serde_json::to_vec_pretty(&diagnostics).unwrap(),
                )
                .unwrap();
            }
            assert_eq!(diagnostics, expected);
        }
        let source = fixture::TOGGLE.replace("\"42\"", "\"true\"");
        fixture::write(&root, "apps/web/Toggle.vue", &source).unwrap();
        let uri = Url::from_file_path(app.join("Toggle.vue")).unwrap();
        let state = state_for_fixture(&app, &uri, &source);
        state.load_workspace_config(&app);
        let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
        if let Some(capture) = &capture {
            std::fs::write(capture.join("Toggle-valid.vue"), source).unwrap();
            std::fs::write(
                capture.join("Toggle-valid.diagnostics.json"),
                serde_json::to_vec_pretty(&diagnostics).unwrap(),
            )
            .unwrap();
        }
        assert_eq!(diagnostics, Vec::new());
    }
}
