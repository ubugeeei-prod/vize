#![cfg(test)]
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "complete stdio fixtures use JSON and authored UTF-16 positions"
)]

use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod positions;
#[path = "support/event_completion_project.rs"]
mod project;
use positions::position;
use project::Project;

const APP: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/component-native-events/App.vue.txt");
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-native-events/MySwitch.vue.txt"
);
const GENERIC: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-native-events/generic.expected.json"
);
const LITERAL: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-native-events/literal-and-handler.expected.json"
);
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-native-events/original.expected.json"
);

#[test]
fn original_complete_response_keeps_declared_types_and_all_other_candidates() {
    for (name, newline) in [("original-lf", "\n"), ("original-crlf", "\r\n")] {
        let app = APP.replace('\n', newline);
        let child = CHILD.replace('\n', newline);
        let mut project = Project::new(name, &app, &child);
        assert_eq!(
            position(&app, " />"),
            json!({ "line": 11, "character": 12 })
        );
        let original = project.original_response(parse(ORIGINAL), newline == "\n");
        project.assert_response(
            "textDocument/completion",
            json!({ "line": 11, "character": 12 }),
            original,
        );
        // The reporter's literal coordinate is on `on` in the next span,
        // not on the unchanged component tag. Preserve it as a separate law.
        project.assert_response(
            "textDocument/completion",
            json!({ "line": 12, "character": 12 }),
            parse(LITERAL),
        );
        assert_native_hover(&mut project, &app, newline);
        project.shutdown();
    }
}

#[test]
fn typed_prefixes_preserve_complete_edits_native_fallbacks_and_utf16() {
    for (name, newline) in [("typed-lf", "\n"), ("typed-crlf", "\r\n")] {
        let app = APP.replace('\n', newline);
        let child = CHILD.replace('\n', newline);
        let mut project = Project::new(name, &app, &child);
        for (index, (prefix, vector)) in [
            ("@", "typed-at"),
            ("@c", "typed-c"),
            ("@cli", "typed-cli"),
            ("@focus", "focus"),
            ("@click.stop", "modifier"),
        ]
        .iter()
        .enumerate()
        {
            let changed = APP
                .replace(
                    "<MySwitch  />",
                    &format!("<MySwitch data-icon=\"😀\" {prefix} />"),
                )
                .replace('\n', newline);
            project.change(false, &changed, index as i64 + 2);
            let start = position(&changed, &format!("{prefix} />"));
            let end = position(&changed, " />");
            let expected = prefixed(vector, &start, &end);
            project.assert_response("textDocument/completion", end, expected);
        }
        project.shutdown();
    }
}

#[test]
fn native_elements_and_runtime_js_component_events_keep_their_full_contracts() {
    let js_child = "<script setup>\ndefineEmits(['change', 'click']);\n</script>\n<template><input /></template>\n";
    for (name, child) in [("native-ts", CHILD), ("native-js", js_child)] {
        let app = if child == js_child {
            APP.replace("<script setup lang=\"ts\">", "<script setup>")
                .replace(
                    "function toggle(value: boolean): void",
                    "function toggle(value)",
                )
        } else {
            APP.to_string()
        };
        let mut project = Project::new(name, &app, child);
        let mut version = 2;
        for tag in ["MySwitch", "button", "input"] {
            for (prefix, fallback) in [("@c", "native-c"), ("@input", "input"), ("@focus", "focus")]
            {
                let changed = app.replace("<MySwitch  />", &format!("<{tag} {prefix} />"));
                project.change(false, &changed, version);
                version += 1;
                let start = position(&changed, &format!("{prefix} />"));
                let end = position(&changed, " />");
                let vector = if tag == "MySwitch" && prefix == "@c" {
                    if child == js_child { "js-c" } else { "typed-c" }
                } else {
                    fallback
                };
                project.assert_response(
                    "textDocument/completion",
                    end.clone(),
                    prefixed(vector, &start, &end),
                );
            }
        }
        for tag in ["button", "input"] {
            let changed = app.replace("<MySwitch  />", &format!("<{tag}  />"));
            project.change(false, &changed, version);
            version += 1;
            project.assert_response(
                "textDocument/completion",
                position(&changed, " />"),
                native_attributes(tag),
            );
        }
        for tag in ["MySwitch", "button"] {
            let changed = app.replace("<MySwitch  />", &format!("<{tag} @click=\"toggle\" />"));
            project.change(false, &changed, version);
            version += 1;
            project.assert_response(
                "textDocument/completion",
                position(&changed, "toggle\" />"),
                parse(LITERAL),
            );
        }
        project.shutdown();
    }
}

