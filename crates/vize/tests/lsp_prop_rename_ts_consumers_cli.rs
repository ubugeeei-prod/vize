#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
use crate::support::lsp_vue_project as support;
use serde_json::{Value, json};
use support::Fixture;

// A component prop consumed from plain TypeScript: a render function
// argument and a contextually typed options object, the two shapes Misskey
// uses for `MkCodeInline` (`MkMfm.ts` and `MkCodeInline.stories.impl.ts`).
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/prop-rename-ts-consumers/Child.vue.txt"
);
const RENDER: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/prop-rename-ts-consumers/render.ts.txt"
);
const OPTIONS: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/prop-rename-ts-consumers/options.ts.txt"
);
const CONFIG: &str = r#"{
  "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}"#;

#[test]
fn prop_rename_edits_every_typescript_consumer_that_references_finds() {
    let files = [
        ("src/render.ts", RENDER),
        ("src/options.ts", OPTIONS),
        ("tsconfig.json", CONFIG),
    ];
    let mut fixture = Fixture::new_with_vue_component_project(CHILD, "src/Child.vue", &files);
    let render_uri = fixture.write_file("src/render.ts", RENDER);
    let options_uri = fixture.write_file("src/options.ts", OPTIONS);
    assert_eq!(fixture.open(CHILD), json!([]));

    let references = fixture.request_with(
        "textDocument/references",
        CHILD,
        "code: string",
        json!({ "context": { "includeDeclaration": true } }),
    );
    let referenced = locations_by_uri(&references);
    for uri in [&render_uri, &options_uri] {
        assert!(
            referenced
                .iter()
                .any(|(location_uri, _)| location_uri == uri),
            "references must find the TypeScript consumer {uri}: {references}"
        );
    }

    let rename = fixture.request_with(
        "textDocument/rename",
        CHILD,
        "code: string",
        json!({ "newName": "source" }),
    );
    let renamed = edits_by_uri(&rename);
    for (uri, range) in &referenced {
        assert!(
            renamed
                .iter()
                .any(|(edit_uri, edit_range)| edit_uri == uri && edit_range == range),
            "rename must edit every referenced location, missing {uri} {range}: {rename}"
        );
    }
    fixture.shutdown();
}

fn locations_by_uri(value: &Value) -> Vec<(String, Value)> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|location| {
            (
                location["uri"].as_str().unwrap_or_default().to_owned(),
                location["range"].clone(),
            )
        })
        .collect()
}

fn edits_by_uri(value: &Value) -> Vec<(String, Value)> {
    let mut edits = Vec::new();
    if let Some(changes) = value["changes"].as_object() {
        for (uri, entries) in changes {
            for entry in entries.as_array().into_iter().flatten() {
                edits.push((uri.clone(), entry["range"].clone()));
            }
        }
    }
    for change in value["documentChanges"].as_array().into_iter().flatten() {
        let uri = change["textDocument"]["uri"].as_str().unwrap_or_default();
        for entry in change["edits"].as_array().into_iter().flatten() {
            edits.push((uri.to_owned(), entry["range"].clone()));
        }
    }
    edits
}
