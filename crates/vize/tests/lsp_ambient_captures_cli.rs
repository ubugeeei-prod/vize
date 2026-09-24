#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

#[test]
fn ambient_captures_preserve_native_editor_identity_and_unsaved_types() {
    for newline in ["\n", "\r\n"] {
        let source = "<script setup lang=\"ts\">\nconst prefix = '\u{1f600}'; const local = 'fixed';\ndeclare const label: typeof local;\nconst exact: 'fixed' = label;\nfunction inner(local: number) { return local + 1; }\n</script>\n<template>{{ local.toUpperCase() }} {{ label.toUpperCase() }}</template>\n".replace('\n', newline);
        let mut fixture = Fixture::new_with_vue_and_cross_file(&source);
        assert_eq!(fixture.open(&source), json!([]));
        for needle in ["local =", "local;", "local.toUpperCase"] {
            assert_eq!(
                fixture.request("textDocument/hover", &source, needle),
                json!({
                    "contents": { "kind": "markdown", "value": "```typescript\nconst local: \"fixed\"\n```" },
                    "range": token_range(&source, needle, "local")
                })
            );
            assert_eq!(
                fixture.request("textDocument/definition", &source, needle),
                json!({
                    "uri": fixture.uri, "range": token_range(&source, "local =", "local")
                })
            );
            assert_local_rename(&mut fixture, &source, needle);
        }
        assert_label(&mut fixture, &source, "label.toUpperCase", "\"fixed\"");
        let broken = source.replace("label.toUpperCase()", "label.toFixed()");
        assert_eq!(
            fixture.change(&broken, 2),
            json!([{
                "code": 2551,
                "message": "Property 'toFixed' does not exist on type '\"fixed\"'. Did you mean 'fixed'?",
                "range": token_range(&broken, "toFixed", "toFixed"),
                "severity": 1, "source": "vize/types"
            }])
        );
        assert_eq!(fixture.change(&source, 3), json!([]));
        let number = source
            .replace("'fixed'", "42")
            .replace("toUpperCase", "toFixed");
        assert_eq!(fixture.change(&number, 4), json!([]));
        assert_label(&mut fixture, &number, "label.toFixed", "42");
        assert_eq!(fixture.change(&source, 5), json!([]));
        fixture.shutdown();
    }
}

#[test]
fn ambient_heritage_keeps_authored_constructor_navigation_and_errors() {
    let source = "<script setup lang=\"ts\">\nclass Base<T> { constructor(public value: T) {} }\ndeclare class Derived extends Base<string> {}\nconst node = new Derived('fixed');\n</script>\n<template>{{ node.value.toUpperCase() }}</template>\n";
    let mut fixture = Fixture::new_with_vue(source);
    assert_eq!(fixture.open(source), json!([]));
    assert_eq!(
        fixture.request("textDocument/definition", source, "Base<string>"),
        json!({
            "uri": fixture.uri, "range": token_range(source, "Base<T>", "Base")
        })
    );
    let rename = fixture.request_with(
        "textDocument/rename",
        source,
        "Base<string>",
        json!({ "newName": "Parent" }),
    );
    let mut actual = rename["changes"][&fixture.uri]
        .as_array()
        .unwrap_or_else(|| panic!("{rename:#}"))
        .clone();
    let mut expected: Vec<Value> = ["Base<T>", "Base<string>"]
        .into_iter()
        .map(|needle| {
            json!({
                "newText": "Parent", "range": token_range(source, needle, "Base")
            })
        })
        .collect();
    sort_edits(&mut actual);
    sort_edits(&mut expected);
    assert_eq!(actual, expected);
    assert_eq!(
        rename["changes"].as_object().unwrap().len(),
        1,
        "{rename:#}"
    );

    let broken = "<script setup lang=\"ts\">\nconst Base = 1;\ndeclare class Label extends Base {}\n</script>\n<template><div /></template>\n";
    assert_eq!(
        fixture.change(broken, 2),
        json!([{
            "code": 2507,
            "message": "Type '1' is not a constructor function type.",
            "range": token_range(broken, "Base {}", "Base"),
            "severity": 1, "source": "vize/types"
        }])
    );
    assert_eq!(fixture.change(source, 3), json!([]));
    fixture.shutdown();
}

fn assert_local_rename(fixture: &mut Fixture, source: &str, needle: &str) {
    let mut expected: Vec<Value> = ["local =", "local;", "local.toUpperCase"]
        .into_iter()
        .map(|usage| {
            json!({
                "range": token_range(source, usage, "local"), "newText": "renamedLocal"
            })
        })
        .collect();
    let rename = fixture.request_with(
        "textDocument/rename",
        source,
        needle,
        json!({ "newName": "renamedLocal" }),
    );
    let mut actual = rename["changes"][&fixture.uri]
        .as_array()
        .unwrap_or_else(|| panic!("{rename:#}"))
        .clone();
    sort_edits(&mut expected);
    sort_edits(&mut actual);
    assert_eq!(actual, expected, "rename from {needle}");
    assert_eq!(
        rename["changes"].as_object().unwrap().len(),
        1,
        "{rename:#}"
    );
    assert!(rename.get("documentChanges").is_none(), "{rename:#}");
    let references = fixture.request_with(
        "textDocument/references",
        source,
        needle,
        json!({"context": {"includeDeclaration": true}}),
    );
    let mut actual: Vec<Value> = references
        .as_array()
        .unwrap_or_else(|| panic!("{references:#}"))
        .iter()
        .map(|location| {
            assert_eq!(location["uri"], fixture.uri);
            json!({ "range": location["range"], "newText": "renamedLocal" })
        })
        .collect();
    sort_edits(&mut actual);
    assert_eq!(actual, expected, "references from {needle}");
}

fn sort_edits(edits: &mut [Value]) {
    edits.sort_by_key(|edit| {
        (
            edit["range"]["start"]["line"].as_u64(),
            edit["range"]["start"]["character"].as_u64(),
        )
    });
}

fn assert_label(fixture: &mut Fixture, source: &str, needle: &str, ty: &str) {
    assert_eq!(
        fixture.request("textDocument/hover", source, needle),
        json!({
            "contents": { "kind": "markdown", "value": format!("```typescript\nconst label: {ty}\n```") },
            "range": token_range(source, needle, "label")
        })
    );
    assert_eq!(
        fixture.request("textDocument/definition", source, needle),
        json!({
            "uri": fixture.uri, "range": token_range(source, "label:", "label")
        })
    );
}

fn token_range(source: &str, needle: &str, token: &str) -> Value {
    let start = position(source, needle);
    let end = json!({ "line": start["line"], "character": start["character"].as_u64().unwrap() + token.encode_utf16().count() as u64 });
    json!({ "start": start, "end": end })
}