#[test]
fn unsaved_component_declarations_invalidate_only_matching_event_fallbacks() {
    let app = APP.replace("<MySwitch  />", "<MySwitch @ />");
    let mut project = Project::new("unsaved-declaration", &app, CHILD);
    let start = position(&app, "@ />");
    let end = position(&app, " />");
    project.assert_response(
        "textDocument/completion",
        end.clone(),
        prefixed("typed-at", &start, &end),
    );
    let changed = CHILD.replace(
        "change: [checked: boolean]; click: []",
        "change: [checked: string]; 'click.stop': []",
    );
    project.change(true, &changed, 2);
    project.assert_disk_child(CHILD);
    project.assert_response(
        "textDocument/completion",
        end.clone(),
        prefixed("unsaved-at", &start, &end),
    );
    project.change(true, CHILD, 3);
    project.assert_disk_child(CHILD);
    project.assert_response(
        "textDocument/completion",
        end.clone(),
        prefixed("typed-at", &start, &end),
    );
    project.shutdown();
}

fn assert_native_hover(project: &mut Project, original: &str, newline: &str) {
    let changed = original.replace("const on = ref(false);", "const tone: 'accent' | 'neutral' = Math.random() > 0.5 ? 'accent' : 'neutral';\nconst on = ref(false);").replace("<span>{{ on }}</span>", "<span>{{ on }} {{ tone }}</span>");
    let changed = if newline == "\r\n" {
        changed.replace("';\nconst", "';\r\nconst")
    } else {
        changed
    };
    project.change(false, &changed, 2);
    let start = position(&changed, "tone }}</span>");
    let end = position(&changed, " }}</span>");
    project.assert_response("textDocument/hover", start.clone(), json!({
        "contents": { "kind": "markdown", "value": "```typescript\nconst tone: \"accent\" | \"neutral\"\n```" },
        "range": { "start": start, "end": end }
    }));
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

// Candidate identities, complete documentation and replacement snippets are
// frozen independently in the corpus. Only authored UTF-16 request ranges vary.
fn prefixed(vector: &str, start: &Value, end: &Value) -> Value {
    let text = match vector {
        "typed-at" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/typed-at.expected.json"
        ),
        "typed-c" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/typed-c.expected.json"
        ),
        "typed-cli" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/typed-cli.expected.json"
        ),
        "focus" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/focus.expected.json"
        ),
        "modifier" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/modifier.expected.json"
        ),
        "native-c" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/native-c.expected.json"
        ),
        "input" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/input.expected.json"
        ),
        "js-c" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/js-c.expected.json"
        ),
        "unsaved-at" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/prefixes/unsaved-at.expected.json"
        ),
        _ => panic!("unknown authored vector: {vector}"),
    };
    let mut rows = parse(text);
    for item in rows.as_array_mut().unwrap() {
        assert!(item["textEdit"]["range"]["start"].is_null());
        assert!(item["textEdit"]["range"]["end"].is_null());
        item["textEdit"]["range"] = json!({ "start": start, "end": end });
    }
    rows
}

fn native_attributes(tag: &str) -> Value {
    let attributes = match tag {
        "button" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/native/button-attributes.expected.json"
        ),
        "input" => include_str!(
            "../../../tests/_fixtures/differential/lsp/component-native-events/native/input-attributes.expected.json"
        ),
        _ => panic!("unknown native vector: {tag}"),
    };
    let mut items = parse(GENERIC);
    items
        .as_array_mut()
        .unwrap()
        .extend(parse(attributes).as_array().unwrap().clone());
    items
}
