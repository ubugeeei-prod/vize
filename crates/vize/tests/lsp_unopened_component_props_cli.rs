#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]
use serde_json::{Value, json};

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use support::{Fixture, position};

const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/unopened-component-props/Child.vue.txt"
);
const PARENT: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/unopened-component-props/Parent.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/unopened-component-props/tsconfig.json"
);

#[test]
fn original_prop_references_and_rename_include_unopened_parent() {
    for newline in ["\n", "\r\n"] {
        for parent_open in [false, true] {
            let child = CHILD.replace('\n', newline);
            let parent = PARENT.replace('\n', newline);
            let files = [
                ("src/Parent.vue", parent.as_str()),
                ("tsconfig.json", CONFIG),
            ];
            let mut fixture =
                Fixture::new_with_vue_component_project(&child, "src/Child.vue", &files);
            let parent_uri = fixture.write_file("src/Parent.vue", &parent);
            assert_eq!(fixture.open(&child), json!([]));
            if parent_open {
                assert_eq!(fixture.open_file(&parent_uri, &parent), json!([]));
            }
            assert_prop_identity(
                &mut fixture,
                &child,
                &parent,
                &parent_uri,
                None,
                parent_open,
            );
            fixture.shutdown();
        }
    }
}

#[test]
fn configured_prop_navigation_keeps_aliases_shadows_foreign_props_and_open_overlays_separate() {
    for newline in ["\n", "\r\n"] {
        for parent_open in [false, true] {
            let child = CHILD.replace("</script>", "function inner(text: string) { return text; }\nconst literal = 'text'; // text\n</script>").replace('\n', newline);
            let parent_disk = PARENT.replace('\n', newline);
            let parent = if parent_open {
                format!("<!-- 😀 unsaved -->{newline}{parent_disk}")
            } else {
                parent_disk.clone()
            };
            let alias = "<script setup lang=\"ts\">\nimport Choice from './Child.vue';\nimport Other from './Other.vue';\nconst text = '😀';\n</script>\n<template>{{ '😀' }}<Choice text=\"Alias\" /><Choice :text=\"text\" /><Other text=\"Foreign\" /></template>\n".replace('\n', newline);
            let outside = PARENT
                .replace("./Child.vue", "../src/Child.vue")
                .replace('\n', newline);
            let files = [
                ("src/Parent.vue", parent_disk.as_str()),
                ("src/Aliased.vue", alias.as_str()),
                ("src/Other.vue", CHILD),
                ("ignored/Outside.vue", outside.as_str()),
                ("tsconfig.json", CONFIG),
            ];
            let mut fixture =
                Fixture::new_with_vue_component_project(&child, "src/Child.vue", &files);
            let parent_uri = fixture.write_file("src/Parent.vue", &parent_disk);
            let alias_uri = fixture.write_file("src/Aliased.vue", &alias);
            assert_eq!(fixture.open(&child), json!([]));
            if parent_open {
                assert_eq!(fixture.open_file(&parent_uri, &parent), json!([]));
            }
            assert_prop_identity(
                &mut fixture,
                &child,
                &parent,
                &parent_uri,
                Some((&alias_uri, &alias)),
                parent_open,
            );
            // A same-spelling function parameter has no public prop identity.
            let expected: Vec<_> = ["text: string", "text;"].iter().map(|needle| {
                json!({ "uri": fixture.uri, "range": token_range(&child, needle, "text") })
            }).collect();
            for needle in ["text: string", "text;"] {
                assert_eq!(
                    fixture.request_with(
                        "textDocument/references",
                        &child,
                        needle,
                        json!({ "context": { "includeDeclaration": true } })
                    ),
                    json!(expected)
                );
                let edits: Vec<_> = expected
                    .iter()
                    .map(|location| json!({ "range": location["range"], "newText": "localText" }))
                    .collect();
                assert_eq!(
                    fixture.request_with(
                        "textDocument/rename",
                        &child,
                        needle,
                        json!({ "newName": "localText" })
                    ),
                    json!({ "changes": { &fixture.uri: edits } })
                );
            }
            assert_eq!(fixture.read_file("ignored/Outside.vue"), outside);
            assert_eq!(fixture.read_file("src/Other.vue"), CHILD);
            assert_eq!(fixture.read_file("src/Aliased.vue"), alias);
            fixture.shutdown();
        }
    }
}

