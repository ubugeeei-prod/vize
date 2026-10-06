//! Whole tag completion responses distinguish real components from typed data.
#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "whole authored source and protocol fixtures"
)]

#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/component_tag_project.rs"]
mod project;

use project::Project;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-tag-classification/App.vue.txt"
);
const CONTROLS: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-tag-classification/Controls.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/lsp/component-tag-classification/responses.authored.json"
);

fn digest(source: &str) -> std::string::String {
    Sha256::digest(source.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn original_pins() {
    let case: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lsp/component-tag-classification/case.json"
    ))
    .unwrap();
    for (name, source) in [
        ("App.vue.txt", ORIGINAL),
        (
            "MyButton.vue.txt",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/component-tag-classification/MyButton.vue.txt"
            ),
        ),
        (
            "tsconfig.json.txt",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/component-tag-classification/tsconfig.json.txt"
            ),
        ),
        (
            "vize.config.json.txt",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/component-tag-classification/vize.config.json.txt"
            ),
        ),
        (
            "original-issue-body.md",
            include_str!(
                "../../../tests/_fixtures/differential/lsp/component-tag-classification/original-issue-body.md"
            ),
        ),
    ] {
        assert_eq!(
            digest(source),
            case["originalSha256"][name].as_str().unwrap()
        );
    }
    assert_eq!(
        digest(CONTROLS),
        case["authoredSha256"]["Controls.vue.txt"].as_str().unwrap()
    );
    assert_eq!(
        digest(EXPECTED),
        case["authoredSha256"]["responses.authored.json"]
            .as_str()
            .unwrap()
    );
}

fn caret(source: &str, prefix: &str) -> usize {
    source.rfind(prefix).unwrap() + prefix.len()
}

fn item(label: &str, detail: &str, source: &str, prefix: &str, tail: &str) -> Value {
    let end = caret(source, prefix);
    let start = end - prefix.len() + 1;
    let point = |byte| {
        let before = &source[..byte];
        json!({"line":before.bytes().filter(|byte| *byte == b'\n').count(),"character":before.rsplit('\n').next().unwrap().encode_utf16().count()})
    };
    json!({"label":label,"kind":7,"detail":detail,"insertTextFormat":1,
        "textEdit":{"range":{"start":point(start),"end":point(end+tail.len())},"newText":label},"sortText":label})
}

#[test]
fn original_capitalized_data_has_whole_component_and_native_tag_responses() {
    original_pins();
    let expected: Vec<Value> = serde_json::from_str(EXPECTED).unwrap();
    for newline in ["\n", "\r\n"] {
        let original = ORIGINAL.replace('\n', newline);
        let mut project = Project::new(&original, true);
        project.open(&original);
        for (index, row) in expected.iter().enumerate() {
            let prefix = row["prefix"].as_str().unwrap();
            let source = original
                .replace("  <\r\n", &format!("  {prefix}\r\n"))
                .replace("  <\n", &format!("  {prefix}\n"));
            project.change(&source, index as i64 + 2);
            project.completion(&source, caret(&source, prefix), row["result"].clone());
            project.unchanged_disk(&original);
        }
        project.shutdown();
    }
}

#[test]
fn disabled_typecheck_retains_exact_original_name_guessing() {
    original_pins();
    let mut project = Project::new(ORIGINAL, false);
    project.open(ORIGINAL);
    for (index, (prefix, label)) in [("<Di", "DialogState"), ("<SO", "SORT_OPTIONS")]
        .into_iter()
        .enumerate()
    {
        let source = ORIGINAL.replace("  <\n", &format!("  {prefix}\n"));
        project.change(&source, index as i64 + 2);
        project.completion(
            &source,
            caret(&source, prefix),
            json!([item(
                label,
                "Component in script setup",
                &source,
                prefix,
                ""
            )]),
        );
    }
    project.unchanged_disk(ORIGINAL);
    project.shutdown();
}

#[test]
fn genuine_factories_functional_types_and_imports_survive_known_data_and_shadows() {
    original_pins();
    for newline in ["\n", "\r\n"] {
        let original = CONTROLS.replace('\n', newline);
        let mut project = Project::new(&original, true);
        project.open(&original);
        let queries = [
            ("<Loc", Some("LocalCard")),
            ("<Asy", Some("AsyncCard")),
            ("<Func", Some("FunctionalCard")),
            ("<Typ", Some("TypedCard")),
            ("<Dyn", Some("DynamicCard")),
            ("<chi", Some("child")),
            ("<Unk", Some("UnknownCard")),
            ("<Any", Some("AnyCard")),
            ("<SO", None),
            ("<Di", None),
            ("<RefD", None),
            ("<Shad", None),
            ("<VoidF", Some("VoidFunction")),
            ("<NumberF", Some("NumberFunction")),
            ("<Imp", None),
        ];
        for (index, (prefix, label)) in queries.into_iter().enumerate() {
            let source = original.replace("😀 <", &format!("😀 {prefix}"));
            project.change(&source, index as i64 + 2);
            let expected = label
                .map(|label| {
                    vec![item(
                        label,
                        "Component in script setup",
                        &source,
                        prefix,
                        "",
                    )]
                })
                .unwrap_or_default();
            project.completion(&source, caret(&source, prefix), json!(expected));
            project.unchanged_disk(&original);
        }
        let source = original.replace("😀 <", "😀 <LocalSuffix");
        project.change(&source, 17);
        project.completion(
            &source,
            caret(&source, "<Loc"),
            json!([item(
                "LocalCard",
                "Component in script setup",
                &source,
                "<Loc",
                "alSuffix"
            )]),
        );
        project.shutdown();
    }
}

#[test]
fn unsaved_same_name_component_data_and_restore_rebind_native_type() {
    original_pins();
    let original = CONTROLS.replace("😀 <", "😀 <Loc");
    let data = original.replace(
        "const LocalCard = build({ render: () => h(\"div\") });",
        "const LocalCard = { None: \"None\" } as const;",
    );
    let mut project = Project::new(&original, true);
    project.open(&original);
    project.completion(
        &original,
        caret(&original, "<Loc"),
        json!([item(
            "LocalCard",
            "Component in script setup",
            &original,
            "<Loc",
            ""
        )]),
    );
    project.change(&data, 2);
    project.completion(&data, caret(&data, "<Loc"), json!([]));
    project.change(&original, 3);
    project.completion(
        &original,
        caret(&original, "<Loc"),
        json!([item(
            "LocalCard",
            "Component in script setup",
            &original,
            "<Loc",
            ""
        )]),
    );
    project.unchanged_disk(&original);
    project.shutdown();
}
