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

const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/reactive-props-rename/7996/Child.vue.txt"
);
const PARENT: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/reactive-props-rename/7996/Parent.vue.txt"
);
const DEFAULT_CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/reactive-props-rename/7996/supplemental/Default.vue.txt"
);
const ALIASED_CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/reactive-props-rename/7996/supplemental/Aliased.vue.txt"
);

#[derive(Clone, Copy, Debug)]
enum Query {
    PublicDeclaration,
    ParentArgument,
    LocalDeclaration,
    LocalTemplate,
}

#[test]
fn native_reactive_prop_public_rename_is_identical_from_declaration_and_parent() {
    for query in [Query::PublicDeclaration, Query::ParentArgument] {
        assert_rename(query);
    }
}

#[test]
fn native_reactive_prop_local_rename_preserves_the_public_key_and_parent() {
    for query in [Query::LocalDeclaration, Query::LocalTemplate] {
        assert_rename(query);
    }
}

fn assert_rename(query: Query) {
    for binding in ["label", "label = 'Name'", "label: localLabel"] {
        for newline in ["\n", "\r\n"] {
            let local_name = if binding.contains(':') {
                "localLabel"
            } else {
                "label"
            };
            let child = match binding {
                "label" => CHILD,
                "label = 'Name'" => DEFAULT_CHILD,
                "label: localLabel" => ALIASED_CHILD,
                _ => unreachable!("complete authored binding matrix"),
            }
            .replace('\n', newline);
            let parent = PARENT.replace('\n', newline);
            let context = format!("{binding}, {query:?}, newline={newline:?}");
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
            let declaration_needle = format!("{local_name} }}");
            let local_declaration = if binding.contains(':') {
                declaration_needle.as_str()
            } else {
                binding
            };
            let local_template = format!("{local_name} }}}}");
            let (uri, source, needle) = match query {
                Query::PublicDeclaration => (&child_uri, &child, "label: string"),
                Query::ParentArgument => (&parent_uri, &parent, "label />"),
                Query::LocalDeclaration => (&child_uri, &child, local_declaration),
                Query::LocalTemplate => (&child_uri, &child, local_template.as_str()),
            };
            let references = fixture.request_file_with(
                "textDocument/references",
                uri,
                source,
                needle,
                json!({"context":{"includeDeclaration":true}}),
            );
            let response = fixture.request_file_with(
                "textDocument/rename",
                uri,
                source,
                needle,
                json!({"newName":"heading"}),
            );
            let changes = response["changes"]
                .as_object()
                .expect("WorkspaceEdit changes");
            for edited_uri in changes.keys() {
                assert!(
                    edited_uri == &child_uri || edited_uri == &parent_uri,
                    "{response:#}"
                );
            }
            // Apply the actual complete native response before checking its
            // exact identity so even a failure retains real diagnostic proof.
            let repaired_child = apply(&child, changes.get(&child_uri));
            let repaired_parent = apply(&parent, changes.get(&parent_uri));
            fixture.write_file("Child.vue", &repaired_child);
            fixture.write_file("Parent.vue", &repaired_parent);
            let child_diagnostics = fixture.change_file(&child_uri, &repaired_child, 2);
            let parent_diagnostics = fixture.change_file(&parent_uri, &repaired_parent, 2);
            fixture.shutdown();

            let public = matches!(query, Query::PublicDeclaration | Query::ParentArgument);
            let (expected_references, expected_response, expected_child, expected_parent) =
                if public {
                    let binding_edit = if binding.contains(':') {
                        "heading"
                    } else {
                        "heading: label"
                    };
                    (
                        json!([
                            location(&child_uri, &child, binding, 5),
                            location(&child_uri, &child, "label: string", 5),
                            location(&parent_uri, &parent, "label />", 5),
                        ]),
                        json!({"changes":{
                            &child_uri:[
                                text_edit(&child, binding, 5, binding_edit),
                                text_edit(&child, "label: string", 5, "heading"),
                            ],
                            &parent_uri:[text_edit(&parent, ":label", 6, ":heading=\"label\"")],
                        }}),
                        child
                            .replacen(
                                &format!("{{ {binding} }}"),
                                &format!("{{ {} }}", binding.replacen("label", binding_edit, 1)),
                                1,
                            )
                            .replace("label: string", "heading: string"),
                        parent.replace(":label />", ":heading=\"label\" />"),
                    )
                } else {
                    let binding_edit = if binding.contains(':') {
                        "heading"
                    } else {
                        "label: heading"
                    };
                    (
                        json!([
                            location(&child_uri, &child, local_declaration, local_name.len()),
                            location(&child_uri, &child, &local_template, local_name.len()),
                        ]),
                        json!({"changes":{&child_uri:[
                            text_edit(&child, local_declaration, local_name.len(), binding_edit),
                            text_edit(&child, &local_template, local_name.len(), "heading"),
                        ]}}),
                        child
                            .replacen(
                                &format!("{{ {binding} }}"),
                                &format!("{{ {} }}", binding.replace(local_name, binding_edit)),
                                1,
                            )
                            .replace(&format!("{{{{ {local_name} }}}}"), "{{ heading }}"),
                        parent.clone(),
                    )
                };
            assert_eq!(
                json!({
                    "references": references,
                    "rename": response,
                    "child": repaired_child,
                    "parent": repaired_parent,
                    "diskChild": fixture.read_file("Child.vue"),
                    "diskParent": fixture.read_file("Parent.vue"),
                    "childDiagnostics": child_diagnostics,
                    "parentDiagnostics": parent_diagnostics,
                }),
                json!({
                    "references": expected_references,
                    "rename": expected_response,
                    "child": expected_child,
                    "parent": expected_parent,
                    "diskChild": expected_child,
                    "diskParent": expected_parent,
                    "childDiagnostics": [],
                    "parentDiagnostics": [],
                }),
                "{context}"
            );
        }
    }
}

fn location(uri: &str, source: &str, needle: &str, length: usize) -> Value {
    json!({"uri":uri,"range":range(source, needle, length)})
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
