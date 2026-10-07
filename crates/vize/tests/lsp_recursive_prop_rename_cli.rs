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
use support::Fixture;

#[path = "lsp_recursive_prop_rename_cli/edits.rs"]
mod edits;
use edits::{apply, location, location_after, text_edit, text_edit_after};

const RECURSIVE: [(&str, &str); 4] = [
    (
        "inline",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/recursive-prop-roles/Recursive-inline.vue.txt"
        ),
    ),
    (
        "alias",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/recursive-prop-roles/Recursive-alias.vue.txt"
        ),
    ),
    (
        "interface",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/recursive-prop-roles/Recursive-interface.vue.txt"
        ),
    ),
    (
        "withDefaults",
        include_str!(
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/recursive-prop-roles/Recursive-defaults.vue.txt"
        ),
    ),
];
const SHADOW: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/recursive-prop-roles/Recursive-shadow.vue.txt"
);
const PARENT: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/recursive-prop-roles/Parent.vue.txt"
);

#[derive(Clone, Copy, Debug)]
enum Query {
    PublicDeclaration,
    RecursiveArgument,
    ParentArgument,
    ParentLocal,
    RecursiveLocal,
}

#[test]
fn native_recursive_public_prop_rename_changes_both_roles_from_declaration() {
    assert_rename(&RECURSIVE, &[Query::PublicDeclaration], false);
}

#[test]
fn native_recursive_public_prop_rename_changes_both_roles_from_recursive_argument() {
    assert_rename(&RECURSIVE, &[Query::RecursiveArgument], false);
}

#[test]
fn native_recursive_public_prop_rename_changes_both_roles_from_parent_argument() {
    assert_rename(&RECURSIVE, &[Query::ParentArgument], false);
}

#[test]
fn native_recursive_parent_local_rename_keeps_every_recursive_source_byte() {
    assert_rename(&RECURSIVE, &[Query::ParentLocal], false);
}

#[test]
fn native_recursive_public_prop_rename_preserves_its_same_file_shadow_value() {
    assert_rename(
        &[("shadow", SHADOW)],
        &[
            Query::PublicDeclaration,
            Query::RecursiveArgument,
            Query::ParentArgument,
        ],
        true,
    );
}

#[test]
fn native_recursive_shadow_local_rename_keeps_public_property_and_parent() {
    assert_rename(&[("shadow", SHADOW)], &[Query::RecursiveLocal], true);
}