fn assert_prop_identity(
    fixture: &mut Fixture,
    child: &str,
    parent: &str,
    parent_uri: &str,
    alias: Option<(&str, &str)>,
    parent_open: bool,
) {
    let child_ranges = [
        token_range(child, "text?:", "text"),
        token_range(child, "text }}", "text"),
    ];
    let parent_range = attribute_range(parent, "text=\"Save\"");
    let mut locations: Vec<_> = child_ranges
        .iter()
        .map(|range| json!({ "uri": fixture.uri, "range": range }))
        .collect();
    locations.push(json!({ "uri": parent_uri, "range": parent_range }));
    let mut changes = json!({
        &fixture.uri: child_ranges.iter().map(|range| json!({ "range": range, "newText": "label" })).collect::<Vec<_>>(),
        parent_uri: [{ "range": parent_range, "newText": "label" }]
    });
    if let Some((uri, source)) = alias {
        let ranges = [
            attribute_range(source, "text=\"Alias\""),
            attribute_range(source, "text=\"text\""),
        ];
        locations.extend(
            ranges
                .iter()
                .map(|range| json!({ "uri": uri, "range": range })),
        );
        changes[uri] = json!(
            ranges
                .iter()
                .map(|range| json!({ "range": range, "newText": "label" }))
                .collect::<Vec<_>>()
        );
    }
    locations.sort_by_key(|location| {
        (
            location["uri"].as_str().unwrap().to_owned(),
            location["range"]["start"]["line"].as_u64().unwrap(),
            location["range"]["start"]["character"].as_u64().unwrap(),
        )
    });
    for needle in ["text?:", "text }}"] {
        for include_declaration in [true, false] {
            let expected: Vec<_> = locations
                .iter()
                .filter(|location| {
                    include_declaration
                        || location["uri"] != fixture.uri
                        || location["range"] != child_ranges[0]
                })
                .cloned()
                .collect();
            assert_eq!(
                fixture.request_with(
                    "textDocument/references",
                    child,
                    needle,
                    json!({ "context": { "includeDeclaration": include_declaration } })
                ),
                json!(expected),
                "references from {needle}"
            );
        }
        let rename = fixture.request_with(
            "textDocument/rename",
            child,
            needle,
            json!({ "newName": "label" }),
        );
        assert_eq!(
            rename,
            json!({ "changes": changes }),
            "complete rename from {needle}"
        );
        assert_eq!(
            apply_edits(child, &rename["changes"][&fixture.uri]),
            child
                .replace("text?:", "label?:")
                .replace("{{ text }}", "{{ label }}")
        );
        assert_eq!(
            apply_edits(parent, &rename["changes"][parent_uri]),
            parent.replace("<Child text=", "<Child label=")
        );
        if let Some((uri, source)) = alias {
            assert_eq!(
                apply_edits(source, &rename["changes"][uri]),
                source
                    .replace("<Choice text=", "<Choice label=")
                    .replace("<Choice :text=", "<Choice :label=")
            );
        }
    }
    if parent_open {
        for include_declaration in [true, false] {
            let expected: Vec<_> = locations
                .iter()
                .filter(|location| {
                    include_declaration
                        || location["uri"] != fixture.uri
                        || location["range"] != child_ranges[0]
                })
                .cloned()
                .collect();
            assert_eq!(
                fixture.request_file_with(
                    "textDocument/references",
                    parent_uri,
                    parent,
                    "text=\"Save\"",
                    json!({ "context": { "includeDeclaration": include_declaration } })
                ),
                json!(expected)
            );
        }
        assert_eq!(
            fixture.request_file_with(
                "textDocument/rename",
                parent_uri,
                parent,
                "text=\"Save\"",
                json!({ "newName": "label" })
            ),
            json!({ "changes": changes })
        );
    }
}

fn attribute_range(source: &str, needle: &str) -> Value {
    token_range(source, needle, "text")
}

fn token_range(source: &str, needle: &str, token: &str) -> Value {
    let start = position(source, needle);
    let end = json!({ "line": start["line"], "character": start["character"].as_u64().unwrap() + token.encode_utf16().count() as u64 });
    json!({ "start": start, "end": end })
}

fn apply_edits(source: &str, edits: &Value) -> String {
    let mut result = source.to_owned();
    for edit in edits.as_array().unwrap().iter().rev() {
        let start = byte_offset(source, &edit["range"]["start"]);
        let end = byte_offset(source, &edit["range"]["end"]);
        result.replace_range(start..end, edit["newText"].as_str().unwrap());
    }
    result
}

fn byte_offset(source: &str, position: &Value) -> usize {
    let line = position["line"].as_u64().unwrap() as usize;
    let units = position["character"].as_u64().unwrap() as usize;
    let line_offset: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
    let mut seen = 0;
    for (index, character) in source[line_offset..].char_indices() {
        if seen == units {
            return line_offset + index;
        }
        seen += character.len_utf16();
        assert!(seen <= units, "edit split a UTF-16 character");
    }
    assert_eq!(seen, units);
    source.len()
}
