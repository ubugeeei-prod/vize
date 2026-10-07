#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "whole authored hover payloads use std strings"
)]

use serde_json::{Value, json};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Url};

use crate::{ide::IdeContext, server::ServerState};

const APP: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/App.vue.txt"
);
const LIST: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/List.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/hover.expected.json"
);

fn native(value: &str) -> Hover {
    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: value.to_string(),
        }),
        range: None,
    }
}

#[test]
fn original_generic_props_keep_whole_declared_hover_when_native_type_is_unknown() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("List.vue"), LIST).unwrap();
    let uri = Url::from_file_path(dir.path().join("App.vue")).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(uri.clone(), APP.to_string(), 1, "vue".to_string());
    state.update_virtual_docs(&uri, APP);
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    for name in ["items", "title"] {
        let ctx = IdeContext::new(&state, &uri, APP.rfind(name).unwrap()).unwrap();
        let mut expected = expected[name].clone();
        expected.as_object_mut().unwrap().remove("range");
        for unknown in [
            "unknown".to_string(),
            "```typescript\nunknown\n```".to_string(),
            format!("```typescript\n(property) {name}: unknown\n```"),
        ] {
            let actual = super::super::hover_attribute_documented(&ctx, Some(&native(&unknown)))
                .expect("the declared generic component prop remains available");
            assert_eq!(serde_json::to_value(actual).unwrap(), expected);
        }
    }
}

#[test]
fn exact_unknown_replacement_retains_whole_native_documentation() {
    let native = native(
        "```typescript\n(property) items: unknown\n```\n\n**Rows**\n\n@example\nitems.length",
    );
    let actual =
        super::documented_prop_signature(super::HoverBuilder::new(), "items: T[]", Some(&native))
            .build();
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        json!({"contents":{"kind":"markdown","value":"```typescript\nitems: T[]\n```\n\n**Rows**\n\n@example\nitems.length"}})
    );
}

#[test]
fn concrete_native_types_and_unknown_mentions_keep_the_complete_native_hover() {
    for value in [
        "```typescript\n(property) items: { id: number; name: string }[]\n```\n\nUnknown rows are excluded.",
        "```typescript\n(property) title?: string | undefined\n```",
        "```typescript\n(property) value: unknown | string\n```",
        "```typescript\n(property) options: {\n  value: unknown;\n}\n```",
        "```typescript\n(property) value: any\n```",
        "An unknown component property.",
    ] {
        let actual = super::documented_prop_signature(
            super::HoverBuilder::new(),
            "items: T[]",
            Some(&native(value)),
        )
        .build();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            json!({"contents":{"kind":"markdown","value":value}})
        );
    }
}
