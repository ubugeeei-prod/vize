//! Original generic component props retain declared types through real stdio.
#![cfg(test)]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "whole authored sources and LSP JSON fixtures"
)]

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod project;

use project::Fixture;
use serde_json::Value;
use sha2::{Digest, Sha256};

const APP: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/App.vue.txt");
const LIST: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/List.vue.txt");
const COMP: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/Comp.vue.txt");
const CHILD: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/Child.vue.txt");
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/hover.expected.json"
);

#[test]
fn complete_original_generic_hover_sources_match_the_reporter_manifest() {
    let source: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/source.json"
    ))
    .unwrap();
    for (name, bytes) in [
        ("App.vue.txt", APP),
        ("List.vue.txt", LIST),
        ("Comp.vue.txt", COMP),
        ("Child.vue.txt", CHILD),
        (
            "original-issue.md",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/generic-prop-hover-8015/original-issue.md"
            ),
        ),
    ] {
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes.as_bytes())),
            source["originalSha256"][name].as_str().unwrap()
        );
    }
}

#[test]
fn original_native_generic_prop_hovers_keep_whole_declared_types_and_authored_ranges() {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let mut project = Fixture::new_with_vue_component_project(
        APP,
        "App.vue",
        &[("List.vue", LIST), ("Comp.vue", COMP), ("Child.vue", CHILD)],
    );
    project.open(APP);
    for (name, needle) in [("items", "items=\"rows\""), ("title", "title=\"Rows\"")] {
        let actual = project.request("textDocument/hover", APP, needle);
        assert_eq!(actual, expected[name], "complete original {name} hover");
    }
    let dirty = APP
        .replace("<template>\n", "<template>\n  <!-- 😀 -->\n")
        .replace('\n', "\r\n");
    project.change(&dirty, 2);
    for (name, needle) in [("items", "items=\"rows\""), ("title", "title=\"Rows\"")] {
        let actual = project.request("textDocument/hover", &dirty, needle);
        let mut expected = expected[name].clone();
        for point in ["start", "end"] {
            let line = expected["range"][point]["line"].as_u64().unwrap();
            expected["range"][point]["line"] = (line + 1).into();
        }
        assert_eq!(actual, expected, "complete unsaved Unicode {name} hover");
    }
    assert_eq!(project.read_file("App.vue"), APP);
    project.change(APP, 3);
    for (name, needle) in [("items", "items=\"rows\""), ("title", "title=\"Rows\"")] {
        let actual = project.request("textDocument/hover", APP, needle);
        assert_eq!(actual, expected[name], "complete restored {name} hover");
    }
    project.shutdown();
}
