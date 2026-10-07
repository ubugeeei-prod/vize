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
    ($name:expr) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/lsp/reactive-props-rename/7996/supplemental/named-type-reactive/",
            $name,
            ".vue.txt"
        ))
    };
}

macro_rules! form {
    ($style:literal, $binding:literal, $kind:expr) => {
        Form {
            name: concat!($style, "-", $binding),
            binding: $kind,
            child: source!(concat!($style, "/", $binding, "/Child")),
            public_child: source!(concat!($style, "/", $binding, "/PublicChild")),
            local_child: source!(concat!($style, "/", $binding, "/LocalChild")),
        }
    };
}

const PARENT: &str = source!("Parent");
const PUBLIC_PARENT: &str = source!("PublicParent");
const FORMS: [Form; 6] = [
    form!("type-alias", "shorthand", Binding::Shorthand),
    form!("type-alias", "default", Binding::Default),
    form!("type-alias", "explicit-alias", Binding::Alias),
    form!("interface", "shorthand", Binding::Shorthand),
    form!("interface", "default", Binding::Default),
    form!("interface", "explicit-alias", Binding::Alias),
];

struct Form {
    name: &'static str,
    binding: Binding,
    child: &'static str,
    public_child: &'static str,
    local_child: &'static str,
}

#[derive(Clone, Copy)]
enum Binding {
    Shorthand,
    Default,
    Alias,
}

#[derive(Clone, Copy, Debug)]
enum Query {
    PublicDeclaration,
    ParentArgument,
    LocalDeclaration,
    LocalTemplate,
}

#[test]
fn native_named_type_reactive_public_origins_preserve_complete_local_and_parent_roles() {
    assert_transactions(&[Query::PublicDeclaration, Query::ParentArgument]);
}

#[test]
fn native_named_type_reactive_local_origins_preserve_complete_public_and_parent_roles() {
    assert_transactions(&[Query::LocalDeclaration, Query::LocalTemplate]);
}

fn assert_transactions(queries: &[Query]) {
    let mut observed = Vec::new();
    let mut wanted = Vec::new();
    for form in &FORMS {
        let (property, local, declaration, public_edit, local_edit) = match form.binding {
            Binding::Shorthand => (
                "label } = defineProps",
                "label",
                "label } = defineProps",
                "heading: label",
                "label: heading",
            ),
            Binding::Default => (
                "label = 'Name' } = defineProps",
                "label",
                "label = 'Name' } = defineProps",
                "heading: label",
                "label: heading",
            ),
            Binding::Alias => (
                "label: localLabel } = defineProps",
                "localLabel",
                "localLabel } = defineProps",
                "heading",
                "heading",
            ),
        };
        let template = format!("{local} }}}}</p>");
        for query in queries {
            for newline in ["\n", "\r\n"] {
                let files = [
                    ("Child.vue", form.child.replace('\n', newline)),
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
                            location(child_uri, child, "label: string", 5),
                            location(child_uri, child, property, 5),
                            location(parent_uri, parent, "label />", 5),
                        ]),
                        json!({"changes":{
                            child_uri:[
                                edit(child, "label: string", 5, "heading"),
                                edit(child, property, 5, public_edit),
                            ],
                            parent_uri:[edit(parent, ":label", 6, ":heading=\"label\"")],
                        }}),
                        [form.public_child, PUBLIC_PARENT],
                    )
                } else {
                    (
                        json!([
                            location(child_uri, child, declaration, local.len()),
                            location(child_uri, child, &template, local.len()),
                        ]),
                        json!({"changes":{child_uri:[
                            edit(child, declaration, local.len(), local_edit),
                            edit(child, &template, local.len(), "heading"),
                        ]}}),
                        [form.local_child, PARENT],
                    )
                };
                let repaired = [
                    ("Child.vue", goldens[0].replace('\n', newline)),
                    ("Parent.vue", goldens[1].replace('\n', newline)),
                ];
                let request = match query {
                    Query::PublicDeclaration => {
                        (child_uri.as_str(), child, "label: string", "heading")
                    }
                    Query::ParentArgument => (parent_uri.as_str(), parent, "label />", "heading"),
                    Query::LocalDeclaration => (child_uri.as_str(), child, declaration, "heading"),
                    Query::LocalTemplate => {
                        (child_uri.as_str(), child, template.as_str(), "heading")
                    }
                };
                // Construct the complete authored oracle before sending any
                // references or rename query. The driver applies actual edits
                // and preserves version 2 before installing version 3 goldens.
                let expected = expected(&repaired, references, rename);
                let context = format!(
                    "named reactive {} {query:?}, newline={newline:?}",
                    form.name
                );
                let actual = observe(&mut fixture, &files, &uris, request, &repaired, &context);
                capture(&fixture, &files, &context, &expected, &actual);
                observed.push(json!({"context":context,"observation":actual}));
                wanted.push(json!({"context":context,"observation":expected}));
            }
        }
    }
    assert_eq!(
        observed, wanted,
        "all complete named-type reactive transactions"
    );
}
