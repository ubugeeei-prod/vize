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

const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/type-alias-navigation/8011/E.vue.txt");

#[test]
fn native_local_type_alias_prop_links_declaration_and_template_uses() {
    for newline in ["\n", "\r\n"] {
        for invocation in [
            "defineProps<Props>();",
            "withDefaults(defineProps<Props>(), { tone: 'light' });",
            "const { tone = 'light' } = defineProps<Props>();",
        ] {
            let source = ORIGINAL
                .replace("defineProps<Props>();", invocation)
                .replace('\n', newline);
            let mut fixture = Fixture::new_with_vue_component_project(&source, "E.vue", &[]);
            assert_eq!(fixture.open(&source), json!([]));
            for (name, declaration, usage, replacement) in [
                ("hidden", "hidden?:", "hidden\"", "concealed"),
                ("tone", "tone?:", "tone\"", "palette"),
            ] {
                // Destructuring tone gives that local binding its own name;
                // the independent bare hidden prop retains the public edge.
                if invocation != "defineProps<Props>();" && name == "tone" {
                    continue;
                }
                let ranges = [
                    range(&source, declaration, name),
                    range(&source, usage, name),
                ];
                let expected: Vec<_> = ranges
                    .iter()
                    .map(|range| json!({"uri":fixture.uri,"range":range}))
                    .collect();
                for query in [declaration, usage] {
                    assert_eq!(
                        fixture.request_with(
                            "textDocument/references",
                            &source,
                            query,
                            json!({"context":{"includeDeclaration":true}})
                        ),
                        json!(expected)
                    );
                    let response = fixture.request_with(
                        "textDocument/rename",
                        &source,
                        query,
                        json!({"newName":replacement}),
                    );
                    let expected_edits: Vec<_> = ranges
                        .iter()
                        .map(|range| json!({"range":range,"newText":replacement}))
                        .collect();
                    assert_eq!(
                        edits(&response, &fixture.uri),
                        expected_edits,
                        "{response:#}"
                    );
                }
                let definition = fixture.request("textDocument/definition", &source, usage);
                assert_eq!(definition, json!({"uri":fixture.uri,"range":ranges[0]}));
            }
            let response = fixture.request_with(
                "textDocument/rename",
                &source,
                "hidden?:",
                json!({"newName":"concealed"}),
            );
            let repaired = apply(&source, &edits(&response, &fixture.uri));
            assert_eq!(repaired, source.replace("hidden", "concealed"));
            assert_eq!(fixture.change(&repaired, 2), json!([]));
            fixture.shutdown();
        }
    }
}

fn range(source: &str, needle: &str, token: &str) -> Value {
    let start = position(source, needle);
    let mut end = start.clone();
    end["character"] = json!(start["character"].as_u64().unwrap() + token.len() as u64);
    json!({"start":start,"end":end})
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
