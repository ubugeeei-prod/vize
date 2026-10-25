use super::*;
use crate::ide::{IdeContext, ecosystem};
use tower_lsp::lsp_types::Url;

const SOURCE: &str = "<script setup lang=\"ts\">\nimport { useRoute } from 'vue-router'\nconst route = useRoute()\nroute.params.page\n</script>";

fn state(configured: bool, typecheck: bool) -> (tempfile::TempDir, ServerState, Url) {
    let root = tempfile::tempdir().unwrap();
    let config = root.path().join("tsconfig.json");
    let options = if configured {
        r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#
    } else {
        "{}"
    };
    std::fs::write(&config, options).unwrap();
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&serde_json::json!({
        "typecheck": typecheck, "ecosystem": true, "lint": false,
    })));
    *state.project_contexts.compiler_configuration.write() = Some(Arc::new(
        vize_canon::CompilerConfigurationSnapshot::capture(root.path(), Some(&config)).unwrap(),
    ));
    let uri = Url::from_file_path(root.path().join("src/pages/products/[id=int].vue")).unwrap();
    (root, state, uri)
}

#[test]
fn typed_route_editor_authority_requires_both_native_typecheck_and_project_opt_in() {
    for (configured, typecheck, expected) in [
        (false, false, false),
        (false, true, false),
        (true, false, false),
        (true, true, true),
    ] {
        let (root, state, uri) = state(configured, typecheck);
        assert_eq!(state.typed_router_typecheck_enabled(), expected);
        let ctx = IdeContext::testing(
            &state,
            &uri,
            SOURCE.find("route.params.page").unwrap() + "route.params.".len(),
            SOURCE.into(),
        );
        let diagnostics = ecosystem::diagnostics(&state, &uri, ctx.descriptor().unwrap());
        let completions = ecosystem::completions(&ctx);
        if expected {
            assert_eq!(diagnostics, Vec::new());
            assert_eq!(completions, Vec::new());
        } else {
            assert_eq!(
                serde_json::to_value(diagnostics).unwrap(),
                serde_json::json!([{
                    "range":{"start":{"line":3,"character":13},"end":{"line":3,"character":17}},
                    "severity":2,"code":"ecosystem/vue-router-route-param","source":"vize/ecosystem",
                    "message":"Route param `page` is not defined by this page file. Available params: id"
                }])
            );
            assert_eq!(
                serde_json::to_value(completions).unwrap(),
                serde_json::json!([{
                    "label":"id","kind":10,"detail":"Vue Router file route param: string","sortText":"0"
                }])
            );
        }
        // Ordinary hint requests do not reread a changed configuration.
        std::fs::write(root.path().join("tsconfig.json"), "{").unwrap();
        assert_eq!(state.typed_router_typecheck_enabled(), expected);
        assert_eq!(
            ecosystem::diagnostics(&state, &uri, ctx.descriptor().unwrap()).is_empty(),
            expected
        );
    }
}
