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

const CHILDREN: [(&str, &str); 4] = [
    (
        "inline",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/mixed-prop-roles/Child-inline.vue.txt"
        ),
    ),
    (
        "alias",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/mixed-prop-roles/Child-alias.vue.txt"
        ),
    ),
    (
        "interface",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/mixed-prop-roles/Child-interface.vue.txt"
        ),
    ),
    (
        "withDefaults",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/mixed-prop-roles/Child-defaults.vue.txt"
        ),
    ),
];
const PARENT: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/mixed-prop-roles/Parent.vue.txt"
);

#[derive(Clone, Copy, Debug)]
enum Query {
    ChildProperty,
    ParentArgument,
    ParentLocal,
}

#[test]
fn native_public_prop_rename_from_child_preserves_dom_and_parent_local_roles() {
    assert_rename(Query::ChildProperty);
}

#[test]
fn native_public_prop_rename_from_parent_argument_preserves_dom_and_local_roles() {
    assert_rename(Query::ParentArgument);
}

#[test]
fn native_parent_local_rename_preserves_child_public_prop_and_dom_roles() {
    assert_rename(Query::ParentLocal);
}

fn assert_rename(query: Query) {
    for (form, child) in CHILDREN {
        for newline in ["\n", "\r\n"] {
            let child = child.replace('\n', newline);
            let parent = PARENT.replace('\n', newline);
            let context = format!("{form}, {query:?}, newline={newline:?}");
            let mut fixture = Fixture::new_with_cross_file_component_project(
                &child,
                "Child.vue",
                &[("Parent.vue", &parent)],
            );
            let child_uri = fixture.uri.clone();
            let parent_uri = fixture.write_file("Parent.vue", &parent);
            assert_eq!(fixture.open(&child), json!([]), "{context}");
            assert_eq!(
                fixture.open_file(&parent_uri, &parent),
                json!([]),
                "{context}"
            );
            let (query_uri, source, needle) = match query {
                Query::ChildProperty => (&child_uri, &child, "id: string"),
                Query::ParentArgument => (&parent_uri, &parent, "id />"),
                Query::ParentLocal => (&parent_uri, &parent, "id ="),
            };
            let references = fixture.request_file_with(
                "textDocument/references",
                query_uri,
                source,
                needle,
                json!({"context":{"includeDeclaration":true}}),
            );
            let response = fixture.request_file_with(
                "textDocument/rename",
                query_uri,
                source,
                needle,
                json!({"newName":"field"}),
            );
            // Apply the service's actual complete edit vectors, including on a
            // failing regression, so the post-edit diagnostic evidence is real.
            let changes = response["changes"]
                .as_object()
                .expect("WorkspaceEdit changes");
            for uri in changes.keys() {
                assert!(uri == &child_uri || uri == &parent_uri, "{response:#}");
            }
            let repaired_child = apply(&child, changes.get(&child_uri));
            let repaired_parent = apply(&parent, changes.get(&parent_uri));
            fixture.write_file("Child.vue", &repaired_child);
            fixture.write_file("Parent.vue", &repaired_parent);
            let child_diagnostics = fixture.change_file(&child_uri, &repaired_child, 2);
            let parent_diagnostics = fixture.change_file(&parent_uri, &repaired_parent, 2);
            fixture.shutdown();

            let (expected_references, expected_response, expected_child, expected_parent) =
                if matches!(query, Query::ParentLocal) {
                    (
                        json!([
                            location(&parent_uri, &parent, "id ="),
                            location(&parent_uri, &parent, "id />"),
                        ]),
                        json!({"changes":{&parent_uri:[
                            text_edit(&parent, "id =", 2, "field"),
                            text_edit(&parent, ":id", 3, ":id=\"field\""),
                        ]}}),
                        child.clone(),
                        parent
                            .replace("const id =", "const field =")
                            .replace(":id />", ":id=\"field\" />"),
                    )
                } else {
                    (
                        json!([
                            location(&child_uri, &child, "id: string"),
                            location(&child_uri, &child, "id />"),
                            location(&parent_uri, &parent, "id />"),
                        ]),
                        json!({"changes":{
                            &child_uri:[
                                text_edit(&child, "id: string", 2, "field"),
                                text_edit(&child, ":id", 3, ":id=\"field\""),
                            ],
                            &parent_uri:[
                                text_edit(&parent, ":id", 3, ":field=\"id\""),
                            ],
                        }}),
                        child
                            .replace("id: string", "field: string")
                            .replace(":id />", ":id=\"field\" />"),
                        parent.replace(":id />", ":field=\"id\" />"),
                    )
                };
            // Compare whole responses: no locations, edits, or fields are
            // removed to turn an unsafe native transaction into a passing one.
            assert_eq!(references, expected_references, "{context}; {response:#}");
            assert_eq!(response, expected_response, "{context}");
            assert_eq!(repaired_child, expected_child, "{context}");
            assert_eq!(repaired_parent, expected_parent, "{context}");
            assert_eq!(fixture.read_file("Child.vue"), expected_child, "{context}");
            assert_eq!(
                fixture.read_file("Parent.vue"),
                expected_parent,
                "{context}"
            );
            assert_eq!(child_diagnostics, json!([]), "{context}");
            assert_eq!(parent_diagnostics, json!([]), "{context}");
        }
    }
}

fn location(uri: &str, source: &str, needle: &str) -> Value {
    json!({"uri":uri,"range":range(source, needle, 2)})
}

fn text_edit(source: &str, needle: &str, length: usize, replacement: &str) -> Value {
    json!({"range":range(source, needle, length),"newText":replacement})
}

fn range(source: &str, needle: &str, length: usize) -> Value {
    let start = position(source, needle);
    let mut end = start.clone();
    end["character"] = json!(start["character"].as_u64().unwrap() + length as u64);
    json!({"start":start,"end":end})
}

fn apply(source: &str, entries: Option<&Value>) -> String {
    let mut edits = entries
        .map(|entries| entries.as_array().unwrap().clone())
        .unwrap_or_default();
    edits.sort_by_key(|edit| {
        (
            edit["range"]["start"]["line"].as_u64().unwrap(),
            edit["range"]["start"]["character"].as_u64().unwrap(),
        )
    });
    let mut result = source.to_owned();
    for edit in edits.iter().rev() {
        result.replace_range(
            offset(source, &edit["range"]["start"])..offset(source, &edit["range"]["end"]),
            edit["newText"].as_str().unwrap(),
        );
    }
    result
}

fn offset(source: &str, position: &Value) -> usize {
    let line = position["line"].as_u64().unwrap() as usize;
    let character = position["character"].as_u64().unwrap() as usize;
    let start: usize = source.split_inclusive('\n').take(line).map(str::len).sum();
    let mut utf16 = 0;
    for (at, ch) in source[start..].char_indices() {
        if utf16 == character {
            return start + at;
        }
        utf16 += ch.len_utf16();
    }
    assert_eq!(utf16, character, "invalid edit position: {position}");
    source.len()
}
