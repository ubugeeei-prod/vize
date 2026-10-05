//! Complete authored component contracts; default legacy path, no migration credit.
#![cfg(all(feature = "native", feature = "glyph"))]

use tower_lsp::lsp_types::{Position, Url};
use vize_maestro::ide::{HoverService, IdeContext, position_to_offset};
use vize_maestro::server::ServerState;

const APP: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-hover-readability/App.vue.txt"
);
const LONG: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-hover-readability/LongComponent.vue.txt"
);
const SHORT: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-hover-readability/ShortComponent.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-hover-readability/responses.expected.json"
);

#[tokio::test]
async fn complete_long_and_short_component_hovers_preserve_types_and_authored_ranges() {
    let workspace = tempfile::tempdir().expect("hover fixture workspace");
    for (name, source) in [
        ("App.vue", APP),
        ("LongComponent.vue", LONG),
        ("ShortComponent.vue", SHORT),
    ] {
        std::fs::write(workspace.path().join(name), source).expect("original fixture bytes");
    }
    let uri = Url::from_file_path(workspace.path().join("App.vue")).expect("fixture URI");
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), APP.to_owned(), 1, "vue".to_owned());
    state.update_virtual_docs(&uri, APP);
    let expected: Vec<serde_json::Value> =
        serde_json::from_str(EXPECTED).expect("whole authored references");
    for row in expected {
        let position: Position =
            serde_json::from_value(row["position"].clone()).expect("authored position");
        let offset =
            position_to_offset(APP, position.line, position.character).expect("fixture offset");
        let ctx = IdeContext::new(&state, &uri, offset).expect("fixture context");
        let actual = HoverService::hover_with_corsa(&ctx, None).await;
        assert_eq!(
            serde_json::to_value(actual).expect("complete Hover"),
            row["result"]
        );
    }
}
