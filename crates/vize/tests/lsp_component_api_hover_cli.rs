//! Component props, emits, slots and their JSDoc in `vize lsp` hover.
#![cfg(test)]

use crate::support::lsp_vue_project::Fixture;
use serde_json::{Map, Value, json};

pub(crate) const FILES: [(&str, &str); 8] = [
    (
        "Parent.vue",
        include_str!("fixtures/component_api_hover/src/Parent.vue"),
    ),
    (
        "Child.vue",
        include_str!("fixtures/component_api_hover/src/Child.vue"),
    ),
    (
        "DefaultsChild.vue",
        include_str!("fixtures/component_api_hover/src/DefaultsChild.vue"),
    ),
    (
        "RuntimeChild.vue",
        include_str!("fixtures/component_api_hover/src/RuntimeChild.vue"),
    ),
    (
        "ArrayChild.vue",
        include_str!("fixtures/component_api_hover/src/ArrayChild.vue"),
    ),
    (
        "OptionsChild.vue",
        include_str!("fixtures/component_api_hover/src/OptionsChild.vue"),
    ),
    (
        "InterfaceChild.vue",
        include_str!("fixtures/component_api_hover/src/InterfaceChild.vue"),
    ),
    (
        "types.ts",
        include_str!("fixtures/component_api_hover/src/types.ts"),
    ),
];

/// `(case, file, needle)`: the hover is requested at the needle's start.
pub(crate) const CASES: [(&str, &str, &str); 30] = [
    ("tag", "Parent.vue", "Child :title"),
    ("bound prop", "Parent.vue", "title=\"t\""),
    ("static prop", "Parent.vue", "title=\"x\""),
    ("event", "Parent.vue", "change=\"onChange\" v-model"),
    ("v-model prop", "Parent.vue", "count=\"n\""),
    (
        "slot name",
        "Parent.vue",
        "item=\"{ row }\">{{ row }}</template></Child>",
    ),
    (
        "slot prop pattern",
        "Parent.vue",
        "row }\">{{ row }}</template></Child>",
    ),
    ("slot prop use", "Parent.vue", "row }}</template></Child>"),
    ("withDefaults prop", "Parent.vue", "title=\"d\""),
    (
        "withDefaults call-signature event",
        "Parent.vue",
        "change=\"onChange\" />\n  <RuntimeChild",
    ),
    ("runtime prop", "Parent.vue", "title=\"r\""),
    (
        "runtime event",
        "Parent.vue",
        "change=\"onChange\" />\n  <ArrayChild",
    ),
    ("array prop", "Parent.vue", "title=\"a\""),
    (
        "array event",
        "Parent.vue",
        "change=\"onChange\" />\n  <OptionsChild",
    ),
    ("options prop", "Parent.vue", "title=\"o\""),
    (
        "options event",
        "Parent.vue",
        "change=\"onChange\"><template #item",
    ),
    (
        "options slot name",
        "Parent.vue",
        "item=\"{ row }\">{{ row }}</template></OptionsChild>",
    ),
    ("interface tag", "Parent.vue", "InterfaceChild title"),
    ("interface prop", "Parent.vue", "title=\"i\""),
    ("interface kebab prop", "Parent.vue", "item-count"),
    ("interface boolean prop", "Parent.vue", "active v-model"),
    ("interface model", "Parent.vue", "name=\"nm\""),
    (
        "interface aliased event",
        "Parent.vue",
        "change=\"onChange\"><template v-slot",
    ),
    (
        "interface slot name",
        "Parent.vue",
        "item=\"{ row }\">{{ row }}</template></InterfaceChild>",
    ),
    ("script prop", "Child.vue", "title)\nemit"),
    ("script emit", "Child.vue", "change', props.title)"),
    (
        "withDefaults script prop",
        "DefaultsChild.vue",
        "title)\nemit",
    ),
    ("runtime script prop", "RuntimeChild.vue", "title)\nemit"),
    (
        "interface script prop",
        "InterfaceChild.vue",
        "itemCount, name.value)",
    ),
    (
        "interface script model",
        "InterfaceChild.vue",
        "name.value)",
    ),
];

fn source(file: &str) -> &'static str {
    FILES
        .iter()
        .find(|(name, _)| *name == file)
        .map(|(_, source)| *source)
        .expect("fixture file")
}

#[test]
fn component_api_hovers_show_declared_types_and_jsdoc() {
    let expected: Value = serde_json::from_str(include_str!(
        "fixtures/component_api_hover/hover.vize-lsp.json"
    ))
    .unwrap();
    let (parent, children) = FILES.split_first().unwrap();
    let mut project =
        Fixture::new_with_vue_options_api_component_project(parent.1, parent.0, children);
    project.open(parent.1);
    let mut opened = Map::new();
    let mut actual = Map::new();
    for (case, file, needle) in CASES {
        let source = source(file);
        let uri = if file == parent.0 {
            project.uri.clone()
        } else if let Some(Value::String(uri)) = opened.get(file) {
            uri.clone()
        } else {
            let uri = project.write_file(file, source);
            project.open_file(&uri, source);
            opened.insert(file.into(), Value::String(uri.clone()));
            uri
        };
        let hover =
            project.request_file_with("textDocument/hover", &uri, source, needle, json!({}));
        actual.insert(case.into(), hover);
    }
    project.shutdown();
    for (case, _, _) in CASES {
        assert_eq!(actual[case], expected[case], "{case}");
    }
}
