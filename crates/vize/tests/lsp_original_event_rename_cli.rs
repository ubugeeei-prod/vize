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
            "../../../tests/_fixtures/differential/lsp/event-rename/8010/",
            $name,
            ".vue.txt"
        ))
    };
}

const TOGGLE: &str = source!("Toggle");
const APP: &str = source!("App");

#[derive(Clone, Copy, Debug)]
enum Query {
    Declaration,
    EmitCall,
    ParentListener,
}

#[test]
fn original_event_declaration_links_the_complete_key_call_and_parent_transaction() {
    assert_transactions(Query::Declaration);
}

#[test]
fn original_emit_string_links_the_complete_key_call_and_parent_transaction() {
    assert_transactions(Query::EmitCall);
}

#[test]
fn original_parent_listener_links_the_complete_key_call_and_parent_transaction() {
    assert_transactions(Query::ParentListener);
}

fn assert_transactions(query: Query) {
    let mut observed = Vec::new();
    let mut wanted = Vec::new();
    for newline in ["\n", "\r\n"] {
        let files = [
            ("Toggle.vue", TOGGLE.replace('\n', newline)),
            ("App.vue", APP.replace('\n', newline)),
        ];
        let (mut fixture, uris) = project(&files);
        let [toggle_uri, app_uri] = &uris;
        let toggle = files[0].1.as_str();
        let app = files[1].1.as_str();
        // All three source-owned sites are required, with full UTF-16 ranges.
        // Whole equality also rejects library/generated targets and duplicates.
        let references = json!([
            location(app_uri, app, "change=\"onChange\"", 6),
            location(toggle_uri, toggle, "change:", 6),
            location(toggle_uri, toggle, "change\", true", 6),
        ]);
        let rename = json!({"changes":{
            app_uri:[edit(app, "change=\"onChange\"", 6, "update")],
            toggle_uri:[
                edit(toggle, "change:", 6, "update"),
                edit(toggle, "change\", true", 6, "update"),
            ],
        }});
        let repaired = [
            (
                "Toggle.vue",
                source!("UpdatedToggle").replace('\n', newline),
            ),
            ("App.vue", source!("UpdatedApp").replace('\n', newline)),
        ];
        // The reported key 2:3 and call 6:9 cursors stay inside the token.
        let request = match query {
            Query::Declaration => (toggle_uri.as_str(), toggle, "hange:", "update"),
            Query::EmitCall => (toggle_uri.as_str(), toggle, "hange\", true", "update"),
            Query::ParentListener => (app_uri.as_str(), app, "hange=\"onChange\"", "update"),
        };
        // Independent full goldens exist before any references/rename query.
        let expected = expected(&repaired, references, rename);
        let context = format!("original event {query:?}, newline={newline:?}");
        let actual = observe(&mut fixture, &files, &uris, request, &repaired, &context);
        capture(&fixture, &files, &context, &expected, &actual);
        observed.push(json!({"context":context,"observation":actual}));
        wanted.push(json!({"context":context,"observation":expected}));
    }
    assert_eq!(observed, wanted, "all complete original event transactions");
}
