#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

const CHILD: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/same-name-rename/7994/Child.vue.txt");
const PARENT: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/same-name-rename/7994/Parent.vue.txt");

#[test]
fn native_local_rename_preserves_component_and_dom_same_name_bindings() {
    for newline in ["\n", "\r\n"] {
        for (name, replacement, directive) in
            [("label", "title", ":label"), ("id", "fieldId", ":id")]
        {
            let source = PARENT.replace('\n', newline);
            let mut fixture = Fixture::new_with_vue_component_project(
                &source,
                "Parent.vue",
                &[("Child.vue", CHILD)],
            );
            assert_eq!(fixture.open(&source), json!([]));
            let child_uri = fixture.project_root().join("Child.vue").to_path_buf();
            let child_uri = lsp_process::file_uri(&child_uri).to_string();
            assert_eq!(fixture.open_file(&child_uri, CHILD), json!([]));
            let needle = format!("{name} =");
            let response = fixture.request_with(
                "textDocument/rename",
                &source,
                &needle,
                json!({"newName": replacement}),
            );
            let expected = vec![
                text_edit(&source, &needle, name.len(), replacement),
                text_edit(
                    &source,
                    directive,
                    directive.len(),
                    &format!("{directive}=\"{replacement}\""),
                ),
            ];
            assert_eq!(edits(&response, &fixture.uri), expected, "{response:#}");
            let repaired = apply(&source, &expected);
            assert_eq!(
                repaired,
                source
                    .replacen(
                        &format!("const {name} ="),
                        &format!("const {replacement} ="),
                        1
                    )
                    .replace(
                        &format!("{directive} /"),
                        &format!("{directive}=\"{replacement}\" /")
                    ),
            );
            assert_eq!(fixture.change(&repaired, 2), json!([]));
            assert_eq!(fixture.read_file("Child.vue"), CHILD);
            fixture.shutdown();
        }
    }
}

fn text_edit(source: &str, needle: &str, length: usize, replacement: &str) -> Value {
    let start = position(source, needle);
    let mut end = start.clone();
    end["character"] = json!(start["character"].as_u64().unwrap() + length as u64);
    json!({"range":{"start":start,"end":end},"newText":replacement})
}

fn edits(response: &Value, uri: &str) -> Vec<Value> {
    let mut edits = Vec::new();
    if let Some(changes) = response["changes"].as_object() {
        for (path, entries) in changes {
            assert_eq!(path, uri, "unexpected edited file: {response:#}");
            edits.extend(entries.as_array().unwrap().iter().cloned());
        }
    }
    if let Some(changes) = response["documentChanges"].as_array() {
        for change in changes {
            assert_eq!(change["textDocument"]["uri"], uri);
            edits.extend(change["edits"].as_array().unwrap().iter().cloned());
        }
    }
    edits.sort_by_key(|edit| {
        (
            edit["range"]["start"]["line"].as_u64().unwrap(),
            edit["range"]["start"]["character"].as_u64().unwrap(),
        )
    });
    edits
}

fn apply(source: &str, edits: &[Value]) -> String {
    let offset = |position: &Value| {
        let line = position["line"].as_u64().unwrap() as usize;
        let character = position["character"].as_u64().unwrap() as usize;
        let start: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
        let mut utf16 = 0;
        source[start..]
            .char_indices()
            .find_map(|(at, ch)| {
                let matches = utf16 == character;
                utf16 += ch.len_utf16();
                matches.then_some(start + at)
            })
            .unwrap()
    };
    let mut result = source.to_owned();
    for edit in edits.iter().rev() {
        result.replace_range(
            offset(&edit["range"]["start"])..offset(&edit["range"]["end"]),
            edit["newText"].as_str().unwrap(),
        );
    }
    result
}
