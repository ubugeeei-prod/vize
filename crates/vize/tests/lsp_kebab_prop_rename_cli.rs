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
            "../../../tests/_fixtures/differential/lsp/same-name-rename/7994/supplemental/kebab-multi-component/",
            $name,
            ".vue.txt"
        ))
    };
}

const PANEL: &str = source!("Panel");
const WRAPPER: &str = source!("Wrapper");
const APP: &str = source!("App");

#[derive(Clone, Copy, Debug)]
enum Query {
    WrapperLocal,
    AppPanelPublic,
    AppWrapperPublic,
    HeadingLocal,
}

#[test]
fn original_kebab_local_and_public_origins_keep_the_complete_three_file_contract() {
    assert_transactions(&[Query::WrapperLocal, Query::AppPanelPublic]);
}

#[test]
fn original_same_spelling_foreign_prop_and_heading_controls_preserve_every_other_site() {
    assert_transactions(&[Query::AppWrapperPublic, Query::HeadingLocal]);
}

fn assert_transactions(queries: &[Query]) {
    let mut observed = Vec::new();
    let mut wanted = Vec::new();
    for query in queries {
        for newline in ["\n", "\r\n"] {
            let files = [
                ("Panel.vue", PANEL.replace('\n', newline)),
                ("Wrapper.vue", WRAPPER.replace('\n', newline)),
                ("App.vue", APP.replace('\n', newline)),
            ];
            let (mut fixture, uris) = project(&files);
            let [panel_uri, wrapper_uri, app_uri] = &uris;
            let panel = files[0].1.as_str();
            let wrapper = files[1].1.as_str();
            let app = files[2].1.as_str();
            let heading_use = format!("heading{newline}");
            let heading_directive = format!(":heading{newline}");
            // Full token ranges deliberately reject the original truncated,
            // duplicated and zero-length locations, including the omitted v-if.
            // Both reported cursor positions remain inside their original token.
            let (references, rename, request, goldens) = match query {
                Query::WrapperLocal => (
                    json!([
                        location(wrapper_uri, wrapper, "isOpened }", 8),
                        location(wrapper_uri, wrapper, "is-opened", 9),
                    ]),
                    json!({"changes":{wrapper_uri:[
                        edit(wrapper, "isOpened }", 8, "isOpened: visible"),
                        edit(wrapper, ":is-opened", 10, ":is-opened=\"visible\""),
                    ]}}),
                    (wrapper_uri.as_str(), wrapper, "Opened }", "visible"),
                    [PANEL, source!("LocalWrapper"), APP],
                ),
                Query::AppPanelPublic => (
                    json!([
                        location(app_uri, app, "is-opened=\"true\"", 9),
                        location(panel_uri, panel, "isOpened:", 8),
                        location(panel_uri, panel, "isOpened\"", 8),
                        location(wrapper_uri, wrapper, "is-opened", 9),
                    ]),
                    json!({"changes":{
                        app_uri:[edit(app, "is-opened=\"true\"", 9, "visible")],
                        panel_uri:[
                            edit(panel, "isOpened:", 8, "visible"),
                            edit(panel, "isOpened\"", 8, "visible"),
                        ],
                        wrapper_uri:[edit(wrapper, ":is-opened", 10, ":visible=\"isOpened\"")],
                    }}),
                    (app_uri.as_str(), app, "s-opened=\"true\"", "visible"),
                    [
                        source!("PublicPanel"),
                        source!("PublicWrapper"),
                        source!("PublicApp"),
                    ],
                ),
                Query::AppWrapperPublic => (
                    json!([
                        location(app_uri, app, "is-opened=\"false\"", 9),
                        location(wrapper_uri, wrapper, "isOpened }", 8),
                        location(wrapper_uri, wrapper, "isOpened:", 8),
                    ]),
                    json!({"changes":{
                        app_uri:[edit(app, "is-opened=\"false\"", 9, "expanded")],
                        wrapper_uri:[
                            edit(wrapper, "isOpened }", 8, "expanded: isOpened"),
                            edit(wrapper, "isOpened:", 8, "expanded"),
                        ],
                    }}),
                    (app_uri.as_str(), app, "is-opened=\"false\"", "expanded"),
                    [PANEL, source!("ForeignWrapper"), source!("ForeignApp")],
                ),
                Query::HeadingLocal => (
                    json!([
                        location(wrapper_uri, wrapper, "heading =", 7),
                        location(wrapper_uri, wrapper, &heading_use, 7),
                    ]),
                    json!({"changes":{wrapper_uri:[
                        edit(wrapper, "heading =", 7, "caption"),
                        edit(wrapper, &heading_directive, 8, ":heading=\"caption\""),
                    ]}}),
                    (wrapper_uri.as_str(), wrapper, "heading =", "caption"),
                    [PANEL, source!("HeadingWrapper"), APP],
                ),
            };
            let repaired = [
                ("Panel.vue", goldens[0].replace('\n', newline)),
                ("Wrapper.vue", goldens[1].replace('\n', newline)),
                ("App.vue", goldens[2].replace('\n', newline)),
            ];
            // The whole expected transaction is fixed before the native query.
            let expected = expected(&repaired, references, rename);
            let context = format!("original three-file kebab {query:?}, newline={newline:?}");
            let actual = observe(&mut fixture, &files, &uris, request, &repaired, &context);
            capture(&fixture, &files, &context, &expected, &actual);
            observed.push(json!({"context":context,"observation":actual}));
            wanted.push(json!({"context":context,"observation":expected}));
        }
    }
    assert_eq!(observed, wanted, "all complete original kebab transactions");
}
