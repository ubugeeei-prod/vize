#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use serde_json::json;

#[path = "support/lsp_authored_rename.rs"]
mod authored;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use authored::{capture, edit, expected, location, observe, project};

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/lsp/reactive-props-rename/7996/supplemental/reported-checked/",
            $name,
            ".vue.txt"
        ))
    };
}

const CHILD: &str = source!("Child");
const PARENT: &str = source!("Parent");

#[derive(Clone, Copy, Debug)]
enum Query {
    PublicDeclaration,
    ParentArgument,
    LocalDeclaration,
    LocalTemplate,
}

#[test]
fn original_checked_public_origins_return_identical_complete_literal_parent_transactions() {
    assert_transactions(&[Query::PublicDeclaration, Query::ParentArgument]);
}

#[test]
fn original_checked_local_controls_keep_the_public_key_and_literal_parent_value() {
    assert_transactions(&[Query::LocalDeclaration, Query::LocalTemplate]);
}

fn assert_transactions(queries: &[Query]) {
    let mut observed = Vec::new();
    let mut wanted = Vec::new();
    for query in queries {
        for newline in ["\n", "\r\n"] {
            let files = [
                ("Child.vue", CHILD.replace('\n', newline)),
                ("Parent.vue", PARENT.replace('\n', newline)),
            ];
            let (mut fixture, uris) = project(&files);
            let [child_uri, parent_uri] = &uris;
            let child = files[0].1.as_str();
            let parent = files[1].1.as_str();
            let public = matches!(query, Query::PublicDeclaration | Query::ParentArgument);
            let (references, rename, goldens) = if public {
                (
                    json!([
                        location(child_uri, child, "checked }", 7),
                        location(child_uri, child, "checked: boolean", 7),
                        location(parent_uri, parent, "checked=\"true\"", 7),
                    ]),
                    json!({"changes":{
                        child_uri:[
                            edit(child, "checked }", 7, "active: checked"),
                            edit(child, "checked: boolean", 7, "active"),
                        ],
                        parent_uri:[edit(parent, "checked=\"true\"", 7, "active")],
                    }}),
                    [source!("PublicChild"), source!("PublicParent")],
                )
            } else {
                (
                    json!([
                        location(child_uri, child, "checked }", 7),
                        location(child_uri, child, "checked ?", 7),
                    ]),
                    json!({"changes":{child_uri:[
                        edit(child, "checked }", 7, "checked: active"),
                        edit(child, "checked ?", 7, "active"),
                    ]}}),
                    [source!("LocalChild"), PARENT],
                )
            };
            let repaired = [
                ("Child.vue", goldens[0].replace('\n', newline)),
                ("Parent.vue", goldens[1].replace('\n', newline)),
            ];
            // The original comment uses human-readable edit coordinates;
            // query the actual complete frozen source token in native UTF-16.
            let request = match query {
                Query::PublicDeclaration => {
                    (child_uri.as_str(), child, "checked: boolean", "active")
                }
                Query::ParentArgument => {
                    (parent_uri.as_str(), parent, "checked=\"true\"", "active")
                }
                Query::LocalDeclaration => (child_uri.as_str(), child, "checked }", "active"),
                Query::LocalTemplate => (child_uri.as_str(), child, "checked ?", "active"),
            };
            let expected = expected(&repaired, references, rename);
            let context = format!("original checked boolean {query:?}, newline={newline:?}");
            let actual = observe(&mut fixture, &files, &uris, request, &repaired, &context);
            capture(&fixture, &files, &context, &expected, &actual);
            observed.push(json!({"context":context,"observation":actual}));
            wanted.push(json!({"context":context,"observation":expected}));
        }
    }
    assert_eq!(
        observed, wanted,
        "all complete original checked transactions"
    );
}
