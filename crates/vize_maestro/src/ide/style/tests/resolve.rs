use serde_json::json;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

use super::{Document, documentation, item};
use crate::ide::CompletionService;

#[test]
fn supporting_client_resolves_only_selected_documentation_without_typechecking() {
    for (css, label) in [
        (".demo { dis| }", "display"),
        (".demo { display: fl|; }", "flex"),
        (".demo { color: re|; }", "red"),
        (".demo { width: au|; }", "auto"),
        (".demo { display: |; }", "initial"),
        (".demo { display: |; }", "inherit"),
        (".demo { display: |; }", "unset"),
        (".demo { display: re|; }", "revert"),
        (".demo { display: re|; }", "revert-layer"),
        (".demo:ho| {}", ":hover"),
        (".demo::bef| {}", "::before"),
        ("@med| {}", "@media"),
        (".demo { col| }", "v-bind"),
        (".demo { col| }", ":deep"),
        (".demo { col| }", ":slotted"),
        (".demo { col| }", ":global"),
    ] {
        let document = Document::css(css, true);
        assert!(document.state.supports_completion_documentation_resolve());
        let items = document.complete();
        assert!(
            items
                .iter()
                .all(|item| item.documentation.is_none() && item.data.is_some())
        );
        let selected = item(&items, label).clone();
        let resolved = futures::executor::block_on(CompletionService::resolve(
            &document.state,
            selected.clone(),
        ));
        let eager = Document::css(css, false).complete();
        assert_eq!(documentation(&resolved), documentation(item(&eager, label)));
        let mut insertion = resolved;
        insertion.documentation = None;
        assert_eq!(
            insertion, selected,
            "resolve must preserve the whole original insertion packet"
        );
        assert!(!document.state.is_lsp_typecheck_enabled());
    }
}

#[test]
fn malformed_css_resolve_data_cannot_attach_docs_or_change_an_insertion_packet() {
    let document = Document::css(".demo { color: re|; }", true);
    for payload in [
        json!(null),
        json!({}),
        json!({ "kind": 1, "name": "red", "property": "color" }),
        json!({ "kind": "color", "name": "red" }),
        json!({ "kind": "color", "name": "red", "property": "width" }),
        json!({ "kind": "value", "name": "red", "property": "display" }),
        json!({ "kind": "color", "name": "not-an-authored-color", "property": "color" }),
        json!({ "kind": "unrecognized", "name": "red", "property": "color" }),
    ] {
        let mut selected = item(&document.complete(), "red").clone();
        selected.data = Some(json!({ "vizeCss": payload }));
        assert_eq!(
            futures::executor::block_on(CompletionService::resolve(
                &document.state,
                selected.clone()
            )),
            selected
        );
    }
    for change in ["label", "kind"] {
        let mut selected = item(&document.complete(), "red").clone();
        if change == "label" {
            selected.label = "blue".into();
        } else {
            selected.kind = Some(CompletionItemKind::PROPERTY);
        }
        assert_eq!(
            futures::executor::block_on(CompletionService::resolve(
                &document.state,
                selected.clone()
            )),
            selected
        );
    }
    let foreign = CompletionItem {
        label: "red".into(),
        data: Some(json!({ "foreign": true })),
        ..CompletionItem::default()
    };
    assert_eq!(
        futures::executor::block_on(CompletionService::resolve(&document.state, foreign.clone())),
        foreign
    );
}