fn assert_rename(sources: &[(&str, &str)], queries: &[Query], shadow: bool) {
    let mut actual = Vec::new();
    let mut expected = Vec::new();
    for &(form, recursive) in sources {
        for newline in ["\n", "\r\n"] {
            let recursive = recursive.replace('\n', newline);
            let parent = PARENT.replace('\n', newline);
            for &query in queries {
                let context = format!("{form}, {query:?}, newline={newline:?}");
                let mut fixture = Fixture::new_with_cross_file_component_project(
                    &recursive,
                    "Recursive.vue",
                    &[("Parent.vue", &parent)],
                );
                let recursive_uri = fixture.uri.clone();
                let parent_uri = fixture.write_file("Parent.vue", &parent);
                let initial_recursive = fixture.open(&recursive);
                let initial_parent = fixture.open_file(&parent_uri, &parent);
                let (query_uri, source, needle) = match query {
                    Query::PublicDeclaration => (&recursive_uri, &recursive, "id: string"),
                    Query::RecursiveArgument => (&recursive_uri, &recursive, "id />"),
                    Query::ParentArgument => (&parent_uri, &parent, "id />"),
                    Query::ParentLocal => (&parent_uri, &parent, "id ="),
                    Query::RecursiveLocal => (&recursive_uri, &recursive, "id ="),
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
                let changes = response["changes"].as_object();
                // A null or differently shaped transaction remains a full
                // observed mismatch. Never substitute expected edits for it.
                let repaired_recursive = apply(
                    &recursive,
                    changes.and_then(|changes| changes.get(&recursive_uri)),
                );
                let repaired_parent = apply(
                    &parent,
                    changes.and_then(|changes| changes.get(&parent_uri)),
                );
                fixture.write_file("Recursive.vue", &repaired_recursive);
                fixture.write_file("Parent.vue", &repaired_parent);
                let recursive_diagnostics =
                    fixture.change_file(&recursive_uri, &repaired_recursive, 2);
                let parent_diagnostics = fixture.change_file(&parent_uri, &repaired_parent, 2);
                let disk_recursive = fixture.read_file("Recursive.vue");
                let disk_parent = fixture.read_file("Parent.vue");
                fixture.shutdown();

                let (expected_references, expected_response, expected_recursive, expected_parent) =
                    expected_transaction(
                        query,
                        shadow,
                        &recursive_uri,
                        &parent_uri,
                        &recursive,
                        &parent,
                    );
                actual.push(json!({
                    "case":context,
                    "initialRecursiveDiagnostics":initial_recursive,
                    "initialParentDiagnostics":initial_parent,
                    "references":references,
                    "rename":response,
                    "appliedRecursive":repaired_recursive,
                    "appliedParent":repaired_parent,
                    "diskRecursive":disk_recursive,
                    "diskParent":disk_parent,
                    "recursiveDiagnostics":recursive_diagnostics,
                    "parentDiagnostics":parent_diagnostics,
                }));
                expected.push(json!({
                    "case":context,
                    "initialRecursiveDiagnostics":[],
                    "initialParentDiagnostics":[],
                    "references":expected_references,
                    "rename":expected_response,
                    "appliedRecursive":expected_recursive,
                    "appliedParent":expected_parent,
                    "diskRecursive":expected_recursive,
                    "diskParent":expected_parent,
                    "recursiveDiagnostics":[],
                    "parentDiagnostics":[],
                }));
            }
        }
    }
    assert_eq!(actual.len(), sources.len() * queries.len() * 2);
    assert_eq!(actual, expected, "all complete native transactions");
}

fn expected_transaction(
    query: Query,
    shadow: bool,
    recursive_uri: &str,
    parent_uri: &str,
    recursive: &str,
    parent: &str,
) -> (Value, Value, String, String) {
    let recursive_argument = "id />";
    // The fixtures have a line break before the closing template; use the
    // complete input tag as the stable marker for its second shorthand.
    let dom_location = location_after(recursive_uri, recursive, "<input :", "id", 2);
    let dom_edit = text_edit_after(recursive, "<input ", ":id", 3, ":id=\"field\"");
    if matches!(query, Query::ParentLocal) {
        return (
            json!([
                location(parent_uri, parent, "id =", 2),
                location(parent_uri, parent, "id />", 2),
            ]),
            json!({"changes":{parent_uri:[
                text_edit(parent, "id =", 2, "field"),
                text_edit(parent, ":id", 3, ":id=\"field\""),
            ]}}),
            recursive.into(),
            parent
                .replace("const id =", "const field =")
                .replace(":id />", ":id=\"field\" />"),
        );
    }
    if matches!(query, Query::RecursiveLocal) {
        return (
            json!([
                location(recursive_uri, recursive, "id =", 2),
                location(recursive_uri, recursive, recursive_argument, 2),
                dom_location,
            ]),
            json!({"changes":{recursive_uri:[
                text_edit(recursive, "id =", 2, "field"),
                text_edit(recursive, ":id", 3, ":id=\"field\""),
                dom_edit,
            ]}}),
            recursive
                .replace("const id =", "const field =")
                .replace(":id />", ":id=\"field\" />"),
            parent.into(),
        );
    }
    let mut references = vec![
        location(parent_uri, parent, "id />", 2),
        location(recursive_uri, recursive, "id: string", 2),
        location(recursive_uri, recursive, recursive_argument, 2),
    ];
    let mut recursive_edits = vec![
        text_edit(recursive, "id: string", 2, "field"),
        text_edit(
            recursive,
            ":id",
            3,
            if shadow {
                ":field=\"id\""
            } else {
                ":field=\"field\""
            },
        ),
    ];
    if !shadow {
        references.push(dom_location);
        recursive_edits.push(dom_edit);
    }
    let repaired = recursive.replace("id: string", "field: string");
    let repaired = if shadow {
        repaired.replacen(":id />", ":field=\"id\" />", 1)
    } else {
        repaired
            .replacen(":id />", ":field=\"field\" />", 1)
            .replace(":id />", ":id=\"field\" />")
    };
    (
        json!(references),
        json!({"changes":{
            parent_uri:[text_edit(parent, ":id", 3, ":field=\"id\"")],
            recursive_uri:recursive_edits,
        }}),
        repaired,
        parent.replace(":id />", ":field=\"id\" />"),
    )
}
