//! Whole contextual data/ARIA completion responses from the source-built CLI.
#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "whole authored source and stdio JSON fixtures"
)]

#[path = "lsp_data_aria_attributes_cli/assignments.rs"]
mod assignments;
#[path = "lsp_data_aria_attributes_cli/controls.rs"]
mod controls;
#[path = "lsp_data_aria_attributes_cli/controls_project.rs"]
mod controls_project;
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "lsp_data_aria_attributes_cli/mirror.rs"]
mod mirror;
#[path = "support/lsp_vue_project.rs"]
mod project;

use project::{Fixture, position};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn source_digest(source: &str) -> std::string::String {
    Sha256::digest(source.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

const COMP: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/Comp.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/Child.vue.txt"
);
const LIST: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/List.vue.txt"
);
const APP: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/App.vue.txt"
);
const DATA: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/data.expected.json"
);
const ARIA: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/aria.expected.json"
);

fn with_edit(mut item: Value, start: Value, end: Value, insert: &str) -> Value {
    item.as_object_mut().unwrap().remove("insertText");
    item["textEdit"] = json!({"range":{"start":start,"end":end},"newText":insert});
    item
}

#[test]
fn parent_original_sources_and_the_independent_aria_vocabulary_remain_byte_exact() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/source.json"
    ))
    .unwrap();
    for (name, source) in [
        ("Comp.vue.txt", COMP),
        ("Child.vue.txt", CHILD),
        ("List.vue.txt", LIST),
        ("App.vue.txt", APP),
        (
            "original-issue.md",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/original-issue.md"
            ),
        ),
    ] {
        assert_eq!(
            source_digest(source),
            manifest["originalSha256"][name].as_str().unwrap()
        );
    }
    let vocabulary = include_str!(
        "../../../tests/_fixtures/differential/lsp/data-aria-attributes-8015/aria-vocabulary.rs.txt"
    );
    assert_eq!(
        source_digest(vocabulary),
        manifest["ariaVocabulary"]["sha256"].as_str().unwrap()
    );
    controls::assert_authored_hashes(&manifest);
}

#[test]
fn original_data_and_full_contextual_aria_completions_survive_native_and_dirty_sources() {
    if controls_project::required_runtime().is_none() {
        return;
    }
    let data: Value = serde_json::from_str(DATA).unwrap();
    let aria: Vec<Value> = serde_json::from_str(ARIA).unwrap();
    let common: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lsp/component-global-attributes-8015/common.expected.json"
    ))
    .unwrap();
    let aria_label = common
        .into_iter()
        .find(|item| item["label"] == "aria-label")
        .unwrap();
    let mut project = Fixture::new_with_vue_component_project(
        COMP,
        "Comp.vue",
        &[("Child.vue", CHILD), ("List.vue", LIST), ("App.vue", APP)],
    );
    project.open(COMP);
    let actual = project.request("textDocument/completion", COMP, "a-role=\"x\"");
    assert_eq!(
        actual,
        json!([with_edit(
            data["data-role"].clone(),
            position(COMP, "data-role"),
            position(COMP, "a-role=\"x\""),
            "dat"
        )])
    );
    let mut version = 2;
    for tag in ["Child", "div"] {
        for (prefix, bank) in [
            ("data-", vec![data["data-*"].clone()]),
            (
                "aria-",
                std::iter::once(aria_label.clone())
                    .chain(aria.clone())
                    .collect(),
            ),
        ] {
            let source = COMP
                .replace("<Child", &format!("<{tag}"))
                .replace("data-role=\"x\"", prefix)
                .replace("<template>\n", "<template>\n  <!-- 😀 -->\n")
                .replace('\n', "\r\n");
            project.change(&source, version);
            version += 1;
            let start = position(&source, prefix);
            let end = position(&source, " />");
            let expected: Vec<Value> = bank
                .into_iter()
                .map(|item| {
                    let insert = item["insertText"].as_str().unwrap().to_string();
                    with_edit(item, start.clone(), end.clone(), &insert)
                })
                .collect();
            assert_eq!(
                project.request("textDocument/completion", &source, " />"),
                json!(expected),
                "whole {tag} {prefix} completion"
            );
        }
    }
    assert_eq!(project.read_file("Comp.vue"), COMP);
    project.change(COMP, version);
    assert_eq!(
        project.request("textDocument/completion", COMP, "a-role=\"x\""),
        json!([with_edit(
            data["data-role"].clone(),
            position(COMP, "data-role"),
            position(COMP, "a-role=\"x\""),
            "dat"
        )])
    );
    project.shutdown();
}
